package com.mikey.media

import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Test
import java.nio.ByteBuffer

class Nv21Test {
    // A 4×2 image: Y is 1..8, the 2×1 chroma row has U = 10, 11 and V = 20, 21.
    private val y = byteArrayOf(1, 2, 3, 4, 5, 6, 7, 8)
    private val expected = byteArrayOf(1, 2, 3, 4, 5, 6, 7, 8, 20, 10, 21, 11)

    @Test
    fun packsPlanarChroma() {
        val out = ByteArray(12)

        yuv420ToNv21(ByteBuffer.wrap(y), ByteBuffer.wrap(byteArrayOf(10, 11)), ByteBuffer.wrap(byteArrayOf(20, 21)), 4, 2, 4, 2, 1, out)

        assertArrayEquals(expected, out)
    }

    @Test
    fun packsInterleavedChromaInOneGo() {
        // Camera2's usual layout: U and V share one buffer, V starting one byte before U.
        val shared = byteArrayOf(20, 10, 21, 11)
        val u = ByteBuffer.wrap(shared, 1, 3).slice()
        val v = ByteBuffer.wrap(shared, 0, 3).slice()
        val out = ByteArray(12)

        yuv420ToNv21(ByteBuffer.wrap(y), u, v, 4, 2, 4, 4, 2, out)

        assertArrayEquals(expected, out)
    }

    @Test
    fun skipsRowPadding() {
        val paddedY = byteArrayOf(1, 2, 3, 4, 0, 0, 5, 6, 7, 8, 0, 0)
        val out = ByteArray(12)

        yuv420ToNv21(ByteBuffer.wrap(paddedY), ByteBuffer.wrap(byteArrayOf(10, 11)), ByteBuffer.wrap(byteArrayOf(20, 21)), 4, 2, 6, 2, 1, out)

        assertArrayEquals(expected, out)
    }

    @Test
    fun cropsTheCenteredSquareStartingOnAnEvenColumn() {
        // 6×2 cropped to 2×2 keeps columns 2 and 3 (an even start keeps chroma pairs aligned): Y 3,4 / 9,10 and the pair 21,11.
        val wide = byteArrayOf(1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 20, 10, 21, 11, 22, 12)
        val out = ByteArray(6)

        val side = cropNv21Square(wide, 6, 2, out)

        assertEquals(2, side)
        assertArrayEquals(byteArrayOf(3, 4, 9, 10, 21, 11), out)
    }
}
