package com.mikey.settings

import android.content.Context
import android.content.SharedPreferences
import com.mikey.media.Aspect
import com.mikey.media.Fps
import com.mikey.media.Lens
import com.mikey.media.Quality
import com.mikey.protocol.AudioSettings
import java.security.SecureRandom

/** The PC we're paired with, and the token it gave us to prove it next time. */
class PairedPc(val id: String, val name: String, val token: String)

/** Everything the app saves. Defaults live here and nowhere else. */
class Settings(context: Context) {
    private val prefs = context.getSharedPreferences("mikey", Context.MODE_PRIVATE)

    /** Random 128-bit id as 32 hex chars. Created on first use, then kept for good. */
    val deviceId: String
        get() = prefs.getString(KEY_DEVICE_ID, null)
            ?: newDeviceId().also { prefs.edit().putString(KEY_DEVICE_ID, it).apply() }

    /** The last PC that accepted us. Null until the first WELCOME, and after Forget. */
    var pairedPc: PairedPc?
        get() {
            val id = prefs.getString(KEY_PC_ID, null) ?: return null
            val token = prefs.getString(KEY_PC_TOKEN, null) ?: return null
            return PairedPc(id, prefs.getString(KEY_PC_NAME, null) ?: "", token)
        }
        set(value) = prefs.edit()
            .putString(KEY_PC_ID, value?.id)
            .putString(KEY_PC_NAME, value?.name)
            .putString(KEY_PC_TOKEN, value?.token)
            .apply()

    /** Where the PC was the last time we reached it over Wi-Fi. Tried first, before searching. */
    var lastPcAddress: String?
        get() = prefs.getString(KEY_PC_LAST_IP, null)
        set(value) = prefs.edit().putString(KEY_PC_LAST_IP, value).apply()

    /** The Bluetooth address of the bonded computer that answered before, so only it is tried from then on. */
    var pcBtAddress: String?
        get() = prefs.getString(KEY_PC_BT_ADDRESS, null)
        set(value) = prefs.edit().putString(KEY_PC_BT_ADDRESS, value).apply()

    /** The level we last streamed on, for "Last connected over USB". 0 before the first connection. */
    var lastLevel: Int
        get() = prefs.getInt(KEY_PC_LAST_LEVEL, 0)
        set(value) = prefs.edit().putInt(KEY_PC_LAST_LEVEL, value).apply()

    /** After this the next connection counts as new, so Wi-Fi asks for approval again. */
    fun forgetPc() {
        pairedPc = null
        lastPcAddress = null
        pcBtAddress = null
        lastLevel = 0
    }

    /** Connection levels the user allows: 1 USB debugging, 2 USB tethering, 3 Wi-Fi, 4 Bluetooth. All by default. */
    var enabledLevels: Set<Int>
        get() = prefs.getStringSet(KEY_LEVELS, null)?.mapNotNull { it.toIntOrNull() }?.toSet() ?: setOf(1, 2, 3, 4)
        set(value) = prefs.edit().putStringSet(KEY_LEVELS, value.map { it.toString() }.toSet()).apply()

    /**
     * The audio processing the PC does for us. Defaults: noise suppression on and high, echo
     * cancellation on, noise gate off (phone-ux.md). Either side can change them; the phone keeps them.
     */
    var audio: AudioSettings
        get() = AudioSettings(
            ns = prefs.getBoolean(KEY_NS, true),
            nsStrength = prefs.getFloat(KEY_NS_STRENGTH, 0.8f),
            aec = prefs.getBoolean(KEY_AEC, true),
            gateDb = if (prefs.contains(KEY_GATE_DB)) prefs.getFloat(KEY_GATE_DB, 0f) else null,
        )
        set(value) {
            val edit = prefs.edit()
                .putBoolean(KEY_NS, value.ns)
                .putFloat(KEY_NS_STRENGTH, value.nsStrength)
                .putBoolean(KEY_AEC, value.aec)
            if (value.gateDb == null) edit.remove(KEY_GATE_DB) else edit.putFloat(KEY_GATE_DB, value.gateDb)
            edit.apply()
        }

    /** The camera used last time; a flip is remembered (phone-ux.md). */
    var lens: Lens
        get() = Lens.fromWire(prefs.getString(KEY_LENS, null)) ?: Lens.BACK
        set(value) = prefs.edit().putString(KEY_LENS, value.wire).apply()

