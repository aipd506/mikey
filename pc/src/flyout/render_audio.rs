//! Microphone card, Lucide vector mic, live VU meter, and mute pill.
//! Clean, ultra-minimal dark mode card.

#![cfg(windows)]

use super::gdi::*;
use super::heroicons::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_mic_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    y: i32,
) -> i32 {
    let is_active = flyout.session_manager.is_active();
    let peak_level = if flyout.is_muted {
        0.0
    } else {
        flyout.jitter_buffer.get_peak_level()
    };

    let card_w = FLYOUT_WIDTH - 28;
    let card_h = 56;
    let card_r = 14.0;
    let card_left = 14;

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            card_left as f32,
            y as f32,
            card_w as f32,
            card_h as f32,
            card_r,
            ARGB_PILL,
            ARGB_BORDER,
        );
    }

    let mic_color = if flyout.is_muted {
        ARGB_ALERT_RED
    } else if is_active {
        ARGB_MIC_ON
    } else {
        ARGB_ICON_OFF
    };
    let cy = (y + card_h / 2) as f32;

    if let Some(g) = g_opt {
        draw_hero_mic(g, graphics, 36.0, cy, mic_color, flyout.is_muted);
    }

    draw_text(
        dc,
        fonts.heading,
        COLOR_TEXT_PRIMARY,
        58,
        y + 10,
        180,
        y + 26,
        "Microphone",
        0,
    );

    let vu_left = 58.0;
    let vu_w = 110.0;
    let vu_h = 4.0;
    let vu_r = 2.0;
    let vu_y = (y + 32) as f32;

    if is_active && !flyout.is_muted {
        if let Some(g) = g_opt {
            draw_smooth_pill(g, graphics, vu_left, vu_y, vu_w, vu_h, vu_r, ARGB_BORDER, 0);
            if peak_level > 0.01 {
                let active_w = (vu_w * peak_level).clamp(vu_r * 2.0, vu_w);
                let bar_color = if peak_level > 0.95 {
                    ARGB_ALERT_RED
                } else if peak_level > 0.75 {
                    ARGB_STATUS_WAIT
                } else {
                    ARGB_MIC_ON
                };
                draw_smooth_pill(
                    g, graphics, vu_left, vu_y, active_w, vu_h, vu_r, bar_color, 0,
                );
            }
        }
    } else {
        let mic_status = if flyout.is_muted {
            "Muted"
        } else if is_active {
            "Active"
        } else {
            "Virtual Mic Ready"
        };
        let status_color = if flyout.is_muted {
            COLOR_ALERT_RED
        } else {
            COLOR_TEXT_SECONDARY
        };
        draw_text(
            dc,
            fonts.body,
            status_color,
            58,
            y + 28,
            180,
            y + 44,
            mic_status,
            0,
        );
    }

    let btn_w = 60;
    let btn_h = 26;
    let btn_r = 13.0;
    let btn_y = y + (card_h - btn_h) / 2;
    let btn_left = card_left + card_w - 10 - btn_w;
    let mute_rect = rect(btn_left, btn_y, btn_left + btn_w, btn_y + btn_h);
    flyout
        .button_rects
        .push((FlyoutButton::MuteToggle, mute_rect));
    let mute_hover = flyout.hover_btn == Some(FlyoutButton::MuteToggle);

    let (m_bg, m_bd, m_fg, m_text) = if flyout.is_muted {
        (ARGB_ALERT_RED, ARGB_ALERT_RED, COLOR_TEXT_PRIMARY, "Unmute")
    } else if mute_hover {
        (ARGB_PILL_HOVER, ARGB_BORDER_HI, COLOR_TEXT_PRIMARY, "Mute")
    } else {
        (ARGB_SURFACE, ARGB_BORDER, COLOR_TEXT_SECONDARY, "Mute")
    };

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            btn_left as f32,
            btn_y as f32,
            btn_w as f32,
            btn_h as f32,
            btn_r,
            m_bg,
            m_bd,
        );
    }
    draw_text(
        dc,
        fonts.body_bold,
        m_fg,
        btn_left,
        btn_y,
        btn_left + btn_w,
        btn_y + btn_h,
        m_text,
        DT_CENTER_V,
    );

    y + card_h + 8
}
