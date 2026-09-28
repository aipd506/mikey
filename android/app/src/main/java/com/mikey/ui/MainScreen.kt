package com.mikey.ui

import androidx.activity.compose.BackHandler
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.mikey.R
import com.mikey.service.MikeyState
import kotlinx.coroutines.delay
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.sin

/** What the main screen can ask for. MainActivity does the work. */
class MainActions(val onMicTap: () -> Unit, val onCameraTap: () -> Unit, val onFlip: () -> Unit, val sheet: SheetActions)

/**
 * Camera on top, mic on the bottom, each half one big tap target (phone-ux.md). The camera half
 * never shows video: the picture is on the PC. The chevron on the split line opens the settings.
 */
@Composable
fun MainScreen(state: MikeyState, micLevel: Float, micDenied: Boolean, prefs: SettingsView, actions: MainActions) {
    var sheetOpen by rememberSaveable { mutableStateOf(false) }
    BackHandler(enabled = sheetOpen) { sheetOpen = false }
    Box(Modifier.fillMaxSize().background(Palette.bg)) {
        Column(Modifier.fillMaxSize()) {
            CameraHalf(state, actions.onCameraTap, Modifier.weight(1f))
            Hairline()
            MicHalf(state, micLevel, micDenied, actions.onMicTap, Modifier.weight(1f))
        }
        RoundButton(Glyphs.chevronUp, 36.dp, 18.dp, stringResource(R.string.cd_settings), Modifier.align(Alignment.Center)) { sheetOpen = true }
        if (state.camera.on && cameraBlock(state) == null) {
            RoundButton(
                Glyphs.flip,
                40.dp,
                20.dp,
                stringResource(R.string.cd_flip),
                Modifier.align(Alignment.TopStart).windowInsetsPadding(WindowInsets.statusBars).padding(start = 12.dp, top = 8.dp),
                actions.onFlip,
            )
        }
        StatusDot(state.link, Modifier.align(Alignment.BottomEnd).windowInsetsPadding(WindowInsets.navigationBars).padding(16.dp))
        Sheet(sheetOpen, onClose = { sheetOpen = false }) {
            SettingsSheet(state, prefs, actions.sheet, onClose = { sheetOpen = false })
        }
    }
}

