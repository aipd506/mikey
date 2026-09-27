//! Camera preview bitmap blitting helper for the Mikey Flyout.

#![cfg(windows)]

use super::types::{win32, FLYOUT_WIDTH};
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