    var aspect: Aspect
        get() = Aspect.fromWire(prefs.getString(KEY_ASPECT, null))
        set(value) = prefs.edit().putString(KEY_ASPECT, value.wire).apply()

    var quality: Quality
        get() = Quality.fromWire(prefs.getString(KEY_QUALITY, null))
        set(value) = prefs.edit().putString(KEY_QUALITY, value.wire).apply()

    var fps: Fps
        get() = Fps.fromWire(prefs.getString(KEY_FPS, null))
        set(value) = prefs.edit().putString(KEY_FPS, value.wire).apply()

    /** Send raw PCM on Wi-Fi instead of Opus. Off by default: Opus is transparent and copes better with busy Wi-Fi. */
    var losslessWifi: Boolean
        get() = prefs.getBoolean(KEY_WIFI_LOSSLESS, false)
        set(value) = prefs.edit().putBoolean(KEY_WIFI_LOSSLESS, value).apply()

    /** The PC's address, typed in for networks where discovery can't find it. Null means find it by itself. */
    var manualPcAddress: String?
        get() = prefs.getString(KEY_MANUAL_PC_ADDRESS, null)
        set(value) = prefs.edit().putString(KEY_MANUAL_PC_ADDRESS, value?.trim()?.ifEmpty { null }).apply()

    /** Keeps the screen on while Mikey is open. */
    var keepScreenOn: Boolean
        get() = prefs.getBoolean(KEY_KEEP_SCREEN_ON, false)
        set(value) = prefs.edit().putBoolean(KEY_KEEP_SCREEN_ON, value).apply()

    /** Turn back on at launch what was on last time. Off by default: the mic and camera start off (phone-ux.md). */
    var rememberState: Boolean
        get() = prefs.getBoolean(KEY_REMEMBER_STATE, false)
        set(value) = prefs.edit().putBoolean(KEY_REMEMBER_STATE, value).apply()

    /** What the user last had on, for [rememberState]. */
    var lastMicOn: Boolean
        get() = prefs.getBoolean(KEY_LAST_MIC_ON, false)
        set(value) = prefs.edit().putBoolean(KEY_LAST_MIC_ON, value).apply()

    var lastCameraOn: Boolean
        get() = prefs.getBoolean(KEY_LAST_CAMERA_ON, false)
        set(value) = prefs.edit().putBoolean(KEY_LAST_CAMERA_ON, value).apply()

    /** Calls [onChange] on the main thread after any setting changes in this process. Keep the result to stop. */
    fun observe(onChange: () -> Unit): SharedPreferences.OnSharedPreferenceChangeListener =
        SharedPreferences.OnSharedPreferenceChangeListener { _, _ -> onChange() }.also(prefs::registerOnSharedPreferenceChangeListener)

    fun stopObserving(listener: SharedPreferences.OnSharedPreferenceChangeListener) =
        prefs.unregisterOnSharedPreferenceChangeListener(listener)

    private companion object {
        const val KEY_DEVICE_ID = "device.id"
        const val KEY_PC_ID = "pc.lastId"
        const val KEY_PC_NAME = "pc.lastName"
        const val KEY_PC_TOKEN = "pc.token"
        const val KEY_PC_LAST_IP = "pc.lastIp"
        const val KEY_PC_BT_ADDRESS = "pc.btAddress"
        const val KEY_PC_LAST_LEVEL = "pc.lastLevel"
        const val KEY_WIFI_LOSSLESS = "audio.wifiLossless"
        const val KEY_LENS = "camera.lens"
        const val KEY_ASPECT = "camera.aspect"
        const val KEY_QUALITY = "camera.quality"
        const val KEY_FPS = "camera.fps"
        const val KEY_NS = "audio.ns"
        const val KEY_NS_STRENGTH = "audio.nsStrength"
        const val KEY_AEC = "audio.aec"
        const val KEY_GATE_DB = "audio.gateDb"
        const val KEY_LEVELS = "levels.enabled"
        const val KEY_MANUAL_PC_ADDRESS = "pc.manualAddress"
        const val KEY_KEEP_SCREEN_ON = "ui.keepScreenOn"
        const val KEY_REMEMBER_STATE = "ui.rememberState"
        const val KEY_LAST_MIC_ON = "ui.lastMicOn"
        const val KEY_LAST_CAMERA_ON = "ui.lastCameraOn"
    }
}

private fun newDeviceId(): String {
    val bytes = ByteArray(16).also { SecureRandom().nextBytes(it) }
    return bytes.joinToString("") { "%02x".format(it) }
}
