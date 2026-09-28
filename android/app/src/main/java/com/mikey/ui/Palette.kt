package com.mikey.ui

import androidx.compose.ui.graphics.Color

/** Colors from vibe/rules/design-language.md. */
object Palette {
    val bg = Color(0xFF000000)
    val sheet = Color(0xFF111111)
    val tile = Color(0xFF1C1C1C)
    val hairline = Color(0xFF2A2A2A)
    val text = Color(0xFFFFFFFF)
    val textSecondary = Color(0xFF8E8E93)
    val inactive = Color(0xFF3A3A3C)

    /** Only when the PC is receiving: the on-air dot on the mic and camera. */
    val live = Color(0xFFD71921)

    /** The dot texture behind the camera half. */
    val grid = Color(0xFF1A1A1A)
    val scrim = Color(0xB8000000)

    /** The status dot, the same on the phone and the PC. */
    val statusOk = Color(0xFF30D158)
    val statusWait = Color(0xFFFFD60A)
    val statusErr = Color(0xFFFF453A)
}
