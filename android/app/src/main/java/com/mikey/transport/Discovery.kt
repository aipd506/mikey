package com.mikey.transport

import android.util.Log
import java.io.IOException
import java.net.DatagramPacket
import java.net.DatagramSocket
import java.net.InetAddress
import java.net.NetworkInterface
import java.net.SocketException
import java.net.SocketTimeoutException

/** A PC that answered our probe. */
class DiscoveredPc(val id: String, val name: String, val address: InetAddress, val port: Int, val proto: Int)

/**
 * Finds PCs on the local networks with the UDP beacon (connection-levels.md): one probe broadcast
 * on every interface that has a broadcast address, then a short wait for the unicast answers.
 * Mobile data has no broadcast address, so no probe ever goes out over it.
 */
class Discovery(private val deviceId: String, private val deviceName: String) {

    /**
     * Returns the PCs that answered, with [preferredPcId] first. Returns as soon as that one
     * answers, otherwise after [WAIT_MS].
     */
    fun find(preferredPcId: String?): List<DiscoveredPc> = find(preferredPcId, broadcastAddresses())

    internal fun find(preferredPcId: String?, targets: List<InetAddress>): List<DiscoveredPc> {
        if (targets.isEmpty()) return emptyList()
        val probe = probePayload(deviceId, deviceName)
        val found = LinkedHashMap<String, DiscoveredPc>()
        try {
            DatagramSocket().use { socket ->
                socket.broadcast = true
                for (target in targets) {
                    try {
                        socket.send(DatagramPacket(probe, probe.size, target, BEACON_PORT))
                    } catch (e: IOException) {
                        Log.d(TAG, "Can't probe $target: $e")
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
                    val pc = parseReply(packet.data, packet.length, packet.address) ?: continue
                    found.putIfAbsent(pc.id, pc)
                    if (pc.id == preferredPcId) break
                }
            }
        } catch (e: IOException) {
            Log.d(TAG, "Discovery failed: $e")
        }
        return found.values.sortedByDescending { it.id == preferredPcId }
    }

    private fun broadcastAddresses(): List<InetAddress> =
        try {
            NetworkInterface.getNetworkInterfaces()?.toList().orEmpty()
                .filter { it.isUp && !it.isLoopback }
                .flatMap { it.interfaceAddresses }
                .mapNotNull { it.broadcast }
        } catch (e: SocketException) {
            emptyList()
        }

    private companion object {
        const val TAG = "Discovery"
        const val BEACON_PORT = 7654
        const val WAIT_MS = 1_000L
        const val MAX_REPLY = 512
    }
}

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
internal fun parseReply(data: ByteArray, length: Int, from: InetAddress): DiscoveredPc? {
    if (length < REPLY_FIXED || !data.copyOfRange(0, 7).contentEquals(REPLY_MAGIC)) return null
    val id = data.copyOfRange(7, 23).joinToString("") { "%02x".format(it) }
    val port = ((data[23].toInt() and 0xFF) shl 8) or (data[24].toInt() and 0xFF)
    val proto = data[25].toInt() and 0xFF
    val nameLength = data[26].toInt() and 0xFF
    if (length < REPLY_FIXED + nameLength) return null
    return DiscoveredPc(id, String(data, REPLY_FIXED, nameLength), from, port, proto)
}

/** The 32-hex device id as its 16 raw bytes. */
private fun idBytes(hex: String) = ByteArray(16) { hex.substring(it * 2, it * 2 + 2).toInt(16).toByte() }
