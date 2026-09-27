package com.mikey.ui

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.mikey.R
import com.mikey.service.MikeyState

/**
 * Camera on top, mic on the bottom. Each half is one big tap target.
 * [onStatusLongPress] is only set in debug builds, to type the PC's address for Wi-Fi testing.
 */
@Composable
fun SplitScreen(state: MikeyState, onMicTap: () -> Unit, onStatusLongPress: (() -> Unit)? = null) {
    Box(
        Modifier
            .fillMaxSize()
            .background(Palette.bg)
            .safeDrawingPadding(),
    ) {
        Column(Modifier.fillMaxSize()) {
            // The camera arrives in Phase 3; until then its half is shown but does nothing.
            Half(
                icon = painterResource(R.drawable.ic_camera),
                tint = Palette.iconOff,
                description = stringResource(R.string.cd_camera),
                onTap = null,
                modifier = Modifier.weight(1f),
            )
            Box(
                Modifier
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(Palette.divider),
            )
            Half(
                icon = painterResource(R.drawable.ic_mic),
                tint = if (state.micOn) Palette.micOn else Palette.iconOff,
                description = stringResource(if (state.micOn) R.string.cd_mic_on else R.string.cd_mic_off),
                onTap = onMicTap,
                modifier = Modifier.weight(1f),
            )
        }
        StatusDot(
            link = state.link,
            modifier = Modifier
                .align(Alignment.BottomEnd)
                // Before the padding, so the long-press area is bigger than the 10 dp dot.
                .then(
                    if (onStatusLongPress != null) {
                        Modifier.pointerInput(Unit) { detectTapGestures(onLongPress = { onStatusLongPress() }) }
                    } else {
                        Modifier
                    },
                )
                .padding(16.dp),
        )
    }
}

@Composable
private fun Half(icon: Painter, tint: Color, description: String, onTap: (() -> Unit)?, modifier: Modifier) {
    Box(
        modifier
            .fillMaxWidth()
            .then(if (onTap != null) Modifier.clickable(onClick = onTap) else Modifier),
        contentAlignment = Alignment.Center,
    ) {
        Image(icon, description, Modifier.size(56.dp), colorFilter = ColorFilter.tint(tint))
    }
}
