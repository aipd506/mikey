//! Header and status badge rendering for the Mikey Flyout.
//! Ultra-compact, web-style minimal header with smooth pill badge.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_header(
    dc: win32::HDC,
    flyout: &FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
) -> i32 {
    let is_active = flyout.session_manager.is_active();
    let active_session = flyout.session_manager.active_session();
    let pending_devices = flyout.session_manager.list_pending();

    // ── Header: "Mikey" title with app icon ──
    let icon_color = if is_active {
        ARGB_MIC_ON
    } else if !pending_devices.is_empty() {
        ARGB_STATUS_WAIT
    } else {
        ARGB_TEXT_PRIMARY
    };

    if let Some(g) = g_opt {
        super::heroicons::draw_hero_mic(g, graphics, 28.0, 24.0, icon_color, false);
    }

    draw_text(
        dc,
        fonts.title,
        COLOR_TEXT_PRIMARY,
        44,
        14,
        120,
        34,
        "Mikey",
        0,
    );

    // Connected device subtitle
    let (dev_text, dev_color) = if let Some(session) = &active_session {
        (
            session.device_name.chars().take(20).collect::<String>(),
            COLOR_TEXT_SECONDARY,
        )
    } else if !pending_devices.is_empty() {
        ("Join request...".to_string(), COLOR_STATUS_WAIT)
    } else {
        ("Ready to connect".to_string(), COLOR_TEXT_MUTED)
    };
    draw_text(dc, fonts.body, dev_color, 44, 34, 200, 50, &dev_text, 0);

    // ── Smooth Status Badge Pill (top-right) ──
    let (badge_text, dot_color) = if is_active {
        let lvl = active_session
            .as_ref()
            .map(|s| s.current_level)
            .unwrap_or(1);
        let name = match lvl {
            1 | 2 => "USB",
            3 => "BT",
            4 => "Wi-Fi",
            _ => "Live",
        };
        (name, ARGB_MIC_ON)
    } else if !pending_devices.is_empty() {
        ("Wait", ARGB_STATUS_WAIT)
    } else {
        ("Idle", ARGB_TEXT_MUTED)
    };

    let pill_w = 68.0;
    let pill_h = 24.0;
    let pill_r = 12.0;
    let pill_left = (FLYOUT_WIDTH - 16) as f32 - pill_w;
    let pill_top = 16.0;

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            pill_left,
            pill_top,
            pill_w,
            pill_h,
            pill_r,
            ARGB_PILL,
            ARGB_BORDER,
        );
        // Smooth glowing status dot
        draw_smooth_circle(
            g,
            graphics,
            pill_left + 12.0,
            pill_top + 12.0,
            3.5,
            dot_color,
        );
    }

    draw_text(
        dc,
        fonts.body_bold,
        if is_active {
            COLOR_TEXT_PRIMARY
        } else {
            COLOR_TEXT_SECONDARY
        },
        (pill_left + 22.0) as i32,
        pill_top as i32,
        (pill_left + pill_w - 4.0) as i32,
        (pill_top + pill_h) as i32,
        badge_text,
        DT_CENTER_V,
    );

    56
}
