//! Camera live preview, persistent preview window toggle, and flip camera controls.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;

pub fn render_video_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_heading: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
    y: i32,
) -> i32 {
    let is_cam_on = flyout.video_pipeline.is_camera_on();
    let is_prev_vis = flyout.video_pipeline.is_preview_visible();
    let is_active = flyout.session_manager.is_active();
    let last_frame = flyout.video_pipeline.get_last_frame();

    let (cam_label, cam_color) = if is_cam_on {
        ("Camera Live", COLOR_CAM_ON)
    } else {
        ("Camera", COLOR_ICON_OFF)
    };

    // Camera icon and label
    draw_text(
        dc,
        font_icon,
        cam_color,
        14,
        y,
        32,
        y + 22,
        "\u{E714}",
        DT_CENTER_V,
    );
    let lbl_fg = if is_cam_on {
        COLOR_TEXT_PRIMARY
    } else {
        COLOR_TEXT_SECONDARY
    };
    draw_text(dc, font_heading, lbl_fg, 36, y, 140, y + 22, cam_label, 0);

    let mut rx = FLYOUT_WIDTH - 14;

    // Persistent Preview button (the previous camera option: always available)
    let prev_w = 66;
    let prev_rect = rect(rx - prev_w, y, rx, y + 22);
    flyout
        .button_rects
        .push((FlyoutButton::TogglePreview, prev_rect));
    let prev_hover = flyout.hover_btn == Some(FlyoutButton::TogglePreview);
    let (p_bg, p_bd, p_fg) = if is_prev_vis {
        (COLOR_CAM_ON, COLOR_CAM_ON, COLOR_TEXT_PRIMARY)
    } else if prev_hover {
        (COLOR_BTN_HOVER, COLOR_BTN_BORDER_HI, COLOR_TEXT_PRIMARY)
    } else {
        (COLOR_BTN_BG, COLOR_BTN_BORDER, COLOR_TEXT_SECONDARY)
    };
    draw_pill_bg(
        dc,
        prev_rect.left,
        prev_rect.top,
        prev_rect.right,
        prev_rect.bottom,
        p_bg,
        p_bd,
    );
    draw_text(
        dc,
        font_body_bold,
        p_fg,
        prev_rect.left,
        prev_rect.top,
        prev_rect.right,
        prev_rect.bottom,
        "Preview",
        DT_CENTER_V,
    );
    rx -= prev_w + 6;

    // Flip camera button (visible when session is active)
    if is_active {
        let flip_w = 32;
        let flip_rect = rect(rx - flip_w, y, rx, y + 22);
        flyout
            .button_rects
            .push((FlyoutButton::FlipCamera, flip_rect));
        let flip_hover = flyout.hover_btn == Some(FlyoutButton::FlipCamera);
        let f_bg = if flip_hover {
            COLOR_BTN_HOVER
        } else {
            COLOR_BTN_BG
        };
        let f_bd = if flip_hover {
            COLOR_BTN_BORDER_HI
        } else {
            COLOR_BTN_BORDER
        };
        let f_fg = if flip_hover {
            COLOR_TEXT_PRIMARY
        } else {
            COLOR_TEXT_SECONDARY
        };
        draw_pill_bg(
            dc,
            flip_rect.left,
            flip_rect.top,
            flip_rect.right,
            flip_rect.bottom,
            f_bg,
            f_bd,
        );
        draw_text(
            dc,
            font_icon,
            f_fg,
            flip_rect.left,
            flip_rect.top,
            flip_rect.right,
            flip_rect.bottom,
            "\u{E72C}",
            DT_CENTER_V,
        );
    }

    // Embedded 16:9 thumbnail preview (if camera on and detached window not popped out)
    if is_cam_on && !is_prev_vis {
        if let Some(frame) = last_frame {
            let prev_top = y + 26;
            super::blit::blit_camera_preview(dc, &frame, prev_top);
            return prev_top + 100 + 10;
        }
    }

    y + 26 + 8
}
