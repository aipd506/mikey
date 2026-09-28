package com.mikey.ui

import com.mikey.R
import com.mikey.media.Lens
import com.mikey.service.CameraBlock
import com.mikey.service.CameraState
import com.mikey.service.Link
import com.mikey.service.MikeyState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ScreenModelTest {
    private val usb = Link.Live(1, setOf("pcm", "opus", "vcam", "aec", "rnnoise"))

    @Test
    fun theMicIsInvitingWhenIdleAndInvertedOnlyWhenThePcReceives() {
        assertEquals(Look.READY, micView(MikeyState(), denied = false).look)
        assertEquals(Look.LIVE, micView(MikeyState(micOn = true, link = usb), denied = false).look)
        assertEquals(Look.MUTED, micView(MikeyState(micOn = true, link = usb, muted = true), denied = false).look)
        assertEquals(Look.DENIED, micView(MikeyState(), denied = true).look)
    }

    @Test
    fun aMicThatIsOnButNotReachingThePcSaysWhy() {
        val first = micView(MikeyState(micOn = true), denied = false)
        assertEquals(listOf(R.string.label_mic_on, R.string.label_no_pc), first.label)
        assertFalse(first.lineStrong)

        val dropped = micView(MikeyState(micOn = true, reconnecting = true), denied = false)
        assertEquals(R.string.line_dropped, dropped.line)
        assertTrue(dropped.lineStrong)

        assertEquals(R.string.line_waiting, micView(MikeyState(micOn = true, link = Link.Waiting), denied = false).line)
        assertEquals(R.string.line_refused_version, micView(MikeyState(micOn = true, link = Link.Refused("version")), denied = false).line)
    }

    @Test
    fun theCameraNamesItsLensAndIsUnavailableWhereVideoCantGo() {
        val back = cameraView(MikeyState(camera = CameraState(on = true), link = usb))
        assertEquals(Look.LIVE, back.look)
        assertEquals(listOf(R.string.camera_back), back.label)

        val front = cameraView(MikeyState(camera = CameraState(on = true, lens = Lens.FRONT)))
        assertEquals(listOf(R.string.camera_front, R.string.label_no_pc), front.label)

        assertEquals(CameraBlock.BLUETOOTH, cameraBlock(MikeyState(link = Link.Live(4))))
        assertEquals(CameraBlock.PC, cameraBlock(MikeyState(link = Link.Live(3, setOf("pcm", "opus")))))
        assertNull(cameraBlock(MikeyState(link = usb)))
        assertEquals(Look.BLOCKED, cameraView(MikeyState(link = Link.Live(4))).look)
    }

    @Test
    fun theOtherHalfIsDimWhileTheLinkIsDown() {
        assertEquals(Look.DIM, micView(MikeyState(camera = CameraState(on = true)), denied = false).look)
        assertEquals(Look.READY, micView(MikeyState(camera = CameraState(on = true), link = usb), denied = false).look)
    }

    @Test
    fun settingsTheConnectedPcCantDoAreGreyedOut() {
        assertTrue(pcCan(MikeyState(), "aec"))
        assertTrue(pcCan(MikeyState(link = usb), "aec"))
        assertFalse(pcCan(MikeyState(link = Link.Live(1, setOf("pcm"))), "rnnoise"))
    }

    @Test
    fun theSheetHeaderFollowsTheLink() {
        assertEquals(Header(R.string.sheet_connected, R.string.level_usb_debugging), sheetHeader(MikeyState(micOn = true, link = usb)))
        assertEquals(Header(R.string.sheet_idle), sheetHeader(MikeyState()))
        assertEquals(Header(R.string.sheet_reconnecting), sheetHeader(MikeyState(micOn = true, reconnecting = true)))
        assertEquals(R.string.level_wifi, levelName(3))
    }

    @Test
    fun theGateSliderRunsFromOffThroughMinus60ToMinus20() {
        assertEquals(0, gateStep(null))
        assertNull(gateDbAt(0))
        assertEquals(1, gateStep(-60f))
        assertEquals(16, gateStep(-45f))
        assertEquals(GATE_STEPS, gateStep(-20f))
        assertEquals(-45f, gateDbAt(16))
        assertEquals(-20f, gateDbAt(GATE_STEPS))
        assertEquals(1, gateStep(-90f)) // Out of range from an old PC: the lowest threshold, not off.
    }

    @Test
    fun theRingLightsFromTheBottomWithTheVoice() {
        assertEquals(0, litDots(0f))
        assertEquals(12, litDots(0.5f))
        assertEquals(25, litDots(1f)) // All 24 on each side, top included.
    }
}
