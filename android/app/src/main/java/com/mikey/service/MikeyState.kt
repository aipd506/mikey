package com.mikey.service

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

/** What the screen shows. MikeyService owns it; the UI only reads it. */
data class MikeyState(
    val micOn: Boolean = false,
    val link: Link = Link.Searching,
)
