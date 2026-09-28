package com.mikey.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.addPathNodes
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * The icons, drawn on a 24 grid with round strokes (design-language.md). The big mic and camera
 * use a thin 0.9 stroke, everything else 1.8.
 */
object Glyphs {
    private const val MIC = "M9 6a3 3 0 0 1 6 0v5a3 3 0 0 1 -6 0z"
    private const val MIC_BASE = "M5.5 11a6.5 6.5 0 0 0 13 0"
    private const val MIC_STEM = "M12 17.5V21"
    private const val CAMERA = "M5.5 6.5h7a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h-7a3 3 0 0 1 -3 -3v-5a3 3 0 0 1 3 -3z"
    private const val CAMERA_LENS = "M15.5 10.5l5.5-3v9l-5.5-3z"

    val mic = glyph(0.9f, MIC, MIC_BASE, MIC_STEM)
    val micSlash = glyph(0.9f, MIC, MIC_BASE, MIC_STEM, "M4 4l16 16")
    val camera = glyph(0.9f, CAMERA, CAMERA_LENS)
    val cameraSlash = glyph(0.9f, CAMERA, CAMERA_LENS, "M3 3l18 18")
    val flip = glyph(1.8f, "M4.5 11a7.5 7.5 0 0 1 13.5-3.5", "M18.5 3.5v4.2h-4.2", "M19.5 13a7.5 7.5 0 0 1-13.5 3.5", "M5.5 20.5v-4.2h4.2")
    val chevronUp = glyph(1.8f, "M6 15l6-6 6 6")
    val chevronDown = glyph(1.8f, "M6 9l6 6 6-6")
    val next = glyph(1.8f, "M9 6l6 6-6 6")
    val usb = glyph(
        1.8f, "M12 18.5V5.5", "M10 6l2-3 2 3z", "M12 16l-4.5-3v-3", circle(7.5f, 9f, 1.3f), "M12 13.5l4.5-3V8.5", "M15.2 5.8h2.6v2.6h-2.6z",
        filled = circle(12f, 20f, 1.4f),
    )
    val wifi = glyph(1.8f, "M2.5 9a13.5 13.5 0 0 1 19 0", "M5.5 12.2a9 9 0 0 1 13 0", "M8.6 15.4a4.6 4.6 0 0 1 6.8 0", filled = circle(12f, 19f, 1.1f))
    val bluetooth = glyph(1.8f, "M7 7l10 10-5 4.5v-19L17 7 7 17")
}

@Composable
fun Glyph(vector: ImageVector, color: Color, size: Dp, modifier: Modifier = Modifier) {
    Image(rememberVectorPainter(vector), contentDescription = null, modifier.size(size), colorFilter = ColorFilter.tint(color))
}

private fun glyph(stroke: Float, vararg paths: String, filled: String? = null): ImageVector =
    ImageVector.Builder(defaultWidth = 24.dp, defaultHeight = 24.dp, viewportWidth = 24f, viewportHeight = 24f).apply {
        paths.forEach {
            addPath(
                addPathNodes(it),
                stroke = SolidColor(Color.White),
                strokeLineWidth = stroke,
                strokeLineCap = StrokeCap.Round,
                strokeLineJoin = StrokeJoin.Round,
            )
        }
        filled?.let { addPath(addPathNodes(it), fill = SolidColor(Color.White)) }
    }.build()

private fun circle(cx: Float, cy: Float, r: Float) = "M${cx - r} ${cy}a$r $r 0 1 0 ${2 * r} 0a$r $r 0 1 0 ${-2 * r} 0"
