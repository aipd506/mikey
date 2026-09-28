@file:OptIn(ExperimentalTextApi::class)

package com.mikey.ui

import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.mikey.R

// Geist and Geist Mono, variable fonts bundled under the SIL Open Font License (android/licenses).
private val geist = FontFamily(
    Font(R.font.geist, FontWeight.Normal, variationSettings = FontVariation.Settings(FontVariation.weight(400))),
    Font(R.font.geist, FontWeight.Medium, variationSettings = FontVariation.Settings(FontVariation.weight(500))),
)
private val geistMono = FontFamily(
    Font(R.font.geist_mono, FontWeight.Normal, variationSettings = FontVariation.Settings(FontVariation.weight(400))),
)

/** Text styles from vibe/rules/design-language.md. */
object Type {
    /** State labels and section titles, shown in capitals: MIC LIVE, AUDIO. */
    val label = TextStyle(fontFamily = geistMono, fontSize = 11.sp, letterSpacing = 0.14.em, color = Palette.textSecondary)
    val header = TextStyle(fontFamily = geist, fontWeight = FontWeight.Medium, fontSize = 17.sp, lineHeight = 23.sp, color = Palette.text)
    val body = TextStyle(fontFamily = geist, fontSize = 15.sp, color = Palette.text)
    val line = TextStyle(fontFamily = geist, fontSize = 14.sp, lineHeight = 20.sp, textAlign = TextAlign.Center, color = Palette.text)
    val hint = TextStyle(fontFamily = geist, fontSize = 12.5.sp, lineHeight = 17.sp, color = Palette.textSecondary)
    val segment = TextStyle(fontFamily = geist, fontSize = 14.sp, color = Palette.text)
    val value = TextStyle(fontFamily = geistMono, fontSize = 12.sp, color = Palette.textSecondary)
    val scale = TextStyle(fontFamily = geistMono, fontSize = 10.sp, letterSpacing = 0.12.em, color = Palette.textSecondary)
    val input = TextStyle(fontFamily = geistMono, fontSize = 15.sp, color = Palette.text)
    val footer = TextStyle(fontFamily = geistMono, fontSize = 11.sp, letterSpacing = 0.08.em, textAlign = TextAlign.Center, color = Palette.textSecondary)
}
