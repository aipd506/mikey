//! Camera card, vector Lucide video icon, preview window toggle, flip lens pill, and live preview.
//! Clean, ultra-minimal dark mode card.

#![cfg(windows)]

use super::blit::render_embedded_preview;
use super::gdi::*;
use super::heroicons::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_video_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    y: i32,
) -> i32 {
    let is_cam_on = flyout.video_pipeline.is_camera_on();
    let is_prev_vis = flyout.video_pipeline.is_preview_visible();
    let is_active = flyout.session_manager.is_active();
    let last_frame = flyout.video_pipeline.get_last_frame();

    let card_w = FLYOUT_WIDTH - 28;
    let card_h = 56;
    let card_left = 14;

    if let Some(g) = g_opt {
        draw_btn_pill(
            g,
            graphics,
            [card_left, y, card_w, card_h],
            ARGB_PILL,
            ARGB_BORDER,
        );
    }

    let cam_color = if is_cam_on {
        ARGB_CAM_ON
    } else {
        ARGB_ICON_OFF
    };
    let cy = (y + card_h / 2) as f32;

    if let Some(g) = g_opt {
        draw_hero_camera(g, graphics, 36.0, cy, cam_color);
    }

    draw_text(
        dc,
        fonts.heading,
        COLOR_TEXT_PRIMARY,
        58,
        y + 10,
        170,
        y + 26,
        "Camera",
        0,
    );

    let cam_status = if is_cam_on {
        if let Some(f) = &last_frame {
            if f.height >= 1080 {
                "Active • 1080p"
            } else {
                "Active • 720p"
            }
        } else {
            "Active • 720p"
        }
    } else if is_active {
        "Connected • Camera Standby"
    } else {
        "Virtual Camera Ready"
    };

    let status_color = if is_cam_on {
        COLOR_CAM_ON
    } else {
        COLOR_TEXT_SECONDARY
    };

    draw_text(
        dc,
        fonts.body,
        status_color,
        58,
        y + 28,
        170,
        y + 44,
        cam_status,
        0,
    );

    let btn_h = 26;
    let btn_y = y + (card_h - btn_h) / 2;
    let mut rx = card_left + card_w - 10;

    let prev_w = 66;
    let prev_rect = rect(rx - prev_w, btn_y, rx, btn_y + btn_h);
    flyout
        .button_rects
        .push((FlyoutButton::TogglePreview, prev_rect));
    let prev_hover = flyout.hover_btn == Some(FlyoutButton::TogglePreview);

    let (p_bg, p_bd, p_fg) = if is_prev_vis {
        (ARGB_CAM_ON, ARGB_CAM_ON, COLOR_TEXT_PRIMARY)
    } else if prev_hover {
        (ARGB_PILL_HOVER, ARGB_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (ARGB_SURFACE, ARGB_BORDER, COLOR_TEXT_SECONDARY)
    };

    if let Some(g) = g_opt {
        draw_btn_pill(g, graphics, [rx - prev_w, btn_y, prev_w, btn_h], p_bg, p_bd);
    }
    draw_text(
        dc,
        fonts.body_bold,
        p_fg,
        rx - prev_w,
        btn_y,
        rx,
        btn_y + btn_h,
        "Preview",
        DT_CENTER_V,
    );
    rx -= prev_w + 6;

    if is_active {
        let flip_w = 34;
        let flip_rect = rect(rx - flip_w, btn_y, rx, btn_y + btn_h);
        flyout
            .button_rects
            .push((FlyoutButton::FlipCamera, flip_rect));
        let flip_hover = flyout.hover_btn == Some(FlyoutButton::FlipCamera);
        let f_bg = if flip_hover {
            ARGB_PILL_HOVER
        } else {
            ARGB_SURFACE
        };
        let f_bd = if flip_hover {
            ARGB_BORDER_HI
        } else {
            ARGB_BORDER
        };

        if let Some(g) = g_opt {
            draw_btn_pill(g, graphics, [rx - flip_w, btn_y, flip_w, btn_h], f_bg, f_bd);
            let flip_icon_color = if flip_hover {
                ARGB_TEXT_PRIMARY
            } else {
                ARGB_TEXT_SECONDARY
            };
            draw_hero_flip(g, graphics, (rx - flip_w / 2) as f32, cy, flip_icon_color);
        }
    }

    if is_cam_on && !is_prev_vis {
        if let Some(frame) = last_frame {
            let prev_top = y + card_h + 8;
            return render_embedded_preview(dc, flyout, g_opt, graphics, fonts, &frame, prev_top);
        }
    }

    y + card_h + 8
}

fn draw_btn_pill(g: &Gdiplus, graphics: *mut c_void, b: [i32; 4], bg: u32, bd: u32) {
    draw_smooth_pill(
        g,
        graphics,
        b[0] as f32,
        b[1] as f32,
        b[2] as f32,
        b[3] as f32,
        b[3] as f32 / 2.0,
        bg,
        bd,
    );
}
