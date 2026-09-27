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
        y += 2;
        draw_pill_bg(
            dc,
            14,
            y,
            FLYOUT_WIDTH - 14,
            y + 30,
            COLOR_BTN_BG,
            COLOR_STATUS_WAIT,
        );
        let ptext = format!(
            "Join: {}",
            pending.device_name.chars().take(12).collect::<String>()
        );
        draw_text(
            dc,
            font_body_bold,
            COLOR_STATUS_WAIT,
            20,
            y + 4,
            170,
            y + 26,
            &ptext,
            0,
        );

        let allow_rect = rect(
            FLYOUT_WIDTH - 14 - 104,
            y + 4,
            FLYOUT_WIDTH - 14 - 56,
            y + 26,
        );
        flyout
            .button_rects
            .push((FlyoutButton::AllowJoin(pending.request_id), allow_rect));
        draw_pill_bg(
            dc,
            allow_rect.left,
            allow_rect.top,
            allow_rect.right,
            allow_rect.bottom,
            COLOR_MIC_ON,
            COLOR_MIC_ON,
        );
        draw_text(
            dc,
            font_body_bold,
            COLOR_TEXT_PRIMARY,
            allow_rect.left,
            allow_rect.top,
            allow_rect.right,
            allow_rect.bottom,
            "Allow",
            DT_CENTER_V,
        );

        let deny_rect = rect(FLYOUT_WIDTH - 14 - 50, y + 4, FLYOUT_WIDTH - 14 - 6, y + 26);
        flyout
            .button_rects
            .push((FlyoutButton::DenyJoin(pending.request_id), deny_rect));
        draw_pill_bg(
            dc,
            deny_rect.left,
            deny_rect.top,
            deny_rect.right,
            deny_rect.bottom,
            COLOR_BTN_BG,
            COLOR_BTN_BORDER,
        );
        draw_text(
            dc,
            font_body_bold,
            COLOR_TEXT_SECONDARY,
            deny_rect.left,
            deny_rect.top,
            deny_rect.right,
            deny_rect.bottom,
            "Deny",
            DT_CENTER_V,
        );
        y += 34;
    }

    // ── Virtual mic setup warning ──
    let (virt_ready, _) = crate::audio::sink::check_virtual_device_status();
    if !virt_ready {
        y += 2;
        draw_text(
            dc,
            font_icon,
            COLOR_STATUS_WAIT,
            14,
            y + 2,
            32,
            y + 24,
            "\u{E7BA}",
            DT_CENTER_V,
        );
        draw_text(
            dc,
            font_body,
            COLOR_TEXT_PRIMARY,
            34,
            y + 2,
            180,
            y + 24,
            "Mic setup needed",
            0,
        );

        let btn_w = 76;
        let btn = rect(FLYOUT_WIDTH - 14 - btn_w, y + 2, FLYOUT_WIDTH - 14, y + 24);
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
            if s_hover {
                COLOR_BTN_HOVER
            } else {
                COLOR_BTN_BG
            },
            if s_hover {
                COLOR_BTN_BORDER_HI
            } else {
                COLOR_BTN_BORDER
            },
        );
        draw_text(
            dc,
            font_body_bold,
            COLOR_TEXT_PRIMARY,
            btn.left,
            btn.top,
            btn.right,
            btn.bottom,
            "Setup Mic",
            DT_CENTER_V,
        );
        y += 28;
    }

    y
}
