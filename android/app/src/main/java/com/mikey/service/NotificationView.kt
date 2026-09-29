package com.mikey.service

import androidx.annotation.StringRes
import com.mikey.R
import com.mikey.ui.cameraBlock
import com.mikey.ui.shortLevelName

/** The glyph in the status bar and the notification's circle. */
internal enum class NotificationIcon { MIC, CAMERA, MIC_OFF }

/** The buttons, in the order they show: the mic's, the camera's, then Stop. */
internal enum class NotificationAction { MUTE, UNMUTE, MIC_ON, CAMERA_OFF, CAMERA_ON, STOP }

/**
 * What the notification says and offers (phone-ux.md §6.5). [title] may take the PC's name as its
 * one argument; [parts] make the text, joined with " · ". [live] means the PC is receiving right
 * now: the only time the accent is red.
 */
internal data class NotificationView(
    @StringRes val title: Int,
    val parts: List<Int>,
    val icon: NotificationIcon,
    val live: Boolean,
    val actions: List<NotificationAction>,
)

internal fun notificationView(state: MikeyState): NotificationView {
    val micLive = state.micOn && !state.muted
    val cameraLive = state.link is Link.Live && state.camera.on && cameraBlock(state) == null
    val icon = when {
        micLive -> NotificationIcon.MIC
        cameraLive -> NotificationIcon.CAMERA
        state.micOn -> NotificationIcon.MIC_OFF
        else -> NotificationIcon.CAMERA
    }
    val onOff = listOf(micPart(state), if (state.camera.on) R.string.notification_camera_on else R.string.notification_camera_off)
    val stop = listOf(NotificationAction.STOP)
    fun grey(title: Int, parts: List<Int>) = NotificationView(title, parts, icon, live = false, actions = stop)

    return when (val link = state.link) {
        Link.Searching -> grey(if (state.reconnecting) R.string.notification_reconnecting else R.string.notification_looking, onOff)
        Link.Waiting -> grey(R.string.notification_waiting, onOff)
        is Link.Refused -> when (link.reason) {
            "denied" -> grey(R.string.notification_denied, listOf(R.string.notification_try_again))
            "version" -> grey(R.string.notification_update, listOf(R.string.notification_versions))
            else -> grey(R.string.notification_refused, listOf(R.string.notification_try_again))
        }
        is Link.Live -> {
            val level = shortLevelName(link.level)
            // Like a call app: the mic's and the camera's buttons stay put and switch between off and on.
            val actions = buildList {
                add(
                    when {
                        !state.micOn -> NotificationAction.MIC_ON
                        state.muted -> NotificationAction.UNMUTE
                        else -> NotificationAction.MUTE
                    },
                )
                when {
                    state.camera.on -> add(NotificationAction.CAMERA_OFF)
                    cameraBlock(state) == null -> add(NotificationAction.CAMERA_ON)
                }
                add(NotificationAction.STOP)
            }
            when {
                micLive || cameraLive -> {
                    val parts = buildList {
                        if (state.micOn) add(if (state.muted) R.string.notification_mic_muted else R.string.notification_mic)
                        if (cameraLive) add(R.string.notification_camera)
                        add(level)
                    }
                    NotificationView(R.string.notification_live, parts, icon, live = true, actions = actions)
                }
                state.muted -> NotificationView(R.string.notification_muted, listOf(onOff[1], level), icon, live = false, actions = actions)
                else -> NotificationView(R.string.notification_connected, onOff + level, icon, live = false, actions = actions)
            }
        }
    }
}

private fun micPart(state: MikeyState) = when {
    !state.micOn -> R.string.notification_mic_off
    state.muted -> R.string.notification_mic_muted
    else -> R.string.notification_mic_on
}