@Composable
private fun CameraHalf(state: MikeyState, onTap: () -> Unit, modifier: Modifier) {
    val view = cameraView(state)
    val block = cameraBlock(state)
    var hint by remember { mutableStateOf(false) }
    LaunchedEffect(hint) {
        if (hint) {
            delay(HINT_MS)
            hint = false
        }
    }
    Box(
        modifier
            .fillMaxWidth()
            .drawBehind { dotGrid() }
            // An unavailable camera that's off explains itself instead of turning on; one that's on can be turned off.
            .clickable(interactionSource = null, indication = null) { if (block != null && !state.camera.on) hint = true else onTap() }
            .windowInsetsPadding(WindowInsets.statusBars),
        contentAlignment = Alignment.Center,
    ) {
        Column(Modifier.padding(horizontal = 40.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(20.dp)) {
            StateCircle(view.look, if (view.look == Look.BLOCKED) Glyphs.cameraSlash else Glyphs.camera)
            StateLabel(view)
            val reason = block?.takeIf { hint || state.camera.on }
            if (reason != null) BasicText(stringResource(blockReason(reason)), Modifier.widthIn(max = 300.dp), style = Type.line)
        }
    }
}

@Composable
private fun MicHalf(state: MikeyState, level: Float, denied: Boolean, onTap: () -> Unit, modifier: Modifier) {
    val view = micView(state, denied)
    Box(
        modifier
            .fillMaxWidth()
            .clickable(interactionSource = null, indication = null, onClick = onTap)
            .windowInsetsPadding(WindowInsets.navigationBars),
        contentAlignment = Alignment.Center,
    ) {
        Column(
            Modifier.padding(start = 40.dp, end = 40.dp, bottom = 24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Box(Modifier.size(184.dp), contentAlignment = Alignment.Center) {
                LevelRing(view.look, level)
                StateCircle(view.look, if (view.look == Look.DENIED || view.look == Look.MUTED) Glyphs.micSlash else Glyphs.mic)
            }
            StateLabel(view)
            view.line?.let {
                BasicText(
                    stringResource(it),
                    Modifier.widthIn(max = 300.dp),
                    style = Type.line.copy(color = if (view.lineStrong) Palette.text else Palette.textSecondary),
                )
            }
        }
    }
}

/** The 112 dp circle: dim outline when off, white outline when on, inverted with the red on-air dot when the PC receives. */
@Composable
private fun StateCircle(look: Look, glyph: ImageVector) {
    val fill = if (look == Look.LIVE || look == Look.MUTED) Palette.text else Color.Transparent
    val (border, icon) = when (look) {
        Look.DIM -> Palette.hairline to Palette.inactive
        Look.READY -> Palette.inactive to Palette.textSecondary
        Look.ARMED -> Palette.text to Palette.text
        Look.LIVE, Look.MUTED -> Palette.text to Palette.bg
        Look.BLOCKED -> Palette.hairline to Palette.inactive
        Look.DENIED -> Palette.inactive to Palette.textSecondary
    }
    val dashed = look == Look.BLOCKED || look == Look.DENIED
    Box(Modifier.size(112.dp)) {
        Box(
            Modifier
                .fillMaxSize()
                .background(fill, CircleShape)
                .drawBehind {
                    val width = 1.5.dp.toPx()
                    val dash = if (dashed) PathEffect.dashPathEffect(floatArrayOf(4.dp.toPx(), 3.dp.toPx())) else null
                    drawCircle(border, size.minDimension / 2 - width / 2, style = Stroke(width, pathEffect = dash))
                },
            contentAlignment = Alignment.Center,
        ) {
            Glyph(glyph, icon, 56.dp)
        }
        if (look == Look.LIVE) {
            Box(Modifier.offset(89.dp, 9.dp).size(14.dp).background(Palette.bg, CircleShape).padding(3.dp).background(Palette.live, CircleShape))
        }
    }
}

/** The label under a circle, in capitals. TalkBack reads it in sentence case. */
@Composable
private fun StateLabel(view: HalfView) {
    val text = view.label.map { stringResource(it) }.joinToString(" · ")
    val strong = view.look == Look.LIVE || view.look == Look.ARMED || view.look == Look.MUTED
    BasicText(
        text.uppercase(),
        Modifier.semantics { contentDescription = text },
        style = Type.label.copy(color = if (strong) Palette.text else Palette.textSecondary, textAlign = TextAlign.Center),
    )
}

/** 48 dots around the mic. Live, they fill from the bottom up with the voice; otherwise they show the state. */
@Composable
private fun LevelRing(look: Look, level: Float) {
    val lit = if (look == Look.LIVE) litDots(level) else -1
    val base = when (look) {
        Look.LIVE, Look.MUTED -> Palette.hairline
        Look.ARMED -> Palette.inactive
        else -> Palette.tile
    }
    Canvas(Modifier.fillMaxSize()) {
        val center = size.minDimension / 2
        val radius = center * 84f / 92f
        val dot = 2.5.dp.toPx()
        for (i in 0 until RING_DOTS) {
            val angle = PI / 2 + i * 2 * PI / RING_DOTS
            val k = min(i, RING_DOTS - i)
            val color = when {
                lit < 0 -> base
                k < lit -> Palette.text
                k == lit -> Palette.textSecondary
                else -> base
            }
            drawCircle(color, dot, Offset(center + radius * cos(angle).toFloat(), center + radius * sin(angle).toFloat()))
        }
    }
}

@Composable
private fun RoundButton(glyph: ImageVector, circle: Dp, icon: Dp, description: String, modifier: Modifier, onClick: () -> Unit) {
    Box(
        modifier
            .size(48.dp)
            .clip(CircleShape)
            .clickable(onClick = onClick)
            .semantics {
                contentDescription = description
                role = Role.Button
            },
        contentAlignment = Alignment.Center,
    ) {
        Box(Modifier.size(circle).background(Palette.bg, CircleShape).border(1.dp, Palette.hairline, CircleShape), contentAlignment = Alignment.Center) {
            Glyph(glyph, Palette.text, icon)
        }
    }
}

/** The settings sheet: 60% of the screen, over a dark scrim. Closes on the scrim, the handle or Back. */
@Composable
private fun Sheet(open: Boolean, onClose: () -> Unit, content: @Composable () -> Unit) {
    val scrim by animateFloatAsState(if (open) 1f else 0f, tween(140, easing = LinearEasing), label = "scrim")
    val hidden by animateFloatAsState(if (open) 0f else 1f, tween(180, easing = Standard), label = "sheet")
    if (scrim > 0f) {
        Box(Modifier.fillMaxSize().graphicsLayer { alpha = scrim }.background(Palette.scrim).clickable(interactionSource = null, indication = null, onClick = onClose))
    }
    if (open || hidden < 1f) {
        Box(Modifier.fillMaxSize(), contentAlignment = Alignment.BottomCenter) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .fillMaxHeight(0.6f)
                    .graphicsLayer { translationY = size.height * hidden }
                    .clip(RoundedCornerShape(topStart = 28.dp, topEnd = 28.dp))
                    .background(Palette.sheet)
                    // Taps on the sheet itself mustn't reach the scrim underneath.
                    .pointerInput(Unit) { detectTapGestures { } }
                    .imePadding(),
            ) { content() }
        }
    }
}

/** The faint dot texture behind the camera half. */
private fun androidx.compose.ui.graphics.drawscope.DrawScope.dotGrid() {
    val step = 16.dp.toPx()
    val r = 1.dp.toPx()
    var y = step / 2
    while (y < size.height) {
        var x = step / 2
        while (x < size.width) {
            drawCircle(Palette.grid, r, Offset(x, y))
            x += step
        }
        y += step
    }
}

private const val RING_DOTS = 48
private const val HINT_MS = 4_000L
