package com.mikey.protocol

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.nio.ByteBuffer

const val PROTO_VERSION = 2

/**
 * [token] is the pairing token the PC gave us last time, or null when we have none. [camera] is
 * whether this phone has one.
 */
fun helloPayload(deviceId: String, deviceName: String, level: Int, token: String?, camera: Boolean): ByteArray =
    JSONObject()
        .put("proto", PROTO_VERSION)
        .put("device_id", deviceId)
        .put("device_name", deviceName)
        .put("level", level)
        .putOpt("token", token)
        .put("caps", JSONArray(phoneCaps(camera, level)))
        .toString()
        .toByteArray()

/** What the phone can send right now. Never video over Bluetooth (level 4): it's too narrow. */
internal fun phoneCaps(camera: Boolean, level: Int): List<String> =
    if (camera && level != 4) listOf("audio", "video") else listOf("audio")

/**
 * [token] goes into every later HELLO. [resumed] means the PC kept our session across a drop.
 * [pcCaps] is what the PC can do, e.g. `opus`.
 */
class Welcome(val pcId: String, val pcName: String, val token: String, val resumed: Boolean, val pcCaps: Set<String>)

fun parseWelcome(payload: ByteArray): Welcome {
    val json = JSONObject(String(payload))
    val caps = json.optJSONArray("pc_caps")
    return Welcome(
        json.getString("pc_id"),
        json.getString("pc_name"),
        json.getString("token"),
        json.optBoolean("resumed", false),
        if (caps == null) emptySet() else (0 until caps.length()).map { caps.getString(it) }.toSet(),
    )
}

/** The reason in a REJECT or BYE from the PC, e.g. `denied` or `disconnect`. "unknown" if it sent none. */
fun parseReason(payload: ByteArray): String =
    try {
        JSONObject(String(payload)).optString("reason", "unknown")
    } catch (e: JSONException) {
        "unknown"
    }

fun heartbeatPayload(sentAtUs: Long): ByteArray = ByteBuffer.allocate(Long.SIZE_BYTES).putLong(sentAtUs).array()

fun heartbeatSentAt(payload: ByteArray): Long = ByteBuffer.wrap(payload).long

fun byePayload(reason: String): ByteArray = JSONObject().put("reason", reason).toString().toByteArray()
