//! Footer toolbar: version, Settings expander button, and circular action icons.

#![cfg(windows)]

use super::gdi::*;
use super::heroicons::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_footer_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    height: i32,
) {
    let is_active = flyout.session_manager.is_active();
    let footer_y = height - 34;
    let btn_size = 28;

    // Left: version
    draw_text(
        dc,
        fonts.body,
        COLOR_TEXT_MUTED,
        16,
        footer_y,
        90,
        footer_y + btn_size,
        concat!("Mikey v", env!("CARGO_PKG_VERSION")),
        DT_CENTER_V,
    );

    let mut right_x = FLYOUT_WIDTH - 14;

    // 1. Quit Button
    let q_rect = rect(right_x - btn_size, footer_y, right_x, footer_y + btn_size);
    flyout.button_rects.push((FlyoutButton::Quit, q_rect));
    let q_hover = flyout.hover_btn == Some(FlyoutButton::Quit);
    if let Some(g) = g_opt {
        draw_circle_btn(g, graphics, right_x, footer_y, btn_size, q_hover);
        let c = if q_hover {
            ARGB_ALERT_RED
        } else {
            ARGB_TEXT_SECONDARY
        };
        draw_hero_power(
            g,
            graphics,
            (right_x - btn_size / 2) as f32,
            (footer_y + btn_size / 2) as f32,
            c,
        );
    }
    right_x -= btn_size + 6;

    // 2. Open Logs Button
    let l_rect = rect(right_x - btn_size, footer_y, right_x, footer_y + btn_size);
    flyout.button_rects.push((FlyoutButton::OpenLogs, l_rect));
    let l_hover = flyout.hover_btn == Some(FlyoutButton::OpenLogs);
    if let Some(g) = g_opt {
        draw_circle_btn(g, graphics, right_x, footer_y, btn_size, l_hover);
        let c = if l_hover {
            ARGB_TEXT_PRIMARY
        } else {
            ARGB_TEXT_SECONDARY
        };
        draw_hero_folder(
            g,
            graphics,
            (right_x - btn_size / 2) as f32,
            (footer_y + btn_size / 2) as f32,
            c,
        );
    }
    right_x -= btn_size + 6;

    // 3. Disconnect Button (if active)
    if is_active {
        let d_rect = rect(right_x - btn_size, footer_y, right_x, footer_y + btn_size);
        flyout.button_rects.push((FlyoutButton::Disconnect, d_rect));
        let d_hover = flyout.hover_btn == Some(FlyoutButton::Disconnect);
        if let Some(g) = g_opt {
            draw_circle_btn(g, graphics, right_x, footer_y, btn_size, d_hover);
            let c = if d_hover {
                ARGB_ALERT_RED
            } else {
                ARGB_TEXT_SECONDARY
            };
            draw_hero_disconnect(
                g,
                graphics,
                (right_x - btn_size / 2) as f32,
                (footer_y + btn_size / 2) as f32,
                c,
            );
        }
        right_x -= btn_size + 6;
    }

    // 4. Settings Drawer Toggle Pill Button
    let set_w = 78;
    let set_h = 24;
    let set_y = footer_y + 2;
    let set_rect = rect(right_x - set_w, set_y, right_x, set_y + set_h);
    flyout
        .button_rects
        .push((FlyoutButton::ToggleAdvanced, set_rect));
    let set_hover = flyout.hover_btn == Some(FlyoutButton::ToggleAdvanced);
    let (s_bg, s_bd, s_fg) = if flyout.settings_expanded {
        (ARGB_PILL_HOVER, ARGB_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else if set_hover {
        (ARGB_PILL_HOVER, ARGB_BORDER, COLOR_TEXT_PRIMARY)
    } else {
        (ARGB_PILL, ARGB_BORDER, COLOR_TEXT_SECONDARY)
    };
    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            (right_x - set_w) as f32,
            set_y as f32,
            set_w as f32,
            set_h as f32,
            12.0,
            s_bg,
            s_bd,
        );
    }
    let s_label = if flyout.settings_expanded {
        "Settings ˄"
    } else {
        "Settings ⌵"
    };
    draw_text(
        dc,
        fonts.body_bold,
        s_fg,
        right_x - set_w,
        set_y,
        right_x,
        set_y + set_h,
        s_label,
        DT_CENTER_V,
    );
}

fn draw_circle_btn(g: &Gdiplus, graphics: *mut c_void, rx: i32, y: i32, size: i32, hover: bool) {
    let r = size as f32 / 2.0;
    let bg = if hover { ARGB_PILL_HOVER } else { ARGB_PILL };
    let bd = if hover { ARGB_BORDER_HI } else { ARGB_BORDER };
    draw_smooth_pill(
        g,
        graphics,
        (rx - size) as f32,
        y as f32,
        size as f32,
        size as f32,
        r,
        bg,
        bd,
    );
}
