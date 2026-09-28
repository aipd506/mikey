//! Camera preview bitmap blitting and embedded preview box rendering for the Mikey Flyout.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use crate::video::DecodedFrame;
use std::ffi::c_void;

pub fn blit_camera_preview(dc: win32::HDC, frame: &DecodedFrame, top: i32) {
    let prev_w = FLYOUT_WIDTH - 28;
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
            14,
            top,
            prev_w,
            100,
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
}

pub fn render_embedded_preview(
    dc: win32::HDC,
    flyout: &mut FlyoutWindow,
    g_opt: Option<&Gdiplus>,
    graphics: *mut c_void,
    fonts: &FlyoutFonts,
    frame: &DecodedFrame,
    top: i32,
) -> i32 {
    let card_left = 14;
    let card_w = FLYOUT_WIDTH - 28;

    blit_camera_preview(dc, frame, top);

    let prev_click_rect = rect(card_left, top, card_left + card_w, top + 100);
    flyout
        .button_rects
        .push((FlyoutButton::TogglePreview, prev_click_rect));

    if let Some(g) = g_opt {
        draw_smooth_pill(
            g,
            graphics,
            card_left as f32,
            top as f32,
            card_w as f32,
            100.0,
            8.0,
            0,
            ARGB_BORDER,
        );
        draw_smooth_circle(
            g,
            graphics,
            (card_left + 14) as f32,
            (top + 14) as f32,
            3.5,
            ARGB_CAM_ON,
        );
    }

    draw_text(
        dc,
        fonts.body_bold,
        COLOR_CAM_ON,
        card_left + 22,
        top + 6,
        card_left + 65,
        top + 22,
        "LIVE",
        0,
    );

    top + 100 + 8
}
