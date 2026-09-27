package com.mikey.transport

import android.util.Log
import com.mikey.settings.Settings
import java.io.IOException
import java.net.Socket

/** Picks the best way to the PC that works right now: USB first, then Wi-Fi. */
class TransportManager(private val settings: Settings, private val discovery: Discovery) {

    class Connection(val socket: Socket, val transport: TcpTransport)

    /** Tries each way in order and returns the first that connects. Throws when none does. */
    fun open(): Connection {
        for (transport in candidates()) {
            try {
                return Connection(transport.open(), transport)
            } catch (e: IOException) {
                Log.d(TAG, "Not via level ${transport.level} at ${transport.host}: $e")
            }
        }
        throw IOException("No PC reachable")
    }

    // Lazy on purpose: discovery only broadcasts when nothing cheaper worked.
    private fun candidates(): Sequence<TcpTransport> = sequence {
        yield(TcpTransport.adb())
        settings.manualPcAddress?.let { yield(TcpTransport.manual(it)) }
        settings.lastPcAddress?.let { yield(TcpTransport.lastKnown(it)) }
        for (pc in discovery.find(preferredPcId = settings.pairedPc?.id)) yield(TcpTransport.discovered(pc))
    }

    private companion object {
        const val TAG = "TransportManager"
    }
}
