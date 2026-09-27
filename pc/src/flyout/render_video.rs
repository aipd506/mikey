//! Camera live preview, pop-out button, and camera-off indicator.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::window::FlyoutWindow;
use std::ffi::c_void;

pub fn render_video_section(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    font_heading: win32::HFONT,
    font_body: win32::HFONT,
    font_body_bold: win32::HFONT,
    font_icon: win32::HFONT,
    y: i32,
) {
    let is_cam_on = flyout.video_pipeline.is_camera_on();
    let last_frame = flyout.video_pipeline.get_last_frame();

    if let (true, Some(frame)) = (is_cam_on, last_frame) {
        draw_text(
            dc,
            font_icon,
            MONO_WHITE,
            16,
            y,
            34,
            y + 24,
            "\u{E714}",
            DT_CENTER_V,
        );
        draw_text(
            dc,
            font_heading,
            MONO_WHITE,
            38,
            y,
            200,
            y + 24,
            "Camera Live",
            0,
        );

        // Pop Out button
        let pop_rect = rect(FLYOUT_WIDTH - 16 - 80, y - 2, FLYOUT_WIDTH - 16, y + 24);
        flyout
            .button_rects
            .push((FlyoutButton::PopOutCamera, pop_rect));
        let pop_hover = flyout.hover_btn == Some(FlyoutButton::PopOutCamera);
        draw_pill_bg(
            dc,
            pop_rect.left,
            pop_rect.top,
            pop_rect.right,
            pop_rect.bottom,
            if pop_hover {
                MONO_BTN_HOVER
            } else {
                MONO_BTN_BG
            },
            if pop_hover {
                MONO_BTN_BORDER_HI
            } else {
                MONO_BTN_BORDER
            },
        );
        draw_text(
            dc,
            font_body_bold,
            MONO_LIGHT,
            pop_rect.left,
            pop_rect.top,
            pop_rect.right,
            pop_rect.bottom,
            "Pop Out",
            DT_CENTER_V,
        );

        // Camera preview bitmap blit
        let prev_top = y + 28;
        let prev_w = FLYOUT_WIDTH - 32;
        let prev_h = 100;
        let bgr = frame.to_bgr();
        let bmi = win32::BITMAPINFO {
            bmiHeader: win32::BITMAPINFOHEADER {
                biSize: std::mem::size_of::<win32::BITMAPINFOHEADER>() as u32,
                biWidth: frame.width as i32,
                biHeight: -(frame.height as i32),
                biPlanes: 1,
                biBitCount: 24,
                biCompression: win32::BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0],
        };
        unsafe {
            win32::StretchDIBits(
                dc,
                16,
                prev_top,
                prev_w,
                prev_h,
                0,
                0,
                frame.width as i32,
                frame.height as i32,
                bgr.as_ptr() as *const c_void,
                &bmi,
                win32::DIB_RGB_COLORS,
                win32::SRCCOPY,
            );
        }
    } else {
        // Camera off row
        draw_text(
            dc,
            font_icon,
            MONO_DIM,
            16,
            y,
            34,
            y + 20,
            "\u{E714}",
            DT_CENTER_V,
        );
        draw_text(dc, font_body, MONO_DIM, 38, y, 180, y + 20, "Camera Off", 0);
    }
}
