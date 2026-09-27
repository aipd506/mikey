//! GDI drawing primitives and typography helpers for the Mikey Flyout.

#![cfg(windows)]

use super::palette::*;
use super::types::to_wide;
use super::win32;

pub const DT_CENTER_V: u32 = win32::DT_SINGLELINE | win32::DT_CENTER | win32::DT_VCENTER;

pub fn make_font(size: i32, weight: i32, face: &str) -> win32::HFONT {
    unsafe {
        win32::CreateFontW(
            size,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            1,
            0,
            0,
            5,
            0,
            to_wide(face).as_ptr(),
        )
    }
}

pub fn rect(left: i32, top: i32, right: i32, bottom: i32) -> win32::RECT {
    win32::RECT {
        left,
        top,
        right,
        bottom,
    }
}

pub fn draw_pill_bg(hdc: win32::HDC, l: i32, t: i32, r: i32, b: i32, bg: u32, border: u32) {
    let brush = unsafe { win32::CreateSolidBrush(bg) };
    let pen = unsafe { win32::CreatePen(win32::PS_SOLID, 1, border) };
    unsafe {
        win32::SelectObject(hdc, brush);
        win32::SelectObject(hdc, pen);
        win32::RoundRect(hdc, l, t, r, b, 6, 6);
        win32::DeleteObject(brush);
        win32::DeleteObject(pen);
    }
}

pub fn draw_dot(hdc: win32::HDC, l: i32, t: i32, r: i32, b: i32, color: u32) {
    let brush = unsafe { win32::CreateSolidBrush(color) };
    let pen = unsafe { win32::CreatePen(win32::PS_SOLID, 1, color) };
    unsafe {
        win32::SelectObject(hdc, brush);
        win32::SelectObject(hdc, pen);
        win32::Ellipse(hdc, l, t, r, b);
        win32::DeleteObject(brush);
        win32::DeleteObject(pen);
    }
}

/// Draws a flat slider: 4 px track with white fill up to `value` and a 12 px circular thumb.
pub fn draw_slider(hdc: win32::HDC, track_left: i32, track_right: i32, center_y: i32, value: f32) {
    let track_h = 4;
    let thumb_r = 6;
    let tt = center_y - track_h / 2;
    let tb = center_y + track_h / 2;

    // Track background
    draw_pill_bg(hdc, track_left, tt, track_right, tb, MONO_DARK, MONO_DARK);

    // Active fill
    let fill_w = ((track_right - track_left) as f32 * value.clamp(0.0, 1.0)) as i32;
    if fill_w > 2 {
        let fill = unsafe { win32::CreateSolidBrush(MONO_WHITE) };
        let fill_p = unsafe { win32::CreatePen(win32::PS_SOLID, 1, MONO_WHITE) };
        unsafe {
            win32::SelectObject(hdc, fill);
            win32::SelectObject(hdc, fill_p);
            win32::RoundRect(hdc, track_left, tt, track_left + fill_w, tb, 4, 4);
            win32::DeleteObject(fill);
            win32::DeleteObject(fill_p);
        }
    }

    // Thumb circle
    let thumb_x = track_left + fill_w;
    draw_dot(
        hdc,
        thumb_x - thumb_r,
        center_y - thumb_r,
        thumb_x + thumb_r,
        center_y + thumb_r,
        MONO_WHITE,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn draw_text(
    hdc: win32::HDC,
    font: win32::HFONT,
    color: u32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    text: &str,
    extra_flags: u32,
) {
    unsafe {
        win32::SelectObject(hdc, font);
        win32::SetTextColor(hdc, color);
        let mut rc = win32::RECT {
            left,
            top,
            right,
            bottom,
        };
        win32::DrawTextW(
            hdc,
            to_wide(text).as_ptr(),
            -1,
            &mut rc,
            win32::DT_SINGLELINE | win32::DT_VCENTER | extra_flags,
        );
    }
}
