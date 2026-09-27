//! Header and status badge rendering for the Mikey Flyout.
//! Ultra-compact, minimal header with device name and status pill.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_header(
    dc: win32::HDC,
    flyout: &FlyoutWindow,
    font_title: win32::HFONT,
    font_body: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
) -> i32 {
    let is_active = flyout.session_manager.is_active();
    let active_session = flyout.session_manager.active_session();
    let pending_devices = flyout.session_manager.list_pending();

    // ── Header: "Mikey" title ──
    draw_text(
        dc,
        font_title,
        COLOR_TEXT_PRIMARY,
        14,
        10,
        80,
        28,
        "Mikey",
        0,
    );

    // Device subtitle / status inline or directly below
    let (dev_text, dev_color) = if let Some(session) = &active_session {
        (
            session.device_name.chars().take(18).collect::<String>(),
            COLOR_TEXT_SECONDARY,
        )
    } else if !pending_devices.is_empty() {
        ("Join request...".to_string(), COLOR_STATUS_WAIT)
    } else {
        ("Waiting for phone...".to_string(), COLOR_TEXT_MUTED)
    };
    draw_text(dc, font_body, dev_color, 14, 28, 190, 42, &dev_text, 0);

    // ── Status badge pill (top-right) ──
    let (badge_text, badge_icon, badge_color) = if is_active {
        let lvl = active_session
            .as_ref()
            .map(|s| s.current_level)
            .unwrap_or(1);
        let (name, icon) = match lvl {
            1 | 2 => ("USB", "\u{E88E}"),
            3 => ("BT", "\u{E702}"),
            4 => ("WI-FI", "\u{E701}"),
            _ => ("LIVE", "\u{E88E}"),
        };
        (name, icon, COLOR_MIC_ON)
    } else if !pending_devices.is_empty() {
        ("WAIT", "\u{E7BA}", COLOR_STATUS_WAIT)
    } else {
        ("IDLE", "", COLOR_TEXT_MUTED)
    };

    let pill_w = 66;
    let pill_left = FLYOUT_WIDTH - 14 - pill_w;
    let pill_top = 11;
    let pill_right = FLYOUT_WIDTH - 14;
    let pill_bottom = 33;

    draw_pill_bg(
        dc,
        pill_left,
        pill_top,
        pill_right,
        pill_bottom,
        COLOR_BTN_BG,
        if is_active {
            COLOR_MIC_ON
        } else {
            COLOR_BTN_BORDER
        },
    );

    let mut tx_left = pill_left + 6;
    if !badge_icon.is_empty() {
        draw_text(
            dc,
            font_icon,
            badge_color,
            tx_left,
            pill_top,
            tx_left + 16,
            pill_bottom,
            badge_icon,
            DT_CENTER_V,
        );
        tx_left += 16;
    }

    draw_text(
        dc,
        font_body_bold,
        if is_active {
            COLOR_TEXT_PRIMARY
        } else {
            badge_color
        },
        tx_left,
        pill_top,
        pill_right - 4,
        pill_bottom,
        badge_text,
        DT_CENTER_V,
    );

    46
}
