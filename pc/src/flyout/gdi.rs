//! GDI & GDI+ drawing primitives, smooth pills, smooth sliders, and typography.

#![cfg(windows)]

use super::types::to_wide;
use super::win32::{self, Gdiplus};
use std::ffi::c_void;

pub const DT_CENTER_V: u32 = win32::DT_SINGLELINE | win32::DT_CENTER | win32::DT_VCENTER;

/// Scalable TrueType font with negative character height for crisp ClearType rendering.
pub fn make_font(char_height_px: i32, weight: i32, face: &str) -> win32::HFONT {
    unsafe {
        win32::CreateFontW(
            -char_height_px,
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

/// Fallback standard GDI pill background.
pub fn draw_pill_bg(dc: win32::HDC, l: i32, t: i32, r: i32, b: i32, bg: u32, border: u32) {
    let rad = ((b - t).min(r - l) / 2) * 2;
    unsafe {
        let brush = win32::CreateSolidBrush(bg);
        let pen = win32::CreatePen(win32::PS_SOLID, 1, border);
        let ob = win32::SelectObject(dc, brush);
        let op = win32::SelectObject(dc, pen);
        win32::RoundRect(dc, l, t, r, b, rad, rad);
        win32::SelectObject(dc, ob);
        win32::SelectObject(dc, op);
        win32::DeleteObject(brush);
        win32::DeleteObject(pen);
    }
}

/// Draws a buttery smooth rounded pill with anti-aliasing via GDI+.
#[allow(clippy::too_many_arguments)]
pub fn draw_smooth_pill(
    g: &Gdiplus,
    graphics: *mut c_void,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    r: f32,
    bg_color: u32,
    border_color: u32,
) {
    unsafe {
        let mut path: *mut c_void = std::ptr::null_mut();
        (g.fn_create_path)(0, &mut path);
        if path.is_null() {
            return;
        }

        let d = r * 2.0;
        for (ax, ay, st) in [
            (x, y, 180.0),
            (x + w - d, y, 270.0),
            (x + w - d, y + h - d, 0.0),
            (x, y + h - d, 90.0),
        ] {
            (g.fn_add_path_arc)(path, ax, ay, d, d, st, 90.0);
        }
        (g.fn_close_path_figure)(path);

        let mut brush: *mut c_void = std::ptr::null_mut();
        (g.fn_create_solid_fill)(bg_color, &mut brush);
        if !brush.is_null() {
            (g.fn_fill_path)(graphics, brush, path);
            (g.fn_delete_brush)(brush);
        }

        if (border_color >> 24) > 0 {
            let mut pen: *mut c_void = std::ptr::null_mut();
            (g.fn_create_pen)(border_color, 1.0, 2, &mut pen);
            if !pen.is_null() {
                (g.fn_draw_path)(graphics, pen, path);
                (g.fn_delete_pen)(pen);
            }
        }

        (g.fn_delete_path)(path);
    }
}

/// Draws an anti-aliased smooth solid circle.
pub fn draw_smooth_circle(
    g: &Gdiplus,
    graphics: *mut c_void,
    cx: f32,
    cy: f32,
    r: f32,
    color: u32,
) {
    unsafe {
        let mut brush: *mut c_void = std::ptr::null_mut();
        (g.fn_create_solid_fill)(color, &mut brush);
        if !brush.is_null() {
            (g.fn_fill_ellipse)(graphics, brush, cx - r, cy - r, r * 2.0, r * 2.0);
            (g.fn_delete_brush)(brush);
        }
    }
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
        win32::SetBkMode(hdc, win32::TRANSPARENT);
        let mut rc = win32::RECT {
            left,
            top,
            right,
            bottom,
        };
        let wide = to_wide(text);
        win32::DrawTextW(
            hdc,
            wide.as_ptr(),
            -1,
            &mut rc,
            win32::DT_NOPREFIX | extra_flags,
        );
    }
}
