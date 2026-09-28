package com.mikey.media

import kotlin.math.log10
import kotlin.math.sqrt

/** How loud the mic is, 0 to 1, for the ring on the main screen: quick to rise, slower to fall. */
class LevelMeter {
    var level = 0f
        private set

    /** Takes one frame of 16-bit little-endian PCM and returns the smoothed level. */
    fun add(pcm: ByteArray): Float {
        val target = levelFor(rms(pcm))
        level = if (target > level) level + (target - level) * RISE else level * (1 - FALL) + target * FALL
        return level
    }

    private companion object {
        const val RISE = 0.6f
        const val FALL = 0.2f
    }
}

/** Maps a frame's loudness onto the ring: -60 dBFS and below is empty, -12 dBFS and above is full. */
internal fun levelFor(rms: Double): Float {
    if (rms <= 0.0) return 0f
    val db = 20 * log10(rms / 32768)
    return ((db + 60) / 48).toFloat().coerceIn(0f, 1f)
}

internal fun rms(pcm: ByteArray): Double {
    val samples = pcm.size / 2
    if (samples == 0) return 0.0
    var sum = 0.0
    for (i in 0 until samples) {
        val s = ((pcm[2 * i + 1].toInt() shl 8) or (pcm[2 * i].toInt() and 0xFF)).toShort().toDouble()
        sum += s * s
    }
    return sqrt(sum / samples)
}
