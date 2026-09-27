package com.mikey.transport

import java.io.IOException
import java.net.InetSocketAddress
import java.net.Socket

/** One way to reach the PC over TCP. [level] is 1 = USB debugging, 4 = Wi-Fi. */
class TcpTransport private constructor(
    val host: String,
    private val port: Int,
    val level: Int,
    private val connectTimeoutMs: Int,
) {
    fun open(): Socket {
        val socket = Socket()
        try {
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
        fun manual(address: String) = TcpTransport(address, PC_PORT, level = 4, connectTimeoutMs = 2_000)

        /** Where the PC was last time. Tried before searching, with a short timeout in case it moved. */
        fun lastKnown(address: String) = TcpTransport(address, PC_PORT, level = 4, connectTimeoutMs = 1_000)

        /** A PC that answered our discovery probe. */
        fun discovered(pc: DiscoveredPc) = TcpTransport(pc.address.hostAddress ?: pc.address.toString(), pc.port, level = 4, connectTimeoutMs = 2_000)
    }
}
