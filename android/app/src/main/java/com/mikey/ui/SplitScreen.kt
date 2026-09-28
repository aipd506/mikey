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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.mikey.R
import com.mikey.media.Lens
import com.mikey.service.CameraBlock
import com.mikey.service.MikeyState

/**
 * Camera on top, mic on the bottom. Each half is one big tap target. The camera half never shows
 * video: the picture is on the PC. [onStatusLongPress] is only set in debug builds, to type the
 * PC's address for Wi-Fi testing.
 */
@Composable
fun SplitScreen(
    state: MikeyState,
    onMicTap: () -> Unit,
    onCameraTap: () -> Unit,
    onFlip: () -> Unit,
    onStatusLongPress: (() -> Unit)? = null,
) {
    Box(
        Modifier
            .fillMaxSize()
            .background(Palette.bg)
            .safeDrawingPadding(),
    ) {
        Column(Modifier.fillMaxSize()) {
            val camera = state.camera
            Half(
                icon = R.drawable.ic_camera,
                tint = if (camera.on) Palette.camOn else Palette.iconOff,
                description = stringResource(if (camera.on) R.string.cd_camera_on else R.string.cd_camera_off),
                label = when {
                    camera.blocked == CameraBlock.BLUETOOTH -> stringResource(R.string.camera_blocked_bluetooth)
                    camera.blocked == CameraBlock.PC -> stringResource(R.string.camera_blocked_pc)
                    camera.on -> stringResource(if (camera.lens == Lens.FRONT) R.string.camera_front else R.string.camera_back)
                    else -> null
                },
                onTap = onCameraTap,
                modifier = Modifier.weight(1f),
            )
            Box(
                Modifier
                    .fillMaxWidth()
                    .height(1.dp)
                    .background(Palette.divider),
            )
            Half(
                icon = R.drawable.ic_mic,
                tint = if (state.micOn) Palette.micOn else Palette.iconOff,
                description = stringResource(if (state.micOn) R.string.cd_mic_on else R.string.cd_mic_off),
                label = null,
                onTap = onMicTap,
                modifier = Modifier.weight(1f),
            )
        }
        if (state.camera.on) FlipButton(onFlip, Modifier.align(Alignment.TopStart).padding(16.dp))
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
private fun Half(icon: Int, tint: Color, description: String, label: String?, onTap: () -> Unit, modifier: Modifier) {
    Box(
        modifier
            .fillMaxWidth()
            .clickable(onClick = onTap),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Image(painterResource(icon), description, Modifier.size(56.dp), colorFilter = ColorFilter.tint(tint))
            if (label != null) {
                BasicText(
                    label,
                    Modifier.padding(top = 12.dp, start = 32.dp, end = 32.dp),
                    style = TextStyle(color = Palette.textSecondary, fontSize = 12.sp, textAlign = TextAlign.Center),
                )
            }
        }
    }
}

/** Small and dimmed, top-left, only while the camera is on (phone-ux.md). */
@Composable
private fun FlipButton(onFlip: () -> Unit, modifier: Modifier) {
    val description = stringResource(R.string.cd_flip)
    Box(
        modifier
            .size(40.dp)
            .clip(CircleShape)
            .alpha(0.6f)
            .clickable(onClick = onFlip)
            .semantics { contentDescription = description },
        contentAlignment = Alignment.Center,
    ) {
        Image(painterResource(R.drawable.ic_flip), null, Modifier.size(24.dp), colorFilter = ColorFilter.tint(Palette.iconOff))
    }
}
