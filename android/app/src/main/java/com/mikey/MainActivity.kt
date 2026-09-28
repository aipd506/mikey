package com.mikey

import android.Manifest
import android.app.AlertDialog
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import android.os.Bundle
import android.text.InputType
import android.widget.EditText
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts.RequestMultiplePermissions
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import com.mikey.service.MikeyService
import com.mikey.settings.Settings
import com.mikey.ui.SplitScreen

class MainActivity : ComponentActivity() {

    private val requestMicPermissions = registerForActivityResult(RequestMultiplePermissions()) {
        // Only the mic is a must. Notifications and Bluetooth are nice to have; without them the app still works.
        if (granted(Manifest.permission.RECORD_AUDIO)) MikeyService.micOn(this)
    }

    private val requestCameraPermission = registerForActivityResult(RequestMultiplePermissions()) {
        if (granted(Manifest.permission.CAMERA)) MikeyService.cameraOn(this)
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(Color.TRANSPARENT),
        )
        setContent {
            val state by MikeyService.state.collectAsState()
            SplitScreen(
                state,
                onMicTap = ::onMicTap,
                onCameraTap = ::onCameraTap,
                onFlip = { MikeyService.flip(this) },
                onStatusLongPress = if (isDebuggable()) ::askPcAddress else null,
            )
        }
    }

    private fun onMicTap() {
        if (MikeyService.state.value.micOn) {
            MikeyService.micOff(this)
            return
        }
        val missing = micPermissions(Settings(this)).filter { !granted(it) }
        if (missing.isEmpty()) MikeyService.micOn(this) else requestMicPermissions.launch(missing.toTypedArray())
    }

    /** Camera permission is asked on the first camera tap, never up front. */
    private fun onCameraTap() {
        when {
            MikeyService.state.value.camera.on -> MikeyService.cameraOff(this)
            granted(Manifest.permission.CAMERA) -> MikeyService.cameraOn(this)
            else -> requestCameraPermission.launch(arrayOf(Manifest.permission.CAMERA))
        }
    }

    private fun granted(permission: String) = checkSelfPermission(permission) == PackageManager.PERMISSION_GRANTED

    /** Debug builds only: type the PC's address to test over Wi-Fi (empty means USB), or forget the paired PC. */
    private fun askPcAddress() {
        val settings = Settings(this)
        val field = EditText(this).apply {
            hint = getString(R.string.debug_pc_address_hint)
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_URI
            setText(settings.manualPcAddress.orEmpty())
        }
        val dialog = AlertDialog.Builder(this)
            .setTitle(R.string.debug_pc_address_title)
            .setMessage(R.string.debug_pc_address_message)
            .setView(field)
            .setPositiveButton(R.string.debug_pc_address_save) { _, _ -> settings.manualPcAddress = field.text.toString() }
            .setNegativeButton(android.R.string.cancel, null)
        settings.pairedPc?.let { pc ->
            dialog.setNeutralButton(getString(R.string.debug_forget_pc, pc.name)) { _, _ -> settings.forgetPc() }
        }
        dialog.show()
    }

    private fun isDebuggable() = (applicationInfo.flags and ApplicationInfo.FLAG_DEBUGGABLE) != 0
}

/**
 * Asked on the first mic tap, never up front. Android 13+ needs notification permission for the
 * status notification, and Android 12+ needs Bluetooth permission to reach a paired PC over it.
 */
private fun micPermissions(settings: Settings): List<String> = buildList {
    add(Manifest.permission.RECORD_AUDIO)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) add(Manifest.permission.POST_NOTIFICATIONS)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && 3 in settings.enabledLevels) add(Manifest.permission.BLUETOOTH_CONNECT)
}
