package com.mikey.media

import android.os.Build

/**
 * Low-latency raw capture with AAudio (cpp/aaudio_jni.c): mono 16-bit at the asked rate, with the
 * phone's own processing off. Android 9 and up; older phones use AudioRecord instead.
 */
class AAudioInput private constructor(private var handle: Long) : AutoCloseable {

    /** Fills [pcm] with one frame of samples. Returns the samples read, or a negative AAudio error. */
    fun read(pcm: ByteArray): Int = nativeRead(handle, pcm, pcm.size / 2, READ_TIMEOUT_NS)

    override fun close() {
        if (handle != 0L) nativeClose(handle)
        handle = 0
    }

    companion object {
        private const val READ_TIMEOUT_NS = 200_000_000L

        /** Null when AAudio can't give exactly what we need; then AudioRecord takes over. */
        fun open(sampleRate: Int): AAudioInput? {
            if (Build.VERSION.SDK_INT < Build.VERSION_CODES.P) return null
            val handle = nativeOpen(sampleRate)
            return if (handle == 0L) null else AAudioInput(handle)
        }

        init {
            System.loadLibrary("mikey")
        }

        @JvmStatic private external fun nativeOpen(sampleRate: Int): Long

        @JvmStatic private external fun nativeRead(handle: Long, pcm: ByteArray, frames: Int, timeoutNs: Long): Int

        @JvmStatic private external fun nativeClose(handle: Long)
    }
}
