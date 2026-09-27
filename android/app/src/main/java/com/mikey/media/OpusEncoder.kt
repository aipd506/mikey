package com.mikey.media

/**
 * Encodes 48 kHz mono PCM s16le with libopus (cpp/opus_jni.c). One instance per stream, used from
 * one thread. The packet lands in [packet] so no copy is made per frame.
 */
class OpusEncoder(application: Application, bitrate: Int) : AutoCloseable {

    enum class Application(val code: Int) {
        /** Lowest delay, music-grade: Wi-Fi. */
        LOW_DELAY(2051),

        /** Tuned for speech and in-band FEC: Bluetooth. */
        VOIP(2048),
    }

    /** The last packet. Valid for the length [encode] returned, until the next call. */
    val packet = ByteArray(MAX_PACKET)

    private var handle = create(SAMPLE_RATE, 1, application.code, bitrate)

    init {
        check(handle != 0L) { "Can't create the Opus encoder" }
    }

    /** [pcm] is one frame of s16le samples, e.g. 960 bytes for 10 ms. Returns the packet length, or 0 if encoding failed. */
    fun encode(pcm: ByteArray): Int = encode(handle, pcm, pcm.size / 2, packet).coerceAtLeast(0)

    override fun close() {
        if (handle != 0L) destroy(handle)
        handle = 0
    }

    private companion object {
        const val SAMPLE_RATE = 48_000

        /** The largest packet Opus can produce for one frame. */
        const val MAX_PACKET = 1275

        init {
            System.loadLibrary("mikey")
        }

        @JvmStatic external fun create(sampleRate: Int, channels: Int, application: Int, bitrate: Int): Long

        @JvmStatic external fun encode(handle: Long, pcm: ByteArray, frameSamples: Int, out: ByteArray): Int

        @JvmStatic external fun destroy(handle: Long)
    }
}
