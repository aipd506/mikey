package com.mikey.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import com.mikey.MainActivity
import com.mikey.R

/** The notification Android requires while MikeyService runs: "Mic on · Camera off · USB", with Mute and Stop. */
class Notifier(private val service: Service) {

    fun build(state: MikeyState): Notification {
        service.getSystemService(NotificationManager::class.java).createNotificationChannel(
            NotificationChannel(
                CHANNEL_ID,
                service.getString(R.string.notification_channel),
                NotificationManager.IMPORTANCE_LOW,
            ),
        )
        val open = PendingIntent.getActivity(
            service,
            0,
            Intent(service, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE,
        )
        val mic = service.getString(
            when {
                !state.micOn -> R.string.notification_mic_off
                state.muted -> R.string.notification_mic_muted
                else -> R.string.notification_mic_on
            },
        )
        val camera = service.getString(if (state.camera.on) R.string.notification_camera_on else R.string.notification_camera_off)
        val builder = Notification.Builder(service, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_mic)
            .setContentTitle(service.getString(R.string.app_name))
            .setContentText(service.getString(R.string.notification_text, mic, camera, service.getString(linkText(state.link))))
            .setContentIntent(open)
            .setOngoing(true)
        if (state.micOn) {
            builder.addAction(action(if (state.muted) R.string.notification_unmute else R.string.notification_mute, if (state.muted) MikeyService.ACTION_UNMUTE else MikeyService.ACTION_MUTE, 1))
        }
        return builder.addAction(action(R.string.notification_stop, MikeyService.ACTION_STOP, 2)).build()
    }

    /** Replaces the notification's text. Shows nothing if the user turned notifications off. */
    fun show(state: MikeyState) {
        service.getSystemService(NotificationManager::class.java).notify(ID, build(state))
    }

    private fun action(label: Int, action: String, requestCode: Int): Notification.Action {
        val intent = PendingIntent.getService(
            service,
            requestCode,
            Intent(service, MikeyService::class.java).setAction(action),
            PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Action.Builder(null, service.getString(label), intent).build()
    }

    private fun linkText(link: Link) = when (link) {
        Link.Searching -> R.string.notification_link_searching
        Link.Waiting -> R.string.notification_link_waiting
        is Link.Live -> when (link.level) {
            1, 2 -> R.string.notification_link_usb
            4 -> R.string.notification_link_bluetooth
            else -> R.string.notification_link_wifi
        }
        is Link.Refused -> when (link.reason) {
            "denied" -> R.string.notification_link_denied
            "version" -> R.string.notification_link_version
            else -> R.string.notification_link_refused
        }
    }

    companion object {
        const val ID = 1
        private const val CHANNEL_ID = "mikey"
    }
}
