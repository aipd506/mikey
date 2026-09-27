//! Notification banners: pending join request and virtual mic setup warning.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_banners(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_body: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
    mut y: i32,
) -> i32 {
    let pending_devices = flyout.session_manager.list_pending();

    // ── Pending join request banner ──
    if let Some(pending) = pending_devices.first() {
        y += 4;
        draw_pill_bg(
            dc,
            16,
            y,
            FLYOUT_WIDTH - 16,
            y + 34,
            MONO_DARKER,
            MONO_BTN_BORDER,
        );
        let ptext = format!("Join: {}", pending.device_name);
        draw_text(
            dc,
            font_body_bold,
            MONO_LIGHT,
            24,
            y + 6,
            200,
            y + 28,
            &ptext,
            0,
        );

        let allow_rect = rect(206, y + 5, 260, y + 29);
        flyout
            .button_rects
            .push((FlyoutButton::AllowJoin(pending.request_id), allow_rect));
        draw_pill_bg(
            dc,
            allow_rect.left,
            allow_rect.top,
            allow_rect.right,
            allow_rect.bottom,
            MONO_WHITE,
            MONO_WHITE,
        );
        draw_text(
            dc,
            font_body_bold,
            MONO_BLACK,
            allow_rect.left,
            allow_rect.top,
            allow_rect.right,
            allow_rect.bottom,
            "Allow",
            DT_CENTER_V,
        );

        let deny_rect = rect(266, y + 5, 314, y + 29);
        flyout
            .button_rects
            .push((FlyoutButton::DenyJoin(pending.request_id), deny_rect));
        draw_pill_bg(
            dc,
            deny_rect.left,
            deny_rect.top,
            deny_rect.right,
            deny_rect.bottom,
            MONO_DARKER,
            MONO_DIM,
        );
        draw_text(
            dc,
            font_body_bold,
            MONO_LIGHT,
            deny_rect.left,
            deny_rect.top,
            deny_rect.right,
            deny_rect.bottom,
            "Deny",
            DT_CENTER_V,
        );
        y += 38;
    }

    // ── Virtual mic setup warning ──
    let (virt_ready, _) = crate::audio::sink::check_virtual_device_status();
    if !virt_ready {
        y += 4;
        draw_text(
            dc,
            font_icon,
            MONO_LIGHT,
            16,
            y + 3,
            36,
            y + 27,
            "\u{E7BA}",
            DT_CENTER_V,
        );
        draw_text(
            dc,
            font_body,
            MONO_LIGHT,
            38,
            y + 3,
            210,
            y + 27,
            "Mikey Mic setup needed",
            0,
        );

        let btn = rect(FLYOUT_WIDTH - 16 - 90, y + 2, FLYOUT_WIDTH - 16, y + 28);
        flyout
            .button_rects
            .push((FlyoutButton::SetupVirtualMic, btn));
        let s_hover = flyout.hover_btn == Some(FlyoutButton::SetupVirtualMic);
        draw_pill_bg(
            dc,
            btn.left,
            btn.top,
            btn.right,
            btn.bottom,
            if s_hover { MONO_BTN_HOVER } else { MONO_DARKER },
            if s_hover {
                MONO_BTN_BORDER_HI
            } else {
                MONO_BTN_BORDER
            },
        );
        draw_text(
            dc,
            font_body_bold,
            MONO_LIGHT,
            btn.left,
            btn.top,
            btn.right,
            btn.bottom,
            "Setup Mic",
            DT_CENTER_V,
        );
        y += 34;
    }

    y
}
