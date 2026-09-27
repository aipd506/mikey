package com.mikey.media

import android.view.Surface
import org.junit.Assert.assertEquals
import org.junit.Test

class OrientationTest {

    @Test
    fun quadrantsMapToScreenRotations() {
        assertEquals(Surface.ROTATION_0, surfaceRotationFor(10, Surface.ROTATION_0))
        assertEquals(Surface.ROTATION_270, surfaceRotationFor(90, Surface.ROTATION_0))
        assertEquals(Surface.ROTATION_180, surfaceRotationFor(180, Surface.ROTATION_0))
        assertEquals(Surface.ROTATION_90, surfaceRotationFor(270, Surface.ROTATION_0))
    }

    @Test
    fun aSmallTiltPastTheHalfwayPointDoesNotFlip() {
        // 50° is just past the 45° halfway point from upright: stay, so a diagonal hold doesn't flicker.
        assertEquals(Surface.ROTATION_0, surfaceRotationFor(50, Surface.ROTATION_0))
        assertEquals(Surface.ROTATION_270, surfaceRotationFor(70, Surface.ROTATION_0))
        // And from the other side, going back needs the same margin.
        assertEquals(Surface.ROTATION_270, surfaceRotationFor(40, Surface.ROTATION_270))
        assertEquals(Surface.ROTATION_0, surfaceRotationFor(20, Surface.ROTATION_270))
    }
}
