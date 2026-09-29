package com.mikey.ui

import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.AnchoredDraggableDefaults
import androidx.compose.foundation.gestures.AnchoredDraggableState
import androidx.compose.foundation.gestures.DraggableAnchors
import androidx.compose.foundation.gestures.FlingBehavior
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.ScrollScope
import androidx.compose.foundation.gestures.anchoredDraggable
import androidx.compose.foundation.gestures.animateTo
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.nestedscroll.NestedScrollConnection
import androidx.compose.ui.input.nestedscroll.NestedScrollSource
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.collapse
import androidx.compose.ui.semantics.dismiss
import androidx.compose.ui.semantics.expand
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Velocity
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.flow.drop
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch

/** Where the sheet can rest, like Material's bottom sheet: gone, 60% of the screen, or full height. */
private enum class SheetStop { Hidden, Half, Full }

private const val HALF = 0.6f
private val Slide = tween<Float>(180, easing = Standard)

/**
 * The settings sheet over a dark scrim. It opens at 60% of the screen; dragging it or scrolling
 * its content up takes it to full height, and down goes back to 60% and then closes. It also
 * closes on the scrim, the handle or Back.
 */
@Composable
fun Sheet(open: Boolean, onClose: () -> Unit, content: @Composable () -> Unit) {
    val state = remember { AnchoredDraggableState(SheetStop.Hidden) }
    val fling = AnchoredDraggableDefaults.flingBehavior(state, animationSpec = Slide)
    val scope = rememberCoroutineScope()
    val close by rememberUpdatedState(onClose)

    LaunchedEffect(open) {
        snapshotFlow { state.anchors.size }.first { it > 0 }
        state.animateTo(if (open) SheetStop.Half else SheetStop.Hidden, Slide)
    }
    // Dragged all the way down: that's a close too. The first value is where it starts, not a drag.
    LaunchedEffect(state) {
        snapshotFlow { state.settledValue }.drop(1).collect { if (it == SheetStop.Hidden) close() }
    }

    BoxWithConstraints(Modifier.fillMaxSize(), contentAlignment = Alignment.BottomCenter) {
        val screen = constraints.maxHeight.toFloat()
        val sheetHeight = screen - WindowInsets.statusBars.getTop(LocalDensity.current)
        SideEffect {
            state.updateAnchors(
                DraggableAnchors {
                    SheetStop.Hidden at sheetHeight
                    SheetStop.Half at sheetHeight - screen * HALF
                    SheetStop.Full at 0f
                },
            )
        }
        // The position is read only while drawing, so a drag doesn't recompose the sheet every frame.
        val offset = { state.offset.takeUnless { it.isNaN() } ?: sheetHeight }
        val shown by remember(sheetHeight) { derivedStateOf { offset() < sheetHeight } }
        if (shown) {
            Box(
                Modifier
                    .fillMaxSize()
                    .graphicsLayer { alpha = ((sheetHeight - offset()) / (screen * HALF)).coerceIn(0f, 1f) }
                    .background(Palette.scrim)
                    .clickable(interactionSource = null, indication = null, onClick = onClose),
            )
        }
        if (open || shown) {
            Box(
                Modifier
                    .fillMaxWidth()
                    .height(with(LocalDensity.current) { sheetHeight.toDp() })
                    .graphicsLayer { translationY = offset() }
                    .semantics {
                        if (state.currentValue == SheetStop.Half) expand { scope.launch { state.animateTo(SheetStop.Full, Slide) }; true }
                        if (state.currentValue == SheetStop.Full) collapse { scope.launch { state.animateTo(SheetStop.Half, Slide) }; true }
                        dismiss { onClose(); true }
                    }
                    .nestedScroll(remember(state, fling) { SheetScroll(state, fling) })
                    .anchoredDraggable(state, reverseDirection = false, orientation = Orientation.Vertical, flingBehavior = fling)
                    .clip(RoundedCornerShape(topStart = 28.dp, topEnd = 28.dp))
                    .background(Palette.sheet)
                    // Taps on the sheet itself mustn't reach the scrim underneath.
                    .pointerInput(Unit) { detectTapGestures { } }
                    .imePadding(),
            ) { content() }
        }
    }
}

/**
 * Scrolling the content moves the sheet first when it should, as in Material: up grows it to
 * full height before the content scrolls, and down, once the content is back at its top, lowers it.
 */
private class SheetScroll(private val state: AnchoredDraggableState<SheetStop>, private val fling: FlingBehavior) : NestedScrollConnection {
    override fun onPreScroll(available: Offset, source: NestedScrollSource): Offset =
        if (available.y < 0 && source == NestedScrollSource.UserInput) Offset(0f, state.dispatchRawDelta(available.y)) else Offset.Zero

    override fun onPostScroll(consumed: Offset, available: Offset, source: NestedScrollSource): Offset =
        if (source == NestedScrollSource.UserInput) Offset(0f, state.dispatchRawDelta(available.y)) else Offset.Zero

    override suspend fun onPreFling(available: Velocity): Velocity =
        if (available.y < 0 && state.requireOffset() > state.anchors.minPosition()) {
            settle(available.y)
            available
        } else {
            Velocity.Zero
        }

    override suspend fun onPostFling(consumed: Velocity, available: Velocity): Velocity {
        settle(available.y)
        return available
    }

    /** Lets go at [velocity] and comes to rest on a stop, the same way the sheet's own drag does. */
    private suspend fun settle(velocity: Float) {
        state.anchoredDrag {
            val scroll = object : ScrollScope {
                override fun scrollBy(pixels: Float): Float {
                    val from = state.requireOffset()
                    val to = (from + pixels).coerceIn(state.anchors.minPosition(), state.anchors.maxPosition())
                    dragTo(to)
                    return to - from
                }
            }
            with(fling) { scroll.performFling(velocity) }
        }
    }
}
