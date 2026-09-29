package com.mikey.service

import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.Handler
import android.os.IBinder
import android.os.Looper
import com.mikey.settings.Settings
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
            ACTION_STOP -> {
                // A deliberate stop: nothing to turn back on next time, even with "Remember mic/camera state".
                Settings(this).run {
                    lastMicOn = false
                    lastCameraOn = false
                }
                stopSelf()
            }
            ACTION_SETTINGS -> session?.settingsChanged()
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

    /**
     * Applies what should be on, and remembers it for "Remember mic/camera state", since the
     * notification's buttons change it too. With nothing on, the service ends.
     */
    private fun turn(mic: Boolean, camera: Boolean) {
        Settings(this).run {
            lastMicOn = mic
            lastCameraOn = camera
        }
        if (!mic && !camera) {
            stopSelf()
            return
        }
        val before = mutableState.value
        mutableState.value = before.copy(micOn = mic, camera = before.camera.copy(on = camera))
        // Android lets a notification tap turn the mic or camera on from the background. If a phone
        // refuses anyway, keep what was on instead of crashing.
        if (!foreground()) {
            mutableState.value = before
            Settings(this).run {
                lastMicOn = before.micOn
                lastCameraOn = before.camera.on
            }
            if (!(before.micOn || before.camera.on) || !foreground()) stopSelf()
            return
        }
        val session = session ?: SessionController(this, this).also {
            session = it
            it.start()
        }
        session.setMicOn(mic)
        session.setCameraOn(camera)
    }

    /**
     * Runs as a foreground service with only the types in use, so Android shows the right
     * indicators. False if Android won't allow those types right now.
     */
    private fun foreground(): Boolean {
        val state = mutableState.value
        val notification = notifier.build(state)
        try {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                var types = 0
                if (state.micOn) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
                if (state.camera.on) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_CAMERA
                startForeground(Notifier.ID, notification, types)
            } else {
                startForeground(Notifier.ID, notification)
            }
        } catch (e: SecurityException) {
            return false
        } catch (e: IllegalStateException) {
            return false
        }
        return true
    }

    // The session calls these from its own threads. Everything runs on the main thread, so it can't race with onDestroy.

    override fun onLink(link: Link) {
        mainThread.post {
            val state = mutableState.value
            if (session == null || link == state.link) return@post
            // Searching right after streaming means the link dropped, not that there never was a PC.
            val reconnecting = link == Link.Searching && (state.link is Link.Live || state.reconnecting)
            mutableState.value = state.copy(link = link, reconnecting = reconnecting)
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
                // The camera type comes and goes with the camera. If Android won't add it, the camera stays off.
                else -> if (!foreground()) session?.setCameraOn(false)
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
        mainThread.post {
            if (session == null) return@post
            Settings(this).run {
                lastMicOn = false
                lastCameraOn = false
            }
            stopSelf()
        }
    }

    override fun onMicLevel(level: Float) {
        mutableMicLevel.value = level
    }

    override fun onDestroy() {
        session?.stop()
        session = null
        mainThread.removeCallbacksAndMessages(null)
        mutableState.value = MikeyState()
        mutableMicLevel.value = 0f
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
        const val ACTION_SETTINGS = "com.mikey.action.SETTINGS"

        private val mutableState = MutableStateFlow(MikeyState())
        val state: StateFlow<MikeyState> = mutableState.asStateFlow()

        private val mutableMicLevel = MutableStateFlow(0f)

        /** How loud the mic is, 0 to 1, while it's on. */
        val micLevel: StateFlow<Float> = mutableMicLevel.asStateFlow()

        /**
         * Call only from the app on screen, or through the notification's button: Android 14+
         * refuses to start a mic or camera service from the background otherwise.
         */
        fun micOn(context: Context) = startOn(context, ACTION_MIC_ON)

        fun cameraOn(context: Context) = startOn(context, ACTION_CAMERA_ON)

        fun micOff(context: Context) = tell(context, ACTION_MIC_OFF)

        fun cameraOff(context: Context) = tell(context, ACTION_CAMERA_OFF)

        fun flip(context: Context) = tell(context, ACTION_FLIP)

        /** Soft mute from the app. Only while the mic is on. */
        fun mute(context: Context, on: Boolean) {
            if (state.value.micOn) tell(context, if (on) ACTION_MUTE else ACTION_UNMUTE)
        }

        /** The user changed a setting: tell the PC now if we're running; otherwise it goes with the next connection. */
        fun settingsChanged(context: Context) {
            if (state.value.let { it.micOn || it.camera.on }) tell(context, ACTION_SETTINGS)
        }

        private fun startOn(context: Context, action: String) {
            context.startForegroundService(Intent(context, MikeyService::class.java).setAction(action))
        }

        /** For a service that is already running. */
        private fun tell(context: Context, action: String) {
            context.startService(Intent(context, MikeyService::class.java).setAction(action))
        }
    }
}
