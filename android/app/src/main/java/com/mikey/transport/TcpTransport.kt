package com.mikey.transport

import android.net.Network
import java.io.IOException
import java.net.InetAddress
import java.net.InetSocketAddress
import java.net.Socket

/**
 * One way to reach the PC over TCP. [level] is 1 = USB debugging, 2 = USB tethering, 4 = Wi-Fi.
 * With [network] the socket is pinned to that network, so the PC is reached even when the phone's
 * default network is mobile data. With [bindTo] it leaves through that interface's address instead.
 */
class TcpTransport private constructor(
    val host: String,
    private val port: Int,
    val level: Int,
    private val connectTimeoutMs: Int,
    private val network: Network? = null,
    private val bindTo: InetAddress? = null,
) {
    fun open(): Socket {
        val socket = network?.socketFactory?.createSocket() ?: Socket()
        try {
            bindTo?.let { socket.bind(InetSocketAddress(it, 0)) }
            socket.tcpNoDelay = true
            socket.soTimeout = LINK_TIMEOUT_MS
            socket.connect(InetSocketAddress(host, port), connectTimeoutMs)
        } catch (e: IOException) {
            socket.close()
            throw e
        }
        return socket
    }

    companion object {
        /** The PC listens here on every level. */
        const val PC_PORT = 7653

        /** No frame from the PC for this long means the link is dead. */
        const val LINK_TIMEOUT_MS = 6_000

        /** Through the `adb reverse` tunnel. Localhost answers at once, so a short timeout is enough. */
        fun adb() = TcpTransport("127.0.0.1", PC_PORT, level = 1, connectTimeoutMs = 300)

        /** A typed-in address (debug builds). */
        fun manual(address: String, network: Network? = null) = TcpTransport(address, PC_PORT, level = 4, connectTimeoutMs = 2_000, network)

        /** Where the PC was last time. Tried before searching, with a short timeout in case it moved. */
        fun lastKnown(address: String, network: Network? = null) = TcpTransport(address, PC_PORT, level = 4, connectTimeoutMs = 1_000, network)

        /** A PC that answered our probe, reached over the interface that carried the answer. */
        fun discovered(pc: DiscoveredPc, network: Network? = null) = TcpTransport(
            pc.address.hostAddress ?: pc.address.toString(),
            pc.port,
            pc.level,
            connectTimeoutMs = 2_000,
            network,
            // A tether interface has no Network object to pin to, so pin to its address instead.
            bindTo = if (network == null) pc.via?.address else null,
        )
    }
}
