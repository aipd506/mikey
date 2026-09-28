package com.mikey.ui

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsBottomHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.mikey.R
import com.mikey.media.Aspect
import com.mikey.media.Fps
import com.mikey.media.Quality
import com.mikey.protocol.AudioSettings
import com.mikey.service.MikeyState
import com.mikey.settings.Settings
import kotlin.math.roundToInt

/** The settings the sheet shows, read again whenever one changes (on the phone, or from the PC). */
data class SettingsView(
    val audio: AudioSettings,
    val aspect: Aspect,
    val quality: Quality,
    val fps: Fps,
    val lossless: Boolean,
    val keepScreenOn: Boolean,
    val rememberState: Boolean,
    val levels: Set<Int>,
    val manualAddress: String,
    val pcName: String?,
    val lastLevel: Int,
    val version: String,
)

fun Settings.view(version: String) = SettingsView(
    audio, aspect, quality, fps, losslessWifi, keepScreenOn, rememberState, enabledLevels,
    manualPcAddress.orEmpty(), pairedPc?.name?.ifEmpty { null }, lastLevel, version,
)

/**
 * What the sheet can change. Mute and the audio settings are shared with the PC: MainActivity
 * saves them and sends them straight away, and the PC's changes come back through [SettingsView].
 */
class SheetActions(
    val setMuted: (Boolean) -> Unit,
    val setAudio: (AudioSettings) -> Unit,
    val setAspect: (Aspect) -> Unit,
    val setQuality: (Quality) -> Unit,
    val setFps: (Fps) -> Unit,
    val setLossless: (Boolean) -> Unit,
    val setKeepScreenOn: (Boolean) -> Unit,
    val setRememberState: (Boolean) -> Unit,
    val setLevel: (Int, Boolean) -> Unit,
    val setManualAddress: (String) -> Unit,
    val forget: () -> Unit,
    val openTethering: () -> Unit,
)

@Composable
fun SettingsSheet(state: MikeyState, prefs: SettingsView, actions: SheetActions, onClose: () -> Unit) {
    var advanced by rememberSaveable { mutableStateOf(false) }
    Column(Modifier.fillMaxSize()) {
        Box(Modifier.fillMaxWidth().height(28.dp).clickable(interactionSource = null, indication = null, onClick = onClose), contentAlignment = Alignment.Center) {
            Box(Modifier.size(32.dp, 4.dp).background(Palette.inactive, RoundedCornerShape(2.dp)))
        }
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
            val header = sheetHeader(state)
            val pc = prefs.pcName ?: stringResource(R.string.the_pc)
            BasicText(
                stringResource(header.text, pc, header.level?.let { stringResource(it) }.orEmpty()),
                Modifier.padding(start = 4.dp, end = 4.dp, top = 6.dp, bottom = 4.dp),
                style = Type.header,
            )
            if (state.cableWithoutLink) {
                Spacer(Modifier.height(12.dp))
                Tile { LinkRow(stringResource(R.string.tether_hint), actions.openTethering) }
            }

            SectionLabel(stringResource(R.string.section_camera))
            Tile { SegmentRow(stringResource(R.string.aspect_ratio), Aspect.entries.map { it to it.wire }, prefs.aspect, actions.setAspect) }

            SectionLabel(stringResource(R.string.section_mic))
            Tile { SwitchRow(stringResource(R.string.mute), state.muted, actions.setMuted, enabled = state.micOn) }

            SectionLabel(stringResource(R.string.section_audio))
            AudioTile(state, prefs.audio, actions.setAudio)

            Spacer(Modifier.height(12.dp))
            AdvancedRow(advanced) { advanced = !advanced }
            if (advanced) Advanced(state, prefs, actions)

            BasicText(stringResource(R.string.version, prefs.version), Modifier.fillMaxWidth().padding(top = 28.dp, bottom = 32.dp), style = Type.footer)
            Spacer(Modifier.windowInsetsBottomHeight(WindowInsets.navigationBars))
        }
    }
}

/** Noise suppression and echo cancellation run on the PC; greyed out if the connected PC can't do them. */
@Composable
private fun AudioTile(state: MikeyState, audio: AudioSettings, setAudio: (AudioSettings) -> Unit) {
    val nsOk = pcCan(state, "rnnoise")
    val aecOk = pcCan(state, "aec")
    val notHere = stringResource(R.string.not_on_this_pc)
    Tile {
        SwitchRow(stringResource(R.string.noise_suppression), audio.ns, { setAudio(audio.copy(ns = it)) }, enabled = nsOk, subtitle = notHere.takeUnless { nsOk })
        Hairline()
        SwitchRow(stringResource(R.string.echo_cancellation), audio.aec, { setAudio(audio.copy(aec = it)) }, enabled = aecOk, subtitle = notHere.takeUnless { aecOk })
    }
}

@Composable
private fun AdvancedRow(open: Boolean, onToggle: () -> Unit) {
    val turn by animateFloatAsState(if (open) 180f else 0f, tween(120, easing = Standard), label = "advanced")
    Tile {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 64.dp).clickable(onClick = onToggle).padding(horizontal = 20.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            BasicText(stringResource(R.string.advanced), Modifier.weight(1f), style = Type.body)
            Glyph(Glyphs.chevronDown, Palette.textSecondary, 20.dp, Modifier.rotate(turn))
        }
    }
}

