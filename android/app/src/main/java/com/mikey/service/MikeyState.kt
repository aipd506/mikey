package com.mikey.service

import com.mikey.media.Lens

/** Where the mic is going right now. */
sealed interface Link {
    /** Not connected: looking for the PC, or reconnecting after a drop. */
    data object Searching : Link

    /** The PC is asking its user whether to allow this phone. */
    data object Waiting : Link

    /** Streaming. [level] is 1 = USB, 4 = Wi-Fi. */
    data class Live(val level: Int) : Link

    /** The PC turned us away and we stopped trying. [reason] is the PC's word for it, e.g. `denied`. */
    data class Refused(val reason: String) : Link
}

/** Why the camera can't send video right now, even though it's on. */
enum class CameraBlock {
    /** Bluetooth can't carry video (connection-levels.md). */
    BLUETOOTH,

    /** The PC has no virtual camera to show it in. */
    PC,
}

/** The camera: whether the user turned it on, which lens, and whether video can't go out just now. */
data class CameraState(val on: Boolean = false, val lens: Lens = Lens.BACK, val blocked: CameraBlock? = null)

/**
 * What the screen shows. MikeyService owns it; the UI only reads it.
 * [muted] is the soft mute: still capturing, but sending silence.
 * [cableWithoutLink]: a USB cable to a computer is in, but neither USB level works, so the UI can
 * suggest turning on USB tethering (connection-levels.md).
 */
data class MikeyState(
    val micOn: Boolean = false,
    val link: Link = Link.Searching,
    val muted: Boolean = false,
    val camera: CameraState = CameraState(),
    val cableWithoutLink: Boolean = false,
)
