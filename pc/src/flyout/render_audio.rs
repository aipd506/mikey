//! Microphone status, mute toggle button, and live VU meter rendering.
//! Ultra-compact, visual-first audio controls.

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

    let (mic_label, mic_glyph, mic_color, label_color) = if flyout.is_muted {
        ("Mic Muted", "\u{E74F}", COLOR_ALERT_RED, COLOR_ALERT_RED)
    } else if is_active {
        ("Microphone", "\u{E720}", COLOR_MIC_ON, COLOR_TEXT_PRIMARY)
    } else {
        (
            "Microphone",
            "\u{E720}",
            COLOR_ICON_OFF,
            COLOR_TEXT_SECONDARY,
        )
    };

    // Mic icon
    draw_text(
        dc,
        font_icon,
        mic_color,
        14,
        y,
        32,
        y + 22,
        mic_glyph,
        DT_CENTER_V,
    );

    // Mic label
    draw_text(
        dc,
        font_heading,
        label_color,
        36,
        y,
        190,
        y + 22,
        mic_label,
        0,
    );

    // Mute button pill
    let mute_w = 60;
    let mute_rect = rect(FLYOUT_WIDTH - 14 - mute_w, y, FLYOUT_WIDTH - 14, y + 22);
    flyout
        .button_rects
        .push((FlyoutButton::MuteToggle, mute_rect));
    let mute_hover = flyout.hover_btn == Some(FlyoutButton::MuteToggle);

    let (mute_bg, mute_bd, mute_fg) = if flyout.is_muted {
        (COLOR_ALERT_RED, COLOR_ALERT_RED, COLOR_TEXT_PRIMARY)
    } else if mute_hover {
        (COLOR_BTN_HOVER, COLOR_BTN_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (COLOR_BTN_BG, COLOR_BTN_BORDER, COLOR_TEXT_SECONDARY)
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

    // VU meter bar (integrated 4px line)
    let vu_top = y + 26;
    let vu_w = FLYOUT_WIDTH - 28;
    draw_pill_bg(
        dc,
        14,
        vu_top,
        14 + vu_w,
        vu_top + 4,
        COLOR_BORDER,
        COLOR_BORDER,
    );

    if is_active && !flyout.is_muted && peak_level > 0.01 {
        let active_w = ((vu_w as f32 * peak_level) as i32).clamp(4, vu_w);
        let bar_color = if peak_level > 0.85 {
            COLOR_ALERT_RED
        } else if peak_level > 0.65 {
            COLOR_STATUS_WAIT
        } else {
            COLOR_MIC_ON
        };
        let fill = unsafe { win32::CreateSolidBrush(bar_color) };
        let fill_p = unsafe { win32::CreatePen(win32::PS_SOLID, 1, bar_color) };
        unsafe {
            win32::SelectObject(dc, fill);
            win32::SelectObject(dc, fill_p);
            win32::RoundRect(dc, 14, vu_top, 14 + active_w, vu_top + 4, 3, 3);
            win32::DeleteObject(fill);
            win32::DeleteObject(fill_p);
        }
    }

    vu_top + 4 + 10
}
