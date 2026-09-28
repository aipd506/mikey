package com.mikey

import android.Manifest
import android.content.Intent
import android.content.SharedPreferences
import android.content.pm.PackageManager
import android.graphics.Color
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.provider.Settings.ACTION_APPLICATION_DETAILS_SETTINGS
import android.provider.Settings.ACTION_WIRELESS_SETTINGS
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts.RequestMultiplePermissions
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.mikey.service.MikeyService
import com.mikey.settings.Settings
import com.mikey.ui.MainActions
import com.mikey.ui.MainScreen
import com.mikey.ui.SettingsView
import com.mikey.ui.SheetActions
import com.mikey.ui.view

class MainActivity : ComponentActivity() {
    private val settings by lazy { Settings(this) }
    private val version by lazy { packageManager.getPackageInfo(packageName, 0).versionName.orEmpty() }
    private var prefs by mutableStateOf<SettingsView?>(null)
    private var micDenied by mutableStateOf(false)

    /** Held here because SharedPreferences only keeps its listeners weakly. */
    private var settingsListener: SharedPreferences.OnSharedPreferenceChangeListener? = null

    /** "Remember mic/camera state" is applied once per launch, from the foreground. */
    private var restorePending = false

    private val requestMicPermissions = registerForActivityResult(RequestMultiplePermissions()) {
        // Only the mic is a must. Notifications and Bluetooth are nice to have; without them the app still works.
        micDenied = !granted(Manifest.permission.RECORD_AUDIO)
        if (!micDenied) turnMicOn()
    }

    private val requestCameraPermission = registerForActivityResult(RequestMultiplePermissions()) {
        if (granted(Manifest.permission.CAMERA)) turnCameraOn()
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
        )
        restorePending = savedInstanceState == null
        // Changes from the sheet and from the PC both land in the settings, so the screen follows either.
        prefs = settings.view(version)
        settingsListener = settings.observe {
            prefs = settings.view(version)
            applyKeepScreenOn()
        }
        applyKeepScreenOn()
        val actions = MainActions(::onMicTap, ::onCameraTap, { MikeyService.flip(this) }, sheetActions())
        setContent {
            val state by MikeyService.state.collectAsState()
            val level by MikeyService.micLevel.collectAsState()
            prefs?.let { MainScreen(state, level, micDenied, it, actions) }
        }
    }

    override fun onResume() {
        super.onResume()
        if (granted(Manifest.permission.RECORD_AUDIO)) micDenied = false
        if (restorePending) {
            restorePending = false
            restoreState()
        }
    }

    override fun onDestroy() {
        settingsListener?.let(settings::stopObserving)
        super.onDestroy()
    }

    private fun onMicTap() {
        if (MikeyService.state.value.micOn) {
            settings.lastMicOn = false
            MikeyService.micOff(this)
            return
        }
        val missing = micPermissions(settings).filter { !granted(it) }
        when {
            missing.isEmpty() -> turnMicOn()
            // After a second no, Android stops asking: only the app's settings page can allow it.
            micDenied && !shouldShowRequestPermissionRationale(Manifest.permission.RECORD_AUDIO) -> openAppSettings()
            else -> requestMicPermissions.launch(missing.toTypedArray())
        }
    }

    /** Camera permission is asked on the first camera tap, never up front. */
    private fun onCameraTap() {
        when {
            MikeyService.state.value.camera.on -> {
                settings.lastCameraOn = false
                MikeyService.cameraOff(this)
            }
            granted(Manifest.permission.CAMERA) -> turnCameraOn()
            else -> requestCameraPermission.launch(arrayOf(Manifest.permission.CAMERA))
        }
    }

    private fun turnMicOn() {
        settings.lastMicOn = true
        MikeyService.micOn(this)
    }

    private fun turnCameraOn() {
        settings.lastCameraOn = true
        MikeyService.cameraOn(this)
    }

    /** Turns back on what was on last time, only if the user asked for that and the permission is still there. */
    private fun restoreState() {
        val state = MikeyService.state.value
        if (!settings.rememberState || state.micOn || state.camera.on) return
        if (settings.lastMicOn && granted(Manifest.permission.RECORD_AUDIO)) MikeyService.micOn(this)
        if (settings.lastCameraOn && granted(Manifest.permission.CAMERA)) MikeyService.cameraOn(this)
    }

    private fun sheetActions() = SheetActions(
        setMuted = { MikeyService.mute(this, it) },
        setAudio = {
            settings.audio = it
            MikeyService.settingsChanged(this)
        },
        setAspect = {
            settings.aspect = it
            MikeyService.settingsChanged(this)
        },
        setQuality = {
            settings.quality = it
            MikeyService.settingsChanged(this)
        },
        setFps = {
            settings.fps = it
            MikeyService.settingsChanged(this)
        },
        setLossless = { settings.losslessWifi = it },
        setKeepScreenOn = { settings.keepScreenOn = it },
        setRememberState = { settings.rememberState = it },
        setLevel = { level, on ->
            val next = if (on) settings.enabledLevels + level else settings.enabledLevels - level
            // At least one way to the PC has to stay on.
            if (next.isNotEmpty()) settings.enabledLevels = next
        },
        setManualAddress = { settings.manualPcAddress = it },
        forget = settings::forgetPc,
        openTethering = ::openTethering,
    )

    private fun applyKeepScreenOn() {
        if (settings.keepScreenOn) {
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        } else {
            window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }

    /** There is no public intent for the tethering page. This one works on most phones; otherwise the network settings. */
    private fun openTethering() {
        try {
            startActivity(Intent().setClassName("com.android.settings", "com.android.settings.TetherSettings"))
        } catch (e: RuntimeException) {
            startActivity(Intent(ACTION_WIRELESS_SETTINGS))
        }
    }

    private fun openAppSettings() {
        startActivity(Intent(ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null)))
    }

    private fun granted(permission: String) = checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED
}

/**
 * Asked on the first mic tap, never up front. Android 13+ needs notification permission for the
 * status notification, and Android 12+ needs Bluetooth permission to reach a paired PC over it.
 */
private fun micPermissions(settings: Settings): List<String> = buildList {
    add(Manifest.permission.RECORD_AUDIO)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) add(Manifest.permission.POST_NOTIFICATIONS)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && 4 in settings.enabledLevels) add(Manifest.permission.BLUETOOTH_CONNECT)
}
