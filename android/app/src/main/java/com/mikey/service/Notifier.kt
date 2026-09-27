package com.mikey.service

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import com.mikey.MainActivity
import com.mikey.R

/** The notification Android requires while MikeyService runs. */
class Notifier(private val service: Service) {

    fun build(link: Link): Notification {
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
        val stop = PendingIntent.getService(
            service,
            0,
            Intent(service, MikeyService::class.java).setAction(MikeyService.ACTION_STOP),
            PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Builder(service, CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_mic)
            .setContentTitle(service.getString(R.string.app_name))
            .setContentText(service.getString(textFor(link)))
            .setContentIntent(open)
            .setOngoing(true)
            .addAction(Notification.Action.Builder(null, service.getString(R.string.notification_stop), stop).build())
            .build()
    }

    /** Replaces the notification's text. Shows nothing if the user turned notifications off. */
    fun show(link: Link) {
        service.getSystemService(NotificationManager::class.java).notify(ID, build(link))
    }

    private fun textFor(link: Link) = when (link) {
        Link.Searching -> R.string.notification_mic_on_searching
        Link.Waiting -> R.string.notification_mic_on_waiting
        is Link.Live -> if (link.level == 1) R.string.notification_mic_on_usb else R.string.notification_mic_on_wifi
        is Link.Refused -> when (link.reason) {
            "denied" -> R.string.notification_mic_on_denied
            "version" -> R.string.notification_mic_on_version
            else -> R.string.notification_mic_on_refused
        }
    }

    companion object {
        const val ID = 1
        private const val CHANNEL_ID = "mikey"
    }
}
