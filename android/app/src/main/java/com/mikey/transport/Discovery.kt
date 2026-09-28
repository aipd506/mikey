package com.mikey.transport

import android.util.Log
import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.NetworkInterface
import java.net.SocketException
import java.net.SocketTimeoutException

/** A network interface we can probe: its name, our address on it, and its broadcast address. */
class NetInterface(val name: String, val address: InetAddress, val prefixLength: Int, val broadcast: InetAddress) {
    /** 2 for USB tethering, 3 for Wi-Fi and everything else (connection-levels.md). */
    val level: Int = levelFor(name)

    fun contains(other: InetAddress): Boolean = inSubnet(other.address, address.address, prefixLength)
}

/** A PC that answered our probe, and the interface the answer came in on. */
class DiscoveredPc(val id: String, val name: String, val address: InetAddress, val port: Int, val proto: Int, val via: NetInterface?) {
    val level: Int get() = via?.level ?: 3
}

/**
 * Finds PCs on the local networks with the UDP beacon (connection-levels.md): one probe broadcast
 * on every interface that has a broadcast address, then a short wait for the unicast answers.
 * Mobile data has no broadcast address, so no probe ever goes out over it.
 */
class Discovery(private val deviceId: String, private val deviceName: String) {

    /** Interfaces that are up, not loopback, and have an IPv4 address with a broadcast address. */
    fun interfaces(): List<NetInterface> =
        try {
            NetworkInterface.getNetworkInterfaces()?.toList().orEmpty()
                .filter { it.isUp && !it.isLoopback }
                .flatMap { nic ->
                    nic.interfaceAddresses.mapNotNull { ia ->
                        ia.broadcast?.let { NetInterface(nic.name, ia.address, ia.networkPrefixLength.toInt(), it) }
                    }
                }
        } catch (e: SocketException) {
            emptyList()
        }

    fun find(preferredPcId: String?): List<DiscoveredPc> = find(preferredPcId, interfaces())

    /**
     * Returns the PCs that answered: [preferredPcId] first, then the best level first. Returns as
     * soon as the preferred one answers, otherwise after [WAIT_MS].
     */
    internal fun find(preferredPcId: String?, interfaces: List<NetInterface>): List<DiscoveredPc> {
        if (interfaces.isEmpty()) return emptyList()
        val probe = probePayload(deviceId, deviceName)
        val found = LinkedHashMap<String, DiscoveredPc>()
        try {
            DatagramSocket().use { socket ->
                socket.broadcast = true
                for (target in interfaces) {
                    try {
                        socket.send(DatagramPacket(probe, probe.size, target.broadcast, BEACON_PORT))
                    } catch (e: IOException) {
                        Log.d(TAG, "Can't probe ${target.name}: $e")
                    }
                }
                val deadline = System.nanoTime() + WAIT_MS * 1_000_000
                val packet = DatagramPacket(ByteArray(MAX_REPLY), MAX_REPLY)
                while (true) {
                    val remainingMs = (deadline - System.nanoTime()) / 1_000_000
                    if (remainingMs <= 0) break
                    socket.soTimeout = remainingMs.toInt()
                    packet.length = MAX_REPLY
                    try {
                        socket.receive(packet)
                    } catch (e: SocketTimeoutException) {
                        break
                    }
                    val reply = parseReply(packet.data, packet.length) ?: continue
                    val via = interfaces.firstOrNull { it.contains(packet.address) }
                    found.putIfAbsent(reply.id, DiscoveredPc(reply.id, reply.name, packet.address, reply.port, reply.proto, via))
                    if (reply.id == preferredPcId) break
                }
            }
        } catch (e: IOException) {
            Log.d(TAG, "Discovery failed: $e")
        }
        return found.values.sortedWith(compareBy({ it.id != preferredPcId }, { it.level }))
    }

    private companion object {
        const val TAG = "Discovery"
        const val BEACON_PORT = 7654
        const val WAIT_MS = 1_000L
        const val MAX_REPLY = 512
    }
}

/** What a PC's beacon answer says. */
internal class BeaconReply(val id: String, val name: String, val port: Int, val proto: Int)

private val PROBE_MAGIC = "MIKEY?1".toByteArray()
private val REPLY_MAGIC = "MIKEY!1".toByteArray()

/** Magic, 16 id bytes, port, proto version, name length: the reply's fixed part. */
private const val REPLY_FIXED = 7 + 16 + 2 + 1 + 1

/** `"MIKEY?1" | device_id (16 bytes) | name_len (1) | name`. */
internal fun probePayload(deviceId: String, deviceName: String): ByteArray {
    val name = deviceName.toByteArray().let { if (it.size > 255) it.copyOf(255) else it }
    return PROBE_MAGIC + idBytes(deviceId) + byteArrayOf(name.size.toByte()) + name
}

/** `"MIKEY!1" | pc_id (16) | tcp_port (u16 BE) | proto_ver (u8) | name_len (1) | name`, or null if it isn't one. */
internal fun parseReply(data: ByteArray, length: Int): BeaconReply? {
    if (length < REPLY_FIXED || !data.copyOfRange(0, 7).contentEquals(REPLY_MAGIC)) return null
    val id = data.copyOfRange(7, 23).joinToString("") { "%02x".format(it) }
    val port = ((data[23].toInt() and 0xFF) shl 8) or (data[24].toInt() and 0xFF)
    val proto = data[25].toInt() and 0xFF
    val nameLength = data[26].toInt() and 0xFF
    if (length < REPLY_FIXED + nameLength) return null
    return BeaconReply(id, String(data, REPLY_FIXED, nameLength), port, proto)
}

/** USB tethering interfaces are named rndis*, usb* or ncm* (connection-levels.md). Everything else counts as Wi-Fi. */
internal fun levelFor(interfaceName: String): Int =
    if (interfaceName.startsWith("rndis") || interfaceName.startsWith("usb") || interfaceName.startsWith("ncm")) 2 else 3

/** Whether [ip] is inside [network]/[prefixLength]. */
internal fun inSubnet(ip: ByteArray, network: ByteArray, prefixLength: Int): Boolean {
    if (ip.size != network.size) return false
    var bits = prefixLength
    for (i in ip.indices) {
        if (bits <= 0) return true
        val mask = if (bits >= 8) 0xFF else (0xFF shl (8 - bits)) and 0xFF
        if ((ip[i].toInt() and mask) != (network[i].toInt() and mask)) return false
        bits -= 8
    }
    return true
}

/** The 32-hex device id as its 16 raw bytes. */
private fun idBytes(hex: String) = ByteArray(16) { hex.substring(it * 2, it * 2 + 2).toInt(16).toByte() }
