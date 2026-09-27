package com.mikey.media

/**
 * Joins consecutive 10 ms capture frames into one of [framesPerPacket] frames, for codecs that
 * want longer frames: Opus on Bluetooth uses 20 ms. With 1 it just passes frames through.
 */
class FrameJoiner(private val framesPerPacket: Int) {
    private var buffer = ByteArray(0)
    private var filled = 0
    private var seq = 0
    private var captureTimeUs = 0L

    /** Returns the joined frame once enough have been added, else null. It is valid until the next call. */
    fun add(frame: AudioFrame): AudioFrame? {
        if (framesPerPacket == 1) return frame
        if (buffer.size != frame.pcm.size * framesPerPacket) buffer = ByteArray(frame.pcm.size * framesPerPacket)
        if (filled == 0) {
            seq = frame.seq
            captureTimeUs = frame.captureTimeUs
        }
        System.arraycopy(frame.pcm, 0, buffer, filled * frame.pcm.size, frame.pcm.size)
        filled++
        if (filled < framesPerPacket) return null
        filled = 0
        return AudioFrame(seq, captureTimeUs, buffer)
    }
}
