package com.mikey.media

import java.nio.ByteBuffer

/**
 * Packs a YUV_420_888 image into NV21 (all of Y, then V and U interleaved), the layout
 * YuvImage compresses to JPEG. The chroma planes may be planar (pixel stride 1) or already
 * interleaved (pixel stride 2); the second case is copied in one go.
 */
fun yuv420ToNv21(
    y: ByteBuffer,
    u: ByteBuffer,
    v: ByteBuffer,
    width: Int,
    height: Int,
    yRowStride: Int,
    uvRowStride: Int,
    uvPixelStride: Int,
    out: ByteArray,
) {
    var pos = 0
    if (yRowStride == width) {
        y.position(0)
        y.get(out, 0, width * height)
        pos = width * height
    } else {
        for (row in 0 until height) {
            y.position(row * yRowStride)
            y.get(out, pos, width)
            pos += width
        }
    }
    val chromaWidth = width / 2
    val chromaHeight = height / 2
    if (uvPixelStride == 2 && uvRowStride == width) {
        // V's buffer already reads V U V U ..., only the very last U is missing from it.
        val chroma = width * chromaHeight
        v.position(0)
        val fromV = minOf(v.remaining(), chroma)
        v.get(out, pos, fromV)
        if (fromV < chroma) out[pos + chroma - 1] = u.get(u.limit() - 1)
        return
    }
    for (row in 0 until chromaHeight) {
        val rowStart = row * uvRowStride
        for (col in 0 until chromaWidth) {
            val at = rowStart + col * uvPixelStride
            out[pos++] = v.get(at)
            out[pos++] = u.get(at)
        }
    }
}

/** Crops an NV21 image to the largest centered square. Returns its side. Even offsets keep the chroma pairs aligned. */
fun cropNv21Square(src: ByteArray, width: Int, height: Int, out: ByteArray): Int {
    val size = minOf(width, height)
    val x0 = ((width - size) / 2) and 1.inv()
    val y0 = ((height - size) / 2) and 1.inv()
    for (row in 0 until size) System.arraycopy(src, (y0 + row) * width + x0, out, row * size, size)
    val srcChroma = width * height
    val outChroma = size * size
    for (row in 0 until size / 2) System.arraycopy(src, srcChroma + (y0 / 2 + row) * width + x0, out, outChroma + row * size, size)
    return size
}
