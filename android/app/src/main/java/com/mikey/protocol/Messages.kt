package com.mikey.protocol

import org.json.JSONArray
import org.json.JSONException
import org.json.JSONObject
import java.nio.ByteBuffer

const val PROTO_VERSION = 1

/** [token] is the pairing token the PC gave us last time, or null when we have none. */
fun helloPayload(deviceId: String, deviceName: String, level: Int, token: String?): ByteArray =
    JSONObject()
        .put("proto", PROTO_VERSION)
        .put("device_id", deviceId)
        .put("device_name", deviceName)
        .put("level", level)
        .putOpt("token", token)
        .put("caps", JSONArray().put("audio"))
        .toString()
        .toByteArray()

/** [token] goes into every later HELLO. [resumed] means the PC kept our session across a drop. */
class Welcome(val pcId: String, val pcName: String, val token: String, val resumed: Boolean)

fun parseWelcome(payload: ByteArray): Welcome {
    val json = JSONObject(String(payload))
    return Welcome(
        json.getString("pc_id"),
        json.getString("pc_name"),
        json.getString("token"),
        json.optBoolean("resumed", false),
    )
}

/** The PC's reason for turning us away, e.g. `denied`. "unknown" if it sent none. */
fun parseReject(payload: ByteArray): String =
    try {
        JSONObject(String(payload)).optString("reason", "unknown")
    } catch (e: JSONException) {
        "unknown"
    }

fun heartbeatPayload(sentAtUs: Long): ByteArray = ByteBuffer.allocate(Long.SIZE_BYTES).putLong(sentAtUs).array()

fun heartbeatSentAt(payload: ByteArray): Long = ByteBuffer.wrap(payload).long

fun byePayload(reason: String): ByteArray = JSONObject().put("reason", reason).toString().toByteArray()
