//! Settings drawer with Noise Suppression slider.
//! Collapsible drawer opened via the Settings button.

#![cfg(windows)]

use super::gdi::*;
use super::heroicons::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_dsp_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    y: i32,
) -> i32 {
    if !flyout.settings_expanded {
        return y;
    }

    let card_w = FLYOUT_WIDTH - 28;
    let card_h = 66;
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

    let cy = (y + 22) as f32;

    if let Some(g) = g_opt {
        draw_hero_sound(g, graphics, 36.0, cy, ARGB_TEXT_SECONDARY);
    }

    let slider_left = 56.0;
    let slider_right = (card_left + card_w - 14) as f32;
    let slider_y = y + 10;
    let slider_touch_rect = rect(
        slider_left as i32,
        slider_y,
        slider_right as i32,
        slider_y + 24,
    );
    flyout
        .button_rects
        .push((FlyoutButton::NsSlider, slider_touch_rect));

    if let Some(g) = g_opt {
        draw_ns_slider(
            g,
            graphics,
            slider_left,
            slider_right,
            cy,
            flyout.ns_strength,
        );
    }

    let pct_str = format!(
        "Noise suppression: {}%",
        (flyout.ns_strength * 100.0) as i32
    );
    draw_text(
        dc,
        fonts.body,
        COLOR_TEXT_MUTED,
        36,
        y + 40,
        card_left + card_w - 10,
        y + 56,
        &pct_str,
        0,
    );

    y + card_h + 8
}

fn draw_ns_slider(g: &Gdiplus, graphics: *mut c_void, left: f32, right: f32, cy: f32, val: f32) {
    let track_h = 6.0;
    let r = track_h / 2.0;
    let w = right - left;
    draw_smooth_pill(
        g,
        graphics,
        left,
        cy - r,
        w,
        track_h,
        r,
        ARGB_SURFACE,
        ARGB_BORDER,
    );
    let fill_w = (w * val.clamp(0.0, 1.0)).max(r * 2.0);
    draw_smooth_pill(
        g,
        graphics,
        left,
        cy - r,
        fill_w,
        track_h,
        r,
        ARGB_MIC_ON,
        0,
    );
    let thumb_x = left + fill_w - r;
    draw_smooth_circle(g, graphics, thumb_x, cy, 7.0, ARGB_TEXT_PRIMARY);
    draw_smooth_circle(g, graphics, thumb_x, cy, 4.0, ARGB_MIC_ON);
}
