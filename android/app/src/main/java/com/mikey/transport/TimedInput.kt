package com.mikey.transport

import java.io.InputStream
import kotlin.concurrent.thread

/**
 * Gives a stream a read timeout, for links whose streams have none, like Bluetooth. A read that
 * blocks longer than [timeoutMs] gets the link closed, which makes the read fail the way a
 * socket timeout would. 0 means no timeout.
 */
class TimedInput(private val input: InputStream, private val closeLink: () -> Unit) : InputStream() {
    @Volatile var timeoutMs = 0

    @Volatile private var deadlineNs = Long.MAX_VALUE

    @Volatile private var closed = false

    init {
        thread(name = "mikey-read-timeout", isDaemon = true) {
            while (!closed) {
                try {
                    Thread.sleep(TICK_MS)
                } catch (e: InterruptedException) {
                    break
                }
                if (System.nanoTime() > deadlineNs) {
                    closeLink()
                    break
                }
            }
        }
    }

    override fun read(): Int = timed { input.read() }

    override fun read(b: ByteArray, off: Int, len: Int): Int = timed { input.read(b, off, len) }

    override fun close() {
        closed = true
        input.close()
    }

    private inline fun timed(read: () -> Int): Int {
        val timeout = timeoutMs
        deadlineNs = if (timeout > 0) System.nanoTime() + timeout * 1_000_000L else Long.MAX_VALUE
        try {
            return read()
        } finally {
            deadlineNs = Long.MAX_VALUE
        }
    }

    private companion object {
        const val TICK_MS = 250L
    }
}
