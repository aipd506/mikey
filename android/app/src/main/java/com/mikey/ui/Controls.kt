package com.mikey.ui

import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import kotlin.math.roundToInt

internal val Pill = RoundedCornerShape(percent = 50)

/** Material's standard easing, which the design uses for switches and the sheet. */
internal val Standard = CubicBezierEasing(0.2f, 0f, 0f, 1f)

@Composable
fun SectionLabel(text: String) {
    BasicText(text.uppercase(), Modifier.padding(start = 4.dp, top = 24.dp, bottom = 10.dp), style = Type.label)
}

/** A rounded group of rows. Separate the rows with [Hairline]. */
@Composable
fun Tile(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth().clip(RoundedCornerShape(24.dp)).background(Palette.tile), content = content)
}

@Composable
fun Hairline() {
    Box(Modifier.fillMaxWidth().height(1.dp).background(Palette.hairline))
}

@Composable
fun SwitchRow(
    title: String,
    on: Boolean,
    onToggle: (Boolean) -> Unit,
    enabled: Boolean = true,
    subtitle: String? = null,
    icon: ImageVector? = null,
) {
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = 64.dp)
            .toggleable(value = on, enabled = enabled, role = Role.Switch, onValueChange = onToggle)
            .padding(horizontal = 20.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        if (icon != null) Glyph(icon, Palette.textSecondary, 20.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            BasicText(title, style = Type.body.copy(color = if (enabled) Palette.text else Palette.textSecondary))
            if (subtitle != null) BasicText(subtitle, style = Type.hint)
        }
        Switch(on, enabled)
    }
}

/** On is white with a black knob, off is dark grey, disabled is dimmer still. The knob slides in 100 ms. */
@Composable
private fun Switch(on: Boolean, enabled: Boolean) {
    val shown = on && enabled
    val track = if (!enabled) Palette.tile else if (on) Palette.text else Palette.hairline
    val border = if (enabled && on) Palette.text else Palette.hairline
    val knob = if (!enabled) Palette.inactive else if (on) Palette.bg else Palette.textSecondary
    val x by animateDpAsState(if (shown) 24.dp else 2.dp, tween(100, easing = Standard), label = "knob")
    Box(Modifier.size(52.dp, 32.dp).clip(Pill).background(track).border(1.dp, border, Pill).padding(1.dp)) {
        Box(Modifier.offset { IntOffset(x.roundToPx(), 3.dp.roundToPx()) }.size(24.dp).background(knob, CircleShape))
    }
}

@Composable
fun <T> SegmentRow(title: String, options: List<Pair<T, String>>, selected: T, onSelect: (T) -> Unit) {
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp).padding(start = 20.dp, end = 8.dp, top = 8.dp, bottom = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        BasicText(title, Modifier.weight(1f), style = Type.body)
        Row(Modifier.clip(Pill).background(Palette.sheet).padding(4.dp), horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            options.forEach { (value, label) ->
                val on = value == selected
                Box(
                    Modifier
                        .height(40.dp)
                        .widthIn(min = 52.dp)
                        .clip(Pill)
                        .border(1.dp, if (on) Palette.text else Color.Transparent, Pill)
                        .selectable(selected = on, role = Role.RadioButton) { onSelect(value) }
                        .padding(horizontal = 12.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    BasicText(label, style = Type.segment.copy(color = if (on) Palette.text else Palette.textSecondary))
                }
            }
        }
    }
}

/**
 * A slider drawn as 25 dots, lit up to the thumb. [fraction] snaps to [steps]. [onChange] follows
 * the finger; [onDone] comes when it lifts, which is when the value should be saved and sent.
 * A [hollow] thumb marks "off".
 */
@Composable
fun SliderRow(
    title: String,
    value: String,
    fraction: Float,
    steps: Int,
    scale: List<String>,
    onChange: (Float) -> Unit,
    onDone: () -> Unit,
    enabled: Boolean = true,
    hollow: Boolean = false,
) {
    Column(Modifier.fillMaxWidth().padding(start = 20.dp, end = 20.dp, top = 16.dp, bottom = 10.dp)) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            BasicText(title, Modifier.weight(1f), style = Type.body.copy(color = if (enabled) Palette.text else Palette.textSecondary))
            BasicText(value, style = Type.value)
        }
        DotSlider(fraction, steps, enabled, hollow, onChange, onDone, Modifier.padding(top = 4.dp))
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            scale.forEach { BasicText(it, style = Type.scale) }
        }
    }
}

@Composable
private fun DotSlider(
    fraction: Float,
    steps: Int,
    enabled: Boolean,
    hollow: Boolean,
    onChange: (Float) -> Unit,
    onDone: () -> Unit,
    modifier: Modifier,
) {
    val change by rememberUpdatedState(onChange)
    val done by rememberUpdatedState(onDone)
    Canvas(
        modifier
            .fillMaxWidth()
            .height(40.dp)
            .pointerInput(enabled, steps) {
                if (!enabled) return@pointerInput
                detectTapGestures { change(snap(it.x, size.width.toFloat(), 2.dp.toPx(), steps)).also { done() } }
            }
            .pointerInput(enabled, steps) {
                if (!enabled) return@pointerInput
                detectHorizontalDragGestures(
                    onDragStart = { change(snap(it.x, size.width.toFloat(), 2.dp.toPx(), steps)) },
                    onDragEnd = { done() },
                    onDragCancel = { done() },
                ) { pointer, _ ->
                    pointer.consume()
                    change(snap(pointer.position.x, size.width.toFloat(), 2.dp.toPx(), steps))
                }
            },
    ) {
        val r = 2.dp.toPx()
        val track = size.width - 2 * r
        val y = size.height / 2
        for (i in 0..24) {
            val lit = enabled && fraction > 0f && i / 24f <= fraction + 1e-6f
            val color = if (!enabled) Palette.hairline else if (lit) Palette.text else Palette.inactive
            drawCircle(color, r, Offset(r + track * i / 24f, y))
        }
        val thumb = Offset(r + track * fraction, y)
        val thumbR = 10.dp.toPx()
        when {
            !enabled -> drawCircle(Palette.inactive, thumbR, thumb)
            hollow -> {
                drawCircle(Palette.tile, thumbR, thumb)
                drawCircle(Palette.text, thumbR - 0.75.dp.toPx(), thumb, style = Stroke(1.5.dp.toPx()))
            }
            else -> drawCircle(Palette.text, thumbR, thumb)
        }
    }
}

private fun snap(x: Float, width: Float, r: Float, steps: Int): Float {
    val f = ((x - r) / (width - 2 * r)).coerceIn(0f, 1f)
    return (f * steps).roundToInt() / steps.toFloat()
}

/** A row that opens something: title, then a chevron. */
@Composable
fun LinkRow(title: String, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().heightIn(min = 64.dp).clickable(onClick = onClick).padding(horizontal = 20.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(title, Modifier.weight(1f), style = Type.body)
        Glyph(Glyphs.next, Palette.textSecondary, 20.dp)
    }
}
