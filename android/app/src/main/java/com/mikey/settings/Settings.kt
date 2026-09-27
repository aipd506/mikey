package com.mikey.settings

import android.content.Context
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

    /** After this the next connection counts as new, so Wi-Fi asks for approval again. */
    fun forgetPc() {
        pairedPc = null
        lastPcAddress = null
        pcBtAddress = null
    }

    /** Connection levels the user allows: 1 USB debugging, 2 USB tethering, 3 Bluetooth, 4 Wi-Fi. All by default. */
    val enabledLevels: Set<Int>
        get() = prefs.getStringSet(KEY_LEVELS, null)?.mapNotNull { it.toIntOrNull() }?.toSet() ?: setOf(1, 2, 3, 4)

    /** Send raw PCM on Wi-Fi instead of Opus. Off by default: Opus is transparent and copes better with busy Wi-Fi. */
    var losslessWifi: Boolean
        get() = prefs.getBoolean(KEY_WIFI_LOSSLESS, false)
        set(value) = prefs.edit().putBoolean(KEY_WIFI_LOSSLESS, value).apply()

    /** PC address typed in for Wi-Fi testing (debug builds only). Null means connect over USB. */
    var manualPcAddress: String?
        get() = prefs.getString(KEY_MANUAL_PC_ADDRESS, null)
        set(value) = prefs.edit().putString(KEY_MANUAL_PC_ADDRESS, value?.trim()?.ifEmpty { null }).apply()

    private companion object {
        const val KEY_DEVICE_ID = "device.id"
        const val KEY_PC_ID = "pc.lastId"
        const val KEY_PC_NAME = "pc.lastName"
        const val KEY_PC_TOKEN = "pc.token"
        const val KEY_PC_LAST_IP = "pc.lastIp"
        const val KEY_PC_BT_ADDRESS = "pc.btAddress"
        const val KEY_WIFI_LOSSLESS = "audio.wifiLossless"
        const val KEY_LEVELS = "levels.enabled"
        const val KEY_MANUAL_PC_ADDRESS = "pc.manualAddress"
    }
}

private fun newDeviceId(): String {
    val bytes = ByteArray(16).also { SecureRandom().nextBytes(it) }
    return bytes.joinToString("") { "%02x".format(it) }
}
