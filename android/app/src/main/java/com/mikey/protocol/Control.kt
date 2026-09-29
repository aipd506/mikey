package com.mikey.protocol

import org.json.JSONObject

/** The audio settings the PC applies for us (docs/AUDIO_PIPELINE.md). */
data class AudioSettings(val ns: Boolean, val nsStrength: Float)

/**
 * A CONTROL frame (wire-protocol.md). Every part is optional: [audio] carries our settings,
 * [muted] our soft mute, [videoOn] whether the camera is on and [lens] which one.
 */
fun controlPayload(audio: AudioSettings? = null, muted: Boolean? = null, videoOn: Boolean? = null, lens: String? = null): ByteArray {
    val json = JSONObject()
    if (audio != null || muted != null) {
        val section = JSONObject()
        audio?.let {
            section.put("ns", it.ns)
                .put("ns_strength", it.nsStrength.toDouble())
        }
        muted?.let { section.put("muted", it) }
        json.put("audio", section)
    }
    if (videoOn != null || lens != null) {
        val section = JSONObject()
        videoOn?.let { section.put("on", it) }
        lens?.let { section.put("lens", it) }
        json.put("video", section)
    }
    return json.toString().toByteArray()
}

/** What a CONTROL frame from the PC asks for. A field that is null was left out, so it stays as it is. */
class ControlUpdate(
    val ns: Boolean? = null,
    val nsStrength: Float? = null,
    val muted: Boolean? = null,
    val videoOn: Boolean? = null,
    /** `back`, `front` or `flip`. */
    val lens: String? = null,
) {
    fun applyTo(settings: AudioSettings) = AudioSettings(
        ns = ns ?: settings.ns,
        nsStrength = nsStrength ?: settings.nsStrength,
    )
}

fun parseControl(payload: ByteArray): ControlUpdate {
    val json = JSONObject(String(payload))
    val audio = json.optJSONObject("audio")
    val video = json.optJSONObject("video")
    return ControlUpdate(
        ns = audio?.bool("ns"),
        nsStrength = audio?.number("ns_strength"),
        muted = audio?.bool("muted"),
        videoOn = video?.bool("on"),
        lens = video?.text("lens"),
    )
}

private fun JSONObject.bool(key: String): Boolean? = if (has(key) && !isNull(key)) getBoolean(key) else null

private fun JSONObject.number(key: String): Float? = if (has(key) && !isNull(key)) getDouble(key).toFloat() else null

private fun JSONObject.text(key: String): String? = if (has(key) && !isNull(key)) getString(key) else null
