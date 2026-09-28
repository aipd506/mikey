package com.mikey.media

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class VideoProfileTest {

    @Test
    fun autoQualityIs1080pOnUsbAnd720pElsewhere() {
        assertEquals(1920 to 1080, captureProfileFor(Lens.BACK, Aspect.WIDE, Quality.AUTO, Fps.AUTO, level = 1).size())
        assertEquals(1920 to 1080, captureProfileFor(Lens.BACK, Aspect.WIDE, Quality.AUTO, Fps.AUTO, level = 2).size())
        assertEquals(1280 to 720, captureProfileFor(Lens.BACK, Aspect.WIDE, Quality.AUTO, Fps.AUTO, level = 3).size())
        assertEquals(1280 to 720, captureProfileFor(Lens.BACK, Aspect.WIDE, Quality.P720, Fps.AUTO, level = 1).size())
        assertEquals(1920 to 1080, captureProfileFor(Lens.BACK, Aspect.WIDE, Quality.P1080, Fps.AUTO, level = 3).size())
    }

    @Test
    fun fourByThreeAndSquareCaptureFourByThreeSizes() {
        val standard = captureProfileFor(Lens.FRONT, Aspect.STANDARD, Quality.P720, Fps.F15, level = 3)
        assertEquals(960 to 720, standard.size())
        assertFalse(standard.square)
        assertEquals(15, standard.fps)

        val square = captureProfileFor(Lens.FRONT, Aspect.SQUARE, Quality.P1080, Fps.F30, level = 3)
        assertEquals(1440 to 1080, square.size())
        assertTrue(square.square)
        assertEquals(Lens.FRONT, square.lens)
    }

    @Test
    fun jpegQualityDropsWhenBehindOrHot() {
        assertEquals(75, jpegQualityFor(backlog = false, thermalStatus = 0))
        assertEquals(55, jpegQualityFor(backlog = true, thermalStatus = 0))
        assertEquals(55, jpegQualityFor(backlog = false, thermalStatus = 2))
        assertEquals(40, jpegQualityFor(backlog = false, thermalStatus = 3))
    }

    @Test
    fun wireNamesRoundTripWithSafeDefaults() {
        assertEquals(Lens.FRONT, Lens.fromWire("front"))
        assertEquals(null, Lens.fromWire("flip"))
        assertEquals(Aspect.SQUARE, Aspect.fromWire("1:1"))
        assertEquals(Aspect.WIDE, Aspect.fromWire("odd"))
        assertEquals(Quality.P1080, Quality.fromWire("1080p"))
        assertEquals(Fps.F15, Fps.fromWire("15"))
        assertEquals(Fps.AUTO, Fps.fromWire(null))
    }

    private fun CaptureProfile.size() = width to height
}
