package com.mikey.service

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.drawable.Icon
import android.os.Build
import com.mikey.MainActivity
import com.mikey.R
import com.mikey.settings.Settings
import kotlin.math.roundToInt

/**
 * The notification Android requires while MikeyService runs, and the remote control while the app
 * is closed: what's live and where, with the mic's and the camera's buttons and Stop (phone-ux.md §6.5).
 */
class Notifier(private val service: Service) {

    fun build(state: MikeyState): Notification {
        val manager = service.getSystemService(NotificationManager::class.java)
        // Not the old low-importance channel: that one sits collapsed under "Silent", hiding the buttons.
        manager.deleteNotificationChannel(OLD_CHANNEL_ID)
        manager.createNotificationChannel(
            NotificationChannel(CHANNEL_ID, service.getString(R.string.notification_channel), NotificationManager.IMPORTANCE_DEFAULT).apply {
                setSound(null, null)
                enableVibration(false)
                setShowBadge(false)
            },
        )
        val view = notificationView(state)
        val pc = Settings(service).pairedPc?.name?.ifEmpty { null } ?: service.getString(R.string.the_pc)
        val glyph = when (view.icon) {
            NotificationIcon.MIC -> R.drawable.ic_notification_mic
            NotificationIcon.CAMERA -> R.drawable.ic_notification_camera
            NotificationIcon.MIC_OFF -> R.drawable.ic_notification_mic_off
        }
        val builder = Notification.Builder(service, CHANNEL_ID)
            .setSmallIcon(glyph)
            .setColor(if (view.live) LIVE else GREY)
            .setLargeIcon(stateIcon(glyph, view.live))
            .setContentTitle(service.getString(view.title, pc))
            .setContentText(view.parts.joinToString(" · ") { service.getString(it) })
            .setContentIntent(openApp())
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            // Seeing that the phone is live, and muting it, shouldn't need unlocking.
            .setVisibility(Notification.VISIBILITY_PUBLIC)
        view.actions.forEach { builder.addAction(action(it)) }
        return builder.build()
    }

    /** Replaces the notification's text. Shows nothing if the user turned notifications off. */
    fun show(state: MikeyState) {
        service.getSystemService(NotificationManager::class.java).notify(ID, build(state))
    }

    private fun action(action: NotificationAction): Notification.Action {
        val (label, intentAction) = when (action) {
            NotificationAction.MUTE -> R.string.notification_mute to MikeyService.ACTION_MUTE
            NotificationAction.UNMUTE -> R.string.notification_unmute to MikeyService.ACTION_UNMUTE
            NotificationAction.MIC_ON -> R.string.notification_turn_mic_on to MikeyService.ACTION_MIC_ON
            NotificationAction.CAMERA_OFF -> R.string.notification_turn_camera_off to MikeyService.ACTION_CAMERA_OFF
            NotificationAction.CAMERA_ON -> R.string.notification_turn_camera_on to MikeyService.ACTION_CAMERA_ON
            NotificationAction.STOP -> R.string.notification_stop to MikeyService.ACTION_STOP
        }
        val turnsOn = action == NotificationAction.MIC_ON || action == NotificationAction.CAMERA_ON
        val permission = if (action == NotificationAction.MIC_ON) Manifest.permission.RECORD_AUDIO else Manifest.permission.CAMERA
        // Android only asks for a permission from the app, so without it the button opens the app.
        val intent = if (turnsOn && service.checkSelfPermission(permission) != PackageManager.PERMISSION_GRANTED) {
            openApp()
        } else {
            PendingIntent.getService(
                service,
                action.ordinal + 1,
                Intent(service, MikeyService::class.java).setAction(intentAction),
                PendingIntent.FLAG_IMMUTABLE,
            )
        }
        val builder = Notification.Action.Builder(null, service.getString(label), intent)
        // Turning the mic or camera on from the lock screen asks for the unlock first; muting and stopping don't.
        if (turnsOn && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) builder.setAuthenticationRequired(true)
        return builder.build()
    }

    /**
     * The state again on the right, as in the app: the glyph on black, white with the red on-air
     * dot while the PC is receiving, grey otherwise. Newer Android and Samsung put the app's own
     * icon in the left circle, so this is where they show red.
     */
    private fun stateIcon(glyph: Int, live: Boolean): Icon {
        val dp = service.resources.displayMetrics.density
        val bitmap = Bitmap.createBitmap((44 * dp).roundToInt(), (44 * dp).roundToInt(), Bitmap.Config.ARGB_8888)
        val canvas = Canvas(bitmap)
        val paint = Paint(Paint.ANTI_ALIAS_FLAG)
        paint.color = Color.BLACK
        canvas.drawCircle(22 * dp, 22 * dp, 20 * dp, paint)
        paint.style = Paint.Style.STROKE
        paint.strokeWidth = dp
        paint.color = INACTIVE
        canvas.drawCircle(22 * dp, 22 * dp, 19.5f * dp, paint)
        service.getDrawable(glyph)?.mutate()?.run {
            setTint(if (live) Color.WHITE else GREY)
            setBounds((12 * dp).roundToInt(), (12 * dp).roundToInt(), (32 * dp).roundToInt(), (32 * dp).roundToInt())
            draw(canvas)
        }
        if (live) {
            paint.style = Paint.Style.FILL
            paint.color = Color.BLACK
            canvas.drawCircle(38 * dp, 6 * dp, 6 * dp, paint)
            paint.color = LIVE
            canvas.drawCircle(38 * dp, 6 * dp, 4 * dp, paint)
        }
        return Icon.createWithBitmap(bitmap)
    }

    private fun openApp() = PendingIntent.getActivity(service, 0, Intent(service, MainActivity::class.java), PendingIntent.FLAG_IMMUTABLE)

    companion object {
        const val ID = 1
        private const val CHANNEL_ID = "mikey_live"
        private const val OLD_CHANNEL_ID = "mikey"
        private const val LIVE = 0xFFD71921.toInt()
        private const val GREY = 0xFF8E8E93.toInt()
        private const val INACTIVE = 0xFF3A3A3C.toInt()
    }
}
