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
 * Owns the session with the PC while the mic or the camera is on. Swiping the app away from
 * Recents stops it (stopWithTask in the manifest), and onDestroy puts everything back to off.
 * The foreground service declares the microphone type while the mic is on and the camera type
 * while the camera is on, and nothing more.
 */
class MikeyService : Service(), SessionController.Listener {
    private val notifier = Notifier(this)
    private val mainThread = Handler(Looper.getMainLooper())
    private var session: SessionController? = null

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        val state = mutableState.value
        when (intent?.action) {
            ACTION_STOP -> stopSelf()
            ACTION_MUTE, ACTION_UNMUTE -> session?.setMuted(intent.action == ACTION_MUTE)
            ACTION_FLIP -> session?.flip()
            ACTION_MIC_OFF -> turn(mic = false, camera = state.camera.on)
            ACTION_CAMERA_OFF -> turn(mic = state.micOn, camera = false)
            ACTION_MIC_ON -> turn(mic = true, camera = state.camera.on)
            ACTION_CAMERA_ON -> turn(mic = state.micOn, camera = true)
        }
        // Not sticky: if Android kills the app, the mic and camera must stay off until the user turns them on again.
        return START_NOT_STICKY
    }

    /** Applies what should be on. With nothing on, the service ends. */
    private fun turn(mic: Boolean, camera: Boolean) {
        if (!mic && !camera) {
            stopSelf()
            return
        }
        mutableState.value = mutableState.value.let { it.copy(micOn = mic, camera = it.camera.copy(on = camera)) }
        foreground()
        val session = session ?: SessionController(this, this).also {
            session = it
            it.start()
        }
        session.setMicOn(mic)
        session.setCameraOn(camera)
    }

    /** Runs as a foreground service with only the types in use, so Android shows the right indicators. */
    private fun foreground() {
        val state = mutableState.value
        val notification = notifier.build(state)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            var types = 0
            if (state.micOn) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
            if (state.camera.on) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_CAMERA
            startForeground(Notifier.ID, notification, types)
        } else {
            startForeground(Notifier.ID, notification)
        }
    }

    // The session calls these from its own threads. Everything runs on the main thread, so it can't race with onDestroy.

    override fun onLink(link: Link) {
        mainThread.post {
            if (session == null || link == mutableState.value.link) return@post
            mutableState.value = mutableState.value.copy(link = link)
            notifier.show(mutableState.value)
        }
    }

    override fun onMuted(muted: Boolean) {
        mainThread.post {
            if (session == null || muted == mutableState.value.muted) return@post
            mutableState.value = mutableState.value.copy(muted = muted)
            notifier.show(mutableState.value)
        }
    }

    override fun onCamera(camera: CameraState) {
        mainThread.post {
            val state = mutableState.value
            if (session == null || camera == state.camera) return@post
            mutableState.value = state.copy(camera = camera)
            when {
                camera.on == state.camera.on -> notifier.show(mutableState.value)
                !camera.on && !state.micOn -> stopSelf() // The PC turned the camera off, and nothing else is on.
                else -> foreground() // The camera type comes and goes with the camera.
            }
        }
    }

    override fun onCableHint(on: Boolean) {
        mainThread.post {
            if (session == null) return@post
            mutableState.value = mutableState.value.copy(cableWithoutLink = on)
        }
    }

    override fun onDisconnectedByPc() {
        mainThread.post { if (session != null) stopSelf() }
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
        const val ACTION_MUTE = "com.mikey.action.MUTE"
        const val ACTION_UNMUTE = "com.mikey.action.UNMUTE"
        const val ACTION_FLIP = "com.mikey.action.FLIP"
        const val ACTION_MIC_ON = "com.mikey.action.MIC_ON"
        const val ACTION_MIC_OFF = "com.mikey.action.MIC_OFF"
        const val ACTION_CAMERA_ON = "com.mikey.action.CAMERA_ON"
        const val ACTION_CAMERA_OFF = "com.mikey.action.CAMERA_OFF"

        private val mutableState = MutableStateFlow(MikeyState())
        val state: StateFlow<MikeyState> = mutableState.asStateFlow()

        /** Call only from the app on screen: Android 14+ refuses to start a mic or camera service from the background. */
        fun micOn(context: Context) = startOn(context, ACTION_MIC_ON)

        fun cameraOn(context: Context) = startOn(context, ACTION_CAMERA_ON)

        fun micOff(context: Context) = tell(context, ACTION_MIC_OFF)

        fun cameraOff(context: Context) = tell(context, ACTION_CAMERA_OFF)

        fun flip(context: Context) = tell(context, ACTION_FLIP)

        private fun startOn(context: Context, action: String) {
            context.startForegroundService(Intent(context, MikeyService::class.java).setAction(action))
        }

        /** For a service that is already running. */
        private fun tell(context: Context, action: String) {
            context.startService(Intent(context, MikeyService::class.java).setAction(action))
        }
    }
}
