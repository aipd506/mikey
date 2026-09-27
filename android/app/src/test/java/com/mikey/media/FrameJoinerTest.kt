package com.mikey.media

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertSame
import org.junit.Test

class FrameJoinerTest {

    @Test
    fun oneFramePerPacketPassesFramesThrough() {
        val frame = AudioFrame(7, 1_000L, byteArrayOf(1, 2))

        assertSame(frame, FrameJoiner(1).add(frame))
    }

    @Test
    fun twoFramesPerPacketJoinsPairsAndKeepsTheFirstFramesSeqAndTime() {
        val joiner = FrameJoiner(2)

        assertNull(joiner.add(AudioFrame(10, 1_000L, byteArrayOf(1, 2))))
        val joined = joiner.add(AudioFrame(11, 1_010L, byteArrayOf(3, 4)))!!

        assertEquals(10, joined.seq)
        assertEquals(1_000L, joined.captureTimeUs)
        assertArrayEquals(byteArrayOf(1, 2, 3, 4), joined.pcm)

        assertNull(joiner.add(AudioFrame(12, 1_020L, byteArrayOf(5, 6))))
        assertArrayEquals(byteArrayOf(5, 6, 7, 8), joiner.add(AudioFrame(13, 1_030L, byteArrayOf(7, 8)))!!.pcm)
    }
}