@Composable
private fun Advanced(state: MikeyState, prefs: SettingsView, actions: SheetActions) {
    val audio = prefs.audio
    val off = stringResource(R.string.gate_off)

    SectionLabel(stringResource(R.string.section_audio_short))
    Tile {
        // The sliders follow the finger locally and save (and reach the PC) when it lifts.
        var ns by remember { mutableStateOf<Float?>(null) }
        val nsOk = pcCan(state, "rnnoise") && audio.ns
        val nsShown = ns ?: audio.nsStrength
        SliderRow(
            stringResource(R.string.ns_strength),
            if (nsOk) stringResource(R.string.ns_value, (nsShown * 100).roundToInt()) else off,
            nsShown,
            steps = 100,
            scale = listOf(stringResource(R.string.scale_low).uppercase(), stringResource(R.string.scale_high).uppercase()),
            onChange = { ns = it },
            onDone = {
                ns?.let { actions.setAudio(audio.copy(nsStrength = it)) }
                ns = null
            },
            enabled = nsOk,
        )
        Hairline()
        var gate by remember { mutableStateOf<Int?>(null) }
        val step = gate ?: gateStep(audio.gateDb)
        SliderRow(
            stringResource(R.string.gate),
            if (step == 0) off else stringResource(R.string.gate_value, 61 - step),
            step / GATE_STEPS.toFloat(),
            steps = GATE_STEPS,
            // Capitals for words only: "dB" keeps its case.
            scale = listOf(off.uppercase(), stringResource(R.string.gate_min), stringResource(R.string.gate_max)),
            onChange = { gate = (it * GATE_STEPS).roundToInt() },
            onDone = {
                gate?.let { actions.setAudio(audio.copy(gateDb = gateDbAt(it))) }
                gate = null
            },
            hollow = step == 0,
        )
        Hairline()
        SwitchRow(stringResource(R.string.lossless), prefs.lossless, actions.setLossless)
    }

    val auto = stringResource(R.string.auto)
    SectionLabel(stringResource(R.string.section_video))
    Tile {
        SegmentRow(stringResource(R.string.video_quality), Quality.entries.map { it to if (it == Quality.AUTO) auto else it.wire }, prefs.quality, actions.setQuality)
        Hairline()
        SegmentRow(stringResource(R.string.frame_rate), Fps.entries.map { it to if (it == Fps.AUTO) auto else it.wire }, prefs.fps, actions.setFps)
    }

    SectionLabel(stringResource(R.string.section_phone))
    Tile {
        SwitchRow(stringResource(R.string.keep_screen_on), prefs.keepScreenOn, actions.setKeepScreenOn)
        Hairline()
        SwitchRow(stringResource(R.string.remember_state), prefs.rememberState, actions.setRememberState)
    }

    SectionLabel(stringResource(R.string.section_levels))
    Tile {
        LEVELS.forEachIndexed { i, (level, name, icon) ->
            if (i > 0) Hairline()
            SwitchRow(stringResource(name), level in prefs.levels, { actions.setLevel(level, it) }, icon = icon)
        }
    }

    SectionLabel(stringResource(R.string.section_address))
    AddressField(prefs.manualAddress, actions.setManualAddress)

    SectionLabel(stringResource(R.string.section_paired))
    Tile { PairedComputer(prefs, actions.forget) }
    Spacer(Modifier.height(12.dp))
    Tile { LinkRow(stringResource(R.string.open_tethering), actions.openTethering) }
}

@Composable
private fun AddressField(initial: String, onChange: (String) -> Unit) {
    var text by rememberSaveable { mutableStateOf(initial) }
    var focused by remember { mutableStateOf(false) }
    val shape = RoundedCornerShape(24.dp)
    BasicTextField(
        value = text,
        onValueChange = {
            text = it
            onChange(it)
        },
        modifier = Modifier.fillMaxWidth().height(56.dp).onFocusChanged { focused = it.isFocused },
        singleLine = true,
        textStyle = Type.input,
        cursorBrush = SolidColor(Palette.text),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Uri, imeAction = ImeAction.Done),
        decorationBox = { field ->
            Box(
                Modifier.fillMaxSize().clip(shape).background(Palette.tile).border(1.dp, if (focused) Palette.text else Palette.hairline, shape).padding(horizontal = 20.dp),
                contentAlignment = Alignment.CenterStart,
            ) {
                if (text.isEmpty()) BasicText(stringResource(R.string.address_placeholder), style = Type.input.copy(color = Palette.inactive))
                field()
            }
        },
    )
    BasicText(stringResource(R.string.address_hint), Modifier.padding(start = 4.dp, top = 8.dp), style = Type.hint)
}

@Composable
private fun PairedComputer(prefs: SettingsView, forget: () -> Unit) {
    val name = prefs.pcName
    if (name == null) {
        Box(Modifier.fillMaxWidth().heightIn(min = 64.dp).padding(horizontal = 20.dp), contentAlignment = Alignment.CenterStart) {
            BasicText(stringResource(R.string.no_paired), style = Type.body.copy(color = Palette.textSecondary))
        }
        return
    }
    Row(Modifier.fillMaxWidth().heightIn(min = 64.dp).padding(start = 20.dp, end = 12.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            BasicText(name, style = Type.body)
            if (prefs.lastLevel != 0) BasicText(stringResource(R.string.paired_last, stringResource(shortLevelName(prefs.lastLevel))), style = Type.hint)
        }
        Box(
            Modifier.padding(4.dp).height(40.dp).clip(Pill).border(1.dp, Palette.inactive, Pill).clickable(onClick = forget).padding(horizontal = 18.dp),
            contentAlignment = Alignment.Center,
        ) {
            BasicText(stringResource(R.string.forget), style = Type.segment)
        }
    }
}

/** The connection levels in order of preference, with their names and icons. */
private val LEVELS = listOf(
    Triple(1, R.string.toggle_usb_debugging, Glyphs.usb),
    Triple(2, R.string.toggle_usb_tethering, Glyphs.usb),
    Triple(3, R.string.level_wifi, Glyphs.wifi),
    Triple(4, R.string.level_bluetooth, Glyphs.bluetooth),
)
