//! Footer buttons rendering: icon-only Logs, Disconnect, and Quit buttons.
//! Ultra-compact bottom toolbar.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_footer_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_body: win32::HFONT,
    font_icon: win32::HFONT,
    height: i32,
) {
    let is_active = flyout.session_manager.is_active();
    let footer_y = height - 28;
    let btn_w = 28;
    let btn_h = 20;

    // Left: version label
    draw_text(
        dc,
        font_body,
        COLOR_TEXT_MUTED,
        14,
        footer_y,
        100,
        footer_y + btn_h,
        "Mikey v1.0",
        DT_CENTER_V,
    );

    let mut right_x = FLYOUT_WIDTH - 14;

    // Quit button (power icon)
    let quit_rect = rect(right_x - btn_w, footer_y, right_x, footer_y + btn_h);
    flyout.button_rects.push((FlyoutButton::Quit, quit_rect));
    let quit_hover = flyout.hover_btn == Some(FlyoutButton::Quit);
    draw_pill_bg(
        dc,
        quit_rect.left,
        quit_rect.top,
        quit_rect.right,
        quit_rect.bottom,
        if quit_hover {
            COLOR_BTN_HOVER
        } else {
            COLOR_BTN_BG
        },
        if quit_hover {
            COLOR_BTN_BORDER_HI
        } else {
            COLOR_BTN_BORDER
        },
    );
    draw_text(
        dc,
        font_icon,
        if quit_hover {
            COLOR_ALERT_RED
        } else {
            COLOR_TEXT_SECONDARY
        },
        quit_rect.left,
        quit_rect.top,
        quit_rect.right,
        quit_rect.bottom,
        "\u{E7E8}",
        DT_CENTER_V,
    );
    right_x -= btn_w + 6;

    // Logs button (folder icon)
    let logs_rect = rect(right_x - btn_w, footer_y, right_x, footer_y + btn_h);
    flyout
        .button_rects
        .push((FlyoutButton::OpenLogs, logs_rect));
    let logs_hover = flyout.hover_btn == Some(FlyoutButton::OpenLogs);
    draw_pill_bg(
        dc,
        logs_rect.left,
        logs_rect.top,
        logs_rect.right,
        logs_rect.bottom,
        if logs_hover {
            COLOR_BTN_HOVER
        } else {
            COLOR_BTN_BG
        },
        if logs_hover {
            COLOR_BTN_BORDER_HI
        } else {
            COLOR_BTN_BORDER
        },
    );
    draw_text(
        dc,
        font_icon,
        if logs_hover {
            COLOR_TEXT_PRIMARY
        } else {
            COLOR_TEXT_SECONDARY
        },
        logs_rect.left,
        logs_rect.top,
        logs_rect.right,
        logs_rect.bottom,
        "\u{ED25}",
        DT_CENTER_V,
    );
    right_x -= btn_w + 6;

    // Disconnect button (visible when active)
    if is_active {
        let disc_rect = rect(right_x - btn_w, footer_y, right_x, footer_y + btn_h);
        flyout
            .button_rects
            .push((FlyoutButton::Disconnect, disc_rect));
        let disc_hover = flyout.hover_btn == Some(FlyoutButton::Disconnect);
        draw_pill_bg(
            dc,
            disc_rect.left,
            disc_rect.top,
            disc_rect.right,
            disc_rect.bottom,
            if disc_hover {
                COLOR_BTN_HOVER
            } else {
                COLOR_BTN_BG
            },
            if disc_hover {
                COLOR_BTN_BORDER_HI
            } else {
                COLOR_BTN_BORDER
            },
        );
        draw_text(
            dc,
            font_icon,
            if disc_hover {
                COLOR_TEXT_PRIMARY
            } else {
                COLOR_TEXT_SECONDARY
            },
            disc_rect.left,
            disc_rect.top,
            disc_rect.right,
            disc_rect.bottom,
            "\u{E8CD}",
            DT_CENTER_V,
        );
    }
}
