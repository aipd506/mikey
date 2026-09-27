package com.mikey.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Owns the mic session. Runs only while the mic is on; swiping the app away from Recents
 * stops it (stopWithTask in the manifest), and onDestroy puts everything back to off.
 */
class MikeyService : Service() {
    private val notifier = Notifier(this)
    private val mainThread = Handler(Looper.getMainLooper())
    private var session: SessionController? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_STOP) {
            stopSelf()
            return START_NOT_STICKY
        }
        val notification = notifier.build(Link.Searching)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            startForeground(Notifier.ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE)
        } else {
            startForeground(Notifier.ID, notification)
        }
        if (session == null) {
            session = SessionController(this) { link -> mainThread.post { onLink(link) } }.also { it.start() }
        }
        mutableState.value = mutableState.value.copy(micOn = true)
        // Not sticky: if Android kills the app, the mic must stay off until the user turns it on again.
        return START_NOT_STICKY
    }

    // Runs on the main thread, so it can't race with onDestroy.
    private fun onLink(link: Link) {
        if (session == null || link == mutableState.value.link) return
        mutableState.value = mutableState.value.copy(link = link)
        notifier.show(link)
    }

    override fun onDestroy() {
        session?.stop()
        session = null
        mainThread.removeCallbacksAndMessages(null)
        mutableState.value = MikeyState()
        super.onDestroy()
    }

    companion object {
        const val ACTION_STOP = "com.mikey.action.STOP"

        private val mutableState = MutableStateFlow(MikeyState())
        val state: StateFlow<MikeyState> = mutableState.asStateFlow()

        /** Call only from the app on screen: Android 14+ refuses to start a mic service from the background. */
        fun micOn(context: Context) {
            context.startForegroundService(Intent(context, MikeyService::class.java))
        }

        fun micOff(context: Context) {
            context.stopService(Intent(context, MikeyService::class.java))
        }
    }
}
