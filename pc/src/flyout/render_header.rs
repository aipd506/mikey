//! Header and status badge rendering for the Mikey Flyout.

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
) {
    let is_active = flyout.session_manager.is_active();
    let active_session = flyout.session_manager.active_session();
    let pending_devices = flyout.session_manager.list_pending();

    // ── Header: "Mikey" title ──
    draw_text(dc, font_title, MONO_WHITE, 16, 12, 120, 34, "Mikey", 0);

    // ── Status badge pill (top-right, no dot) ──
    let (badge_text, badge_icon, brightness) = if is_active {
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
        (name, icon, MONO_WHITE)
    } else if !pending_devices.is_empty() {
        ("WAIT", "\u{E7BA}", MONO_LIGHT)
    } else {
        ("IDLE", "", MONO_DIM)
    };

    let pill_left = FLYOUT_WIDTH - 16 - 80;
    let pill_top = 12;
    let pill_right = FLYOUT_WIDTH - 16;
    let pill_bottom = 34;

    draw_pill_bg(
        dc,
        pill_left,
        pill_top,
        pill_right,
        pill_bottom,
        MONO_DARKER,
        MONO_BTN_BORDER,
    );

    let icon_start = pill_left + 8;
    if !badge_icon.is_empty() {
        draw_text(
            dc,
            font_icon,
            brightness,
            icon_start,
            pill_top,
            icon_start + 18,
            pill_bottom,
            badge_icon,
            DT_CENTER_V,
        );
    }

    let txt_l = if badge_icon.is_empty() {
        icon_start
    } else {
        icon_start + 20
    };
    draw_text(
        dc,
        font_body_bold,
        brightness,
        txt_l,
        pill_top,
        pill_right - 4,
        pill_bottom,
        badge_text,
        DT_CENTER_V,
    );

    // ── Subtitle ──
    let sub_text = if let Some(session) = &active_session {
        format!(
            "{} · Connected",
            session.device_name.chars().take(20).collect::<String>()
        )
    } else {
        "Waiting for phone...".to_string()
    };
    draw_text(
        dc,
        font_body,
        MONO_MID,
        16,
        36,
        pill_left - 8,
        52,
        &sub_text,
        0,
    );
}
