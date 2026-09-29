package com.mikey.ui

import androidx.annotation.StringRes
import com.mikey.R
import com.mikey.media.Lens
import com.mikey.service.CameraBlock
import com.mikey.service.Link
import com.mikey.service.MikeyState
import kotlin.math.roundToInt

/** How a mic or camera circle looks (design-language.md). */
enum class Look {
    /** Off, while the link isn't up: dimmest. */
    DIM,

    /** Off, and a tap starts it. */
    READY,

    /** On, but nothing reaches the PC yet. */
    ARMED,

    /** The PC is receiving: inverted, with the red on-air dot. */
    LIVE,

    /** The mic streams, but muted: the PC hears silence. */
    MUTED,

    /** The camera can't send video over this link. */
    BLOCKED,

    /** No mic permission: a tap asks for it. */
    DENIED,
}

/** What one half shows. The [label] parts are joined with " · " and shown in capitals; a [lineStrong] line is white. */
data class HalfView(
    val look: Look,
    @StringRes val label: List<Int>,
    @StringRes val line: Int? = null,
    val lineStrong: Boolean = false,
)

internal fun micView(state: MikeyState, denied: Boolean): HalfView {
    val link = state.link
    return when {
        denied -> HalfView(Look.DENIED, listOf(R.string.label_mic_no_permission), R.string.line_mic_denied, lineStrong = true)
        state.micOn -> when (link) {
            is Link.Live -> if (state.muted) HalfView(Look.MUTED, listOf(R.string.label_mic_muted)) else HalfView(Look.LIVE, listOf(R.string.label_mic_live))
            Link.Waiting -> HalfView(Look.ARMED, listOf(R.string.label_mic_on, R.string.label_waiting_for_pc), R.string.line_waiting, lineStrong = true)
            is Link.Refused -> HalfView(Look.ARMED, listOf(R.string.label_mic_on, R.string.label_no_pc), refusalLine(link.reason), lineStrong = true)
            Link.Searching -> if (state.reconnecting) {
                HalfView(Look.ARMED, listOf(R.string.label_mic_on, R.string.label_not_reaching_pc), R.string.line_dropped, lineStrong = true)
            } else {
                HalfView(Look.ARMED, listOf(R.string.label_mic_on, R.string.label_no_pc), R.string.line_no_pc)
            }
        }
        running(state) && link !is Link.Live -> HalfView(Look.DIM, listOf(R.string.label_mic_off))
        else -> HalfView(Look.READY, listOf(R.string.label_mic_off, R.string.label_tap_to_start))
    }
}

internal fun cameraView(state: MikeyState): HalfView {
    val camera = state.camera
    val lens = if (camera.lens == Lens.FRONT) R.string.camera_front else R.string.camera_back
    val link = state.link
    return when {
        cameraBlock(state) != null -> HalfView(Look.BLOCKED, listOf(R.string.label_camera_unavailable))
        camera.on -> when (link) {
            is Link.Live -> HalfView(Look.LIVE, listOf(lens))
            Link.Waiting -> HalfView(Look.ARMED, listOf(lens, R.string.label_waiting_for_pc))
            is Link.Refused -> HalfView(Look.ARMED, listOf(lens, R.string.label_no_pc))
            Link.Searching -> HalfView(Look.ARMED, listOf(lens, if (state.reconnecting) R.string.label_not_reaching_pc else R.string.label_no_pc))
        }
        running(state) && link !is Link.Live -> HalfView(Look.DIM, listOf(R.string.label_camera_off))
        else -> HalfView(Look.READY, listOf(R.string.label_camera_off, R.string.label_tap_to_start))
    }
}

/** Why the camera can't send video right now: it's blocked, or the link we're on can't carry it. */
internal fun cameraBlock(state: MikeyState): CameraBlock? {
    state.camera.blocked?.let { return it }
    val link = state.link as? Link.Live ?: return null
    return when {
        link.level == 4 -> CameraBlock.BLUETOOTH
        "vcam" !in link.pcCaps -> CameraBlock.PC
        else -> null
    }
}

@StringRes
internal fun blockReason(block: CameraBlock) = when (block) {
    CameraBlock.BLUETOOTH -> R.string.camera_blocked_bluetooth
    CameraBlock.PC -> R.string.camera_blocked_pc
}

/** Whether the connected PC can do [cap], e.g. `rnnoise`. While not connected it may, so the setting can be changed ahead. */
internal fun pcCan(state: MikeyState, cap: String) = (state.link as? Link.Live)?.let { cap in it.pcCaps } ?: true

/** The sheet's first line. [text] may take the PC's name and [level]'s name. */
data class Header(@StringRes val text: Int, @StringRes val level: Int? = null)

internal fun sheetHeader(state: MikeyState): Header = when (val link = state.link) {
    is Link.Live -> Header(R.string.sheet_connected, levelName(link.level))
    Link.Waiting -> Header(R.string.sheet_waiting)
    is Link.Refused -> Header(R.string.sheet_refused)
    Link.Searching -> when {
        !running(state) -> Header(R.string.sheet_idle)
        state.reconnecting -> Header(R.string.sheet_reconnecting)
        else -> Header(R.string.sheet_searching)
    }
}

/** "USB" for both USB levels, for "Last connected over USB". */
@StringRes
internal fun shortLevelName(level: Int): Int = when (level) {
    1, 2 -> R.string.notification_link_usb
    4 -> R.string.notification_link_bluetooth
    else -> R.string.notification_link_wifi
}

@StringRes
internal fun levelName(level: Int): Int = when (level) {
    1 -> R.string.level_usb_debugging
    2 -> R.string.level_usb_tethering
    4 -> R.string.level_bluetooth
    else -> R.string.level_wifi
}

/** Dots lit on each side of the 48-dot ring, from the bottom up, for a level of 0 to 1. */
internal fun litDots(level: Float) = (level.coerceIn(0f, 1f) * 24.99f).roundToInt()

private fun running(state: MikeyState) = state.micOn || state.camera.on

@StringRes
private fun refusalLine(reason: String) = when (reason) {
    "denied" -> R.string.line_refused_denied
    "version" -> R.string.line_refused_version
    else -> R.string.line_refused
}
