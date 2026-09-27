//! Microphone status, mute toggle button, and live VU meter rendering.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_mic_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_heading: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
    y: i32,
) -> i32 {
    let is_active = flyout.session_manager.is_active();
    let peak_level = if flyout.is_muted {
        0.0
    } else {
        flyout.jitter_buffer.get_peak_level()
    };

    let (mic_label, mic_glyph, mic_bright) = if flyout.is_muted {
        ("Mic Muted", "\u{E74F}", MONO_DIM)
    } else if is_active {
        ("Microphone Live", "\u{E720}", MONO_WHITE)
    } else {
        ("Microphone Idle", "\u{E720}", MONO_DIM)
    };

    // Mic icon
    draw_text(
        dc,
        font_icon,
        mic_bright,
        16,
        y,
        36,
        y + 26,
        mic_glyph,
        DT_CENTER_V,
    );

    // Mic label
    let label_bright = if flyout.is_muted {
        MONO_DIM
    } else if is_active {
        MONO_WHITE
    } else {
        MONO_MID
    };
    draw_text(
        dc,
        font_heading,
        label_bright,
        40,
        y,
        220,
        y + 26,
        mic_label,
        0,
    );

    // Mute button pill
    let mute_rect = rect(FLYOUT_WIDTH - 16 - 76, y, FLYOUT_WIDTH - 16, y + 26);
    flyout
        .button_rects
        .push((FlyoutButton::MuteToggle, mute_rect));
    let mute_hover = flyout.hover_btn == Some(FlyoutButton::MuteToggle);
    let (mute_bg, mute_bd, mute_fg) = if flyout.is_muted {
        (MONO_WHITE, MONO_WHITE, MONO_BLACK)
    } else if mute_hover {
        (MONO_BTN_HOVER, MONO_BTN_BORDER_HI, MONO_WHITE)
    } else {
        (MONO_BTN_BG, MONO_BTN_BORDER, MONO_LIGHT)
    };
    draw_pill_bg(
        dc,
        mute_rect.left,
        mute_rect.top,
        mute_rect.right,
        mute_rect.bottom,
        mute_bg,
        mute_bd,
    );
    draw_text(
        dc,
        font_body_bold,
        mute_fg,
        mute_rect.left,
        mute_rect.top,
        mute_rect.right,
        mute_rect.bottom,
        if flyout.is_muted { "Unmute" } else { "Mute" },
        DT_CENTER_V,
    );

    // VU meter bar
    let vu_top = y + 32;
    let vu_w = FLYOUT_WIDTH - 32;
    draw_pill_bg(dc, 16, vu_top, 16 + vu_w, vu_top + 5, MONO_DARK, MONO_DARK);
    if is_active && !flyout.is_muted && peak_level > 0.01 {
        let active_w = ((vu_w as f32 * peak_level) as i32).clamp(4, vu_w);
        let fill = unsafe { win32::CreateSolidBrush(MONO_WHITE) };
        let fill_p = unsafe { win32::CreatePen(win32::PS_SOLID, 1, MONO_WHITE) };
        unsafe {
            win32::SelectObject(dc, fill);
            win32::SelectObject(dc, fill_p);
            win32::RoundRect(dc, 16, vu_top, 16 + active_w, vu_top + 5, 4, 4);
            win32::DeleteObject(fill);
            win32::DeleteObject(fill_p);
        }
    }

    vu_top + 5 + 14
}
