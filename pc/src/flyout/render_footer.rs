//! Footer buttons rendering: icon-only Quit and Disconnect buttons.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_footer_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_icon_lg: win32::HFONT,
    height: i32,
) {
    let is_active = flyout.session_manager.is_active();
    let footer_y = height - 38;
    let btn_size = 36;

    // Quit button (power icon only)
    let quit_rect = rect(
        FLYOUT_WIDTH - 16 - btn_size,
        footer_y,
        FLYOUT_WIDTH - 16,
        footer_y + 28,
    );
    flyout.button_rects.push((FlyoutButton::Quit, quit_rect));
    let quit_hover = flyout.hover_btn == Some(FlyoutButton::Quit);
    draw_pill_bg(
        dc,
        quit_rect.left,
        quit_rect.top,
        quit_rect.right,
        quit_rect.bottom,
        if quit_hover {
            MONO_BTN_HOVER
        } else {
            MONO_DARKER
        },
        if quit_hover {
            MONO_BTN_BORDER_HI
        } else {
            MONO_BTN_BORDER
        },
    );
    let q_fg = if quit_hover { MONO_WHITE } else { MONO_MID };
    draw_text(
        dc,
        font_icon_lg,
        q_fg,
        quit_rect.left,
        footer_y,
        quit_rect.right,
        footer_y + 28,
        "\u{E7E8}",
        DT_CENTER_V,
    );

    // Disconnect button (icon only, visible only when active)
    if is_active {
        let disc_rect = rect(
            quit_rect.left - 8 - btn_size,
            footer_y,
            quit_rect.left - 8,
            footer_y + 28,
        );
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
                MONO_BTN_HOVER
            } else {
                MONO_DARKER
            },
            if disc_hover {
                MONO_BTN_BORDER_HI
            } else {
                MONO_BTN_BORDER
            },
        );
        let d_fg = if disc_hover { MONO_WHITE } else { MONO_MID };
        draw_text(
            dc,
            font_icon_lg,
            d_fg,
            disc_rect.left,
            footer_y,
            disc_rect.right,
            footer_y + 28,
            "\u{E8CD}",
            DT_CENTER_V,
        );
    }
}
