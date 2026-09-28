package com.mikey.service

import com.mikey.media.OpusEncoder
import org.junit.Assert.assertEquals
import org.junit.Test
import java.util.concurrent.ArrayBlockingQueue

class SessionControllerTest {

    @Test
    fun reconnectWaitsDoubleFromHalfASecondAndStopAtFive() {
        val waits = listOf(0, 1, 2, 3, 4, 5, 100).map(::reconnectDelayMs)

        assertEquals(listOf(500L, 1_000L, 2_000L, 4_000L, 5_000L, 5_000L, 5_000L), waits)
    }

    @Test
    fun rejectPolicyRetriesOnlyWhatCanClearUpByItself() {
        assertEquals(Reaction.RETRY, rejectPolicy("timeout"))
        assertEquals(Reaction.RETRY, rejectPolicy("busy"))
        assertEquals(Reaction.FORGET_AND_RETRY, rejectPolicy("bad_token"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("denied"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("version"))
        assertEquals(Reaction.GIVE_UP, rejectPolicy("something-new"))
    }

    @Test
    fun rawPcmOnUsbAndForLosslessWifiOpusOtherwise() {
        assertEquals(AudioCodec.PCM, audioCodecFor(level = 1, losslessWifi = false, pcHasOpus = true))
        assertEquals(AudioCodec.PCM, audioCodecFor(level = 2, losslessWifi = false, pcHasOpus = true))
        assertEquals(AudioCodec.OPUS, audioCodecFor(level = 3, losslessWifi = false, pcHasOpus = true))
        assertEquals(AudioCodec.OPUS, audioCodecFor(level = 4, losslessWifi = false, pcHasOpus = true))
        assertEquals(AudioCodec.PCM, audioCodecFor(level = 3, losslessWifi = true, pcHasOpus = true))
        assertEquals(AudioCodec.OPUS, audioCodecFor(level = 4, losslessWifi = true, pcHasOpus = true))
        assertEquals(AudioCodec.PCM, audioCodecFor(level = 3, losslessWifi = false, pcHasOpus = false))
    }

    @Test
    fun bluetoothGetsSpeechModeAt48kbpsIn20msFramesWifiLowDelay96() {
        val bluetooth = opusProfileFor(4)
        assertEquals(OpusEncoder.Application.VOIP, bluetooth.application)
        assertEquals(48_000, bluetooth.bitrate)
        assertEquals(2, bluetooth.framesPerPacket)

        val wifi = opusProfileFor(3)
        assertEquals(OpusEncoder.Application.LOW_DELAY, wifi.application)
        assertEquals(96_000, wifi.bitrate)
        assertEquals(1, wifi.framesPerPacket)
    }

    @Test
    fun fullQueueDropsTheOldestItem() {
        val queue = ArrayBlockingQueue<Int>(2)

        listOf(1, 2, 3).forEach { queue.offerDroppingOldest(it) }

        assertEquals(listOf(2, 3), queue.toList())
    }
}
