package com.mikey.media

/** Which camera. [wire] is the name in CONTROL frames. */
enum class Lens(val wire: String) {
    BACK("back"),
    FRONT("front"),
    ;

    fun other() = if (this == BACK) FRONT else BACK

    companion object {
        fun fromWire(wire: String?) = entries.firstOrNull { it.wire == wire }
    }
}

enum class Aspect(val wire: String) {
    WIDE("16:9"),
    STANDARD("4:3"),
    SQUARE("1:1"),
    ;

    companion object {
        fun fromWire(wire: String?) = entries.firstOrNull { it.wire == wire } ?: WIDE
    }
}

enum class Quality(val wire: String) {
    AUTO("auto"),
    P720("720p"),
    P1080("1080p"),
    ;

    companion object {
        fun fromWire(wire: String?) = entries.firstOrNull { it.wire == wire } ?: AUTO
    }
}

enum class Fps(val wire: String, val value: Int) {
    AUTO("auto", 30),
    F30("30", 30),
    F15("15", 15),
    ;

    companion object {
        fun fromWire(wire: String?) = entries.firstOrNull { it.wire == wire } ?: AUTO
    }
}

/** What to capture: the lens, the sensor size to ask for (landscape), whether to crop to a square, and the frame rate. */
data class CaptureProfile(val lens: Lens, val width: Int, val height: Int, val square: Boolean, val fps: Int)

/**
 * Resolution from aspect ratio and quality (media-pipeline.md): 16:9 is 1280×720 or 1920×1080,
 * 4:3 is 960×720 or 1440×1080, and 1:1 captures 4:3 and crops. Auto means 1080p on USB, where
 * bandwidth is free, and 720p elsewhere.
 */
fun captureProfileFor(lens: Lens, aspect: Aspect, quality: Quality, fps: Fps, level: Int): CaptureProfile {
    val full = when (quality) {
        Quality.P1080 -> true
        Quality.P720 -> false
        Quality.AUTO -> level in 1..2
    }
    val (width, height) = when (aspect) {
        Aspect.WIDE -> if (full) 1920 to 1080 else 1280 to 720
        Aspect.STANDARD, Aspect.SQUARE -> if (full) 1440 to 1080 else 960 to 720
    }
    return CaptureProfile(lens, width, height, square = aspect == Aspect.SQUARE, fps = fps.value)
}

/**
 * JPEG quality: 75 normally, lower when frames pile up in the send queue or the phone runs hot.
 * [thermalStatus] is PowerManager's: 2 is moderate, 3 severe and up.
 */
fun jpegQualityFor(backlog: Boolean, thermalStatus: Int): Int = when {
    thermalStatus >= 3 -> 40
    backlog || thermalStatus >= 2 -> 55
    else -> 75
}
