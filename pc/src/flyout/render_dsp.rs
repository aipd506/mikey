//! Noise suppression slider, AEC toggle, and noise gate toggle controls.
//! Ultra-compact, visual-first DSP controls.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_dsp_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_body: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
    mut y: i32,
) -> i32 {
    // ── Row 1: Noise Suppression slider ──
    draw_text(
        dc,
        font_icon,
        COLOR_TEXT_SECONDARY,
        14,
        y,
        30,
        y + 20,
        "\u{E767}",
        DT_CENTER_V,
    );
    draw_text(
        dc,
        font_body,
        COLOR_TEXT_SECONDARY,
        34,
        y,
        70,
        y + 20,
        "NS",
        0,
    );

    let slider_left = 74;
    let slider_right = FLYOUT_WIDTH - 14;
    let slider_cy = y + 10;
    let slider_touch_rect = rect(slider_left, y, slider_right, y + 20);
    flyout
        .button_rects
        .push((FlyoutButton::NsSlider, slider_touch_rect));
    draw_slider(dc, slider_left, slider_right, slider_cy, flyout.ns_strength);

    y += 24;

    // ── Row 2: Unified DSP Pills: [ AEC ✓ ] and [ Gate ] ──
    let aec_w = 64;
    let aec_rect = rect(14, y, 14 + aec_w, y + 22);
    flyout
        .button_rects
        .push((FlyoutButton::ToggleAec, aec_rect));
    let aec_hover = flyout.hover_btn == Some(FlyoutButton::ToggleAec);

    let (aec_bg, aec_bd, aec_fg) = if flyout.aec_enabled {
        (
            if aec_hover {
                COLOR_BTN_HOVER
            } else {
                COLOR_BTN_BG
            },
            COLOR_MIC_ON,
            COLOR_MIC_ON,
        )
    } else if aec_hover {
        (COLOR_BTN_HOVER, COLOR_BTN_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (COLOR_BTN_BG, COLOR_BTN_BORDER, COLOR_TEXT_SECONDARY)
    };
    draw_pill_bg(
        dc,
        aec_rect.left,
        aec_rect.top,
        aec_rect.right,
        aec_rect.bottom,
        aec_bg,
        aec_bd,
    );
    draw_text(
        dc,
        font_body_bold,
        aec_fg,
        aec_rect.left,
        aec_rect.top,
        aec_rect.right,
        aec_rect.bottom,
        if flyout.aec_enabled { "AEC ✓" } else { "AEC" },
        DT_CENTER_V,
    );

    let gate_w = 56;
    let gate_left = 14 + aec_w + 8;
    let gate_rect = rect(gate_left, y, gate_left + gate_w, y + 22);
    flyout
        .button_rects
        .push((FlyoutButton::ToggleGate, gate_rect));
    let gate_hover = flyout.hover_btn == Some(FlyoutButton::ToggleGate);

    let (gate_bg, gate_bd, gate_fg) = if flyout.dsp_gate_enabled {
        (
            if gate_hover {
                COLOR_BTN_HOVER
            } else {
                COLOR_BTN_BG
            },
            COLOR_MIC_ON,
            COLOR_MIC_ON,
        )
    } else if gate_hover {
        (COLOR_BTN_HOVER, COLOR_BTN_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (COLOR_BTN_BG, COLOR_BTN_BORDER, COLOR_TEXT_SECONDARY)
    };
    draw_pill_bg(
        dc,
        gate_rect.left,
        gate_rect.top,
        gate_rect.right,
        gate_rect.bottom,
        gate_bg,
        gate_bd,
    );
    draw_text(
        dc,
        font_body_bold,
        gate_fg,
        gate_rect.left,
        gate_rect.top,
        gate_rect.right,
        gate_rect.bottom,
        if flyout.dsp_gate_enabled {
            "Gate ✓"
        } else {
            "Gate"
        },
        DT_CENTER_V,
    );

    y + 26
}
