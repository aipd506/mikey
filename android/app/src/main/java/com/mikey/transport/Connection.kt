package com.mikey.transport

import java.io.Closeable
import java.io.InputStream
import java.io.OutputStream

/** One way to reach the PC. */
interface Transport {
    /** 1 = USB debugging, 2 = USB tethering, 3 = Bluetooth, 4 = Wi-Fi. */
    val level: Int

    /** Where it leads: an address, for the logs. */
    val host: String

    /** Opens the link, or throws. */
    fun open(): Connection
}

/** An open link to the PC, whatever carries it. */
class Connection(
    val level: Int,
    val host: String,
    val input: InputStream,
    val output: OutputStream,
    private val setReadTimeout: (Int) -> Unit,
    private val closer: () -> Unit,
) : Closeable {
    /** Bounds every blocking read: a read that takes longer fails and the link is dead. 0 means no bound. */
    var readTimeoutMs: Int = 0
        set(value) {
            field = value
            setReadTimeout(value)
        }

    override fun close() = closer()
}
