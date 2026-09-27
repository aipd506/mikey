//! Noise suppression slider and echo cancellation toggle controls.

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
    // ── Noise Suppression slider ──
    let ns_label_w = 120;
    draw_text(
        dc,
        font_icon,
        MONO_MID,
        16,
        y,
        34,
        y + 24,
        "\u{E767}",
        DT_CENTER_V,
    );
    draw_text(
        dc,
        font_body,
        MONO_MID,
        36,
        y,
        36 + ns_label_w,
        y + 24,
        "Noise Suppr.",
        0,
    );

    let slider_left = 36 + ns_label_w + 4;
    let slider_right = FLYOUT_WIDTH - 16;
    let slider_cy = y + 12;
    let slider_touch_rect = rect(slider_left, y, slider_right, y + 24);
    flyout
        .button_rects
        .push((FlyoutButton::NsSlider, slider_touch_rect));
    draw_slider(dc, slider_left, slider_right, slider_cy, flyout.ns_strength);

    y += 28;

    // ── Echo Cancellation toggle (ON by default) ──
    let aec_bright = if flyout.aec_enabled {
        MONO_WHITE
    } else {
        MONO_MID
    };
    draw_text(
        dc,
        font_icon,
        aec_bright,
        16,
        y,
        34,
        y + 24,
        "\u{E7F6}",
        DT_CENTER_V,
    );
    draw_text(
        dc,
        font_body,
        aec_bright,
        36,
        y,
        160,
        y + 24,
        "Echo Cancel.",
        0,
    );

    let aec_rect = rect(FLYOUT_WIDTH - 16 - 54, y, FLYOUT_WIDTH - 16, y + 24);
    flyout
        .button_rects
        .push((FlyoutButton::ToggleAec, aec_rect));
    let aec_hover = flyout.hover_btn == Some(FlyoutButton::ToggleAec);
    let (aec_bg, aec_bd, aec_fg) = if flyout.aec_enabled {
        (
            if aec_hover { MONO_WHITE } else { MONO_LIGHT },
            MONO_WHITE,
            MONO_BLACK,
        )
    } else if aec_hover {
        (MONO_BTN_HOVER, MONO_BTN_BORDER_HI, MONO_LIGHT)
    } else {
        (MONO_DARKER, MONO_BORDER, MONO_MID)
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
        if flyout.aec_enabled { "On" } else { "Off" },
        DT_CENTER_V,
    );

    y + 28
}
