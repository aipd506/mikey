//! Settings drawer with Noise Suppression slider and Gate toggle.
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

    let gate_w = 56;
    let gate_h = 24;
    let gate_r = 12.0;
    let gate_left = card_left + card_w - 10 - gate_w;
    let gate_y = y + 10;
    let gate_rect = rect(gate_left, gate_y, gate_left + gate_w, gate_y + gate_h);
    flyout
        .button_rects
        .push((FlyoutButton::ToggleGate, gate_rect));
    let gate_hover = flyout.hover_btn == Some(FlyoutButton::ToggleGate);

    let (gate_bg, gate_bd, gate_fg) = if flyout.dsp_gate_enabled {
        (
            if gate_hover {
                ARGB_PILL_HOVER
            } else {
                ARGB_SURFACE
            },
            ARGB_MIC_ON,
            COLOR_MIC_ON,
        )
    } else if gate_hover {
        (ARGB_PILL_HOVER, ARGB_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (ARGB_SURFACE, ARGB_BORDER, COLOR_TEXT_SECONDARY)
    };

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            gate_left as f32,
            gate_y as f32,
            gate_w as f32,
            gate_h as f32,
            gate_r,
            gate_bg,
            gate_bd,
        );
    }
    draw_text(
        dc,
        fonts.body_bold,
        gate_fg,
        gate_left,
        gate_y,
        gate_left + gate_w,
        gate_y + gate_h,
        "Gate",
        DT_CENTER_V,
    );

    let slider_left = 56.0;
    let slider_right = (gate_left - 10) as f32;
    let slider_touch_rect = rect(
        slider_left as i32,
        gate_y,
        slider_right as i32,
        gate_y + gate_h,
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
