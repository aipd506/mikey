package com.mikey.protocol

import java.io.DataOutputStream

/** Start of every AUDIO/VIDEO payload: seq u32 | capture_ts u64 µs | codec u8 | reserved u8. */
object MediaHeader {
    const val SIZE = 14
    const val CODEC_PCM_S16LE = 0x01
    const val CODEC_OPUS = 0x02

    fun write(out: DataOutputStream, seq: Int, captureTimeUs: Long, codec: Int) {
        out.writeInt(seq)
        out.writeLong(captureTimeUs)
        out.writeByte(codec)
        out.writeByte(0)
    }
}
