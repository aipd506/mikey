//! System icons: power quit, logs folder, disconnect, sliders, and alert.

#![cfg(windows)]

use super::super::win32::Gdiplus;
use super::canvas::{create_vector_pen, SvgCanvas};
use std::ffi::c_void;

/// Official Lucide power quit vector icon
pub fn draw_hero_power(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.8);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.arc(4.0, 4.0, 16.0, 16.0, 130.0, 280.0);
    c.line(12.0, 2.5, 12.0, 11.5);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide folder logs vector icon
pub fn draw_hero_folder(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(3.0, 6.0, 8.5, 6.0);
    c.line(8.5, 6.0, 10.5, 4.0);
    c.line(10.5, 4.0, 14.5, 4.0);
    c.line(14.5, 4.0, 14.5, 6.0);
    c.line(14.5, 6.0, 20.0, 6.0);
    c.arc(18.0, 6.0, 4.0, 4.0, 270.0, 90.0);
    c.line(22.0, 8.0, 22.0, 18.0);
    c.arc(18.0, 16.0, 4.0, 4.0, 0.0, 90.0);
    c.line(20.0, 20.0, 4.0, 20.0);
    c.arc(2.0, 16.0, 4.0, 4.0, 90.0, 90.0);
    c.line(2.0, 18.0, 2.0, 8.0);
    c.arc(2.0, 6.0, 4.0, 4.0, 180.0, 90.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide unlink disconnect vector icon
pub fn draw_hero_disconnect(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.8);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(3.0, 3.0, 21.0, 21.0);

    c.line(18.0, 12.0, 20.0, 10.0);
    c.arc(14.0, 3.0, 7.0, 7.0, -45.0, 180.0);
    c.line(14.0, 7.0, 12.0, 9.0);

    c.line(6.0, 12.0, 4.0, 14.0);
    c.arc(3.0, 14.0, 7.0, 7.0, 135.0, 180.0);
    c.line(10.0, 17.0, 12.0, 15.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide sliders-horizontal vector icon
pub fn draw_hero_sliders(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(3.0, 5.0, 21.0, 5.0);
    c.line(8.0, 2.5, 8.0, 7.5);

    c.line(3.0, 12.0, 21.0, 12.0);
    c.line(16.0, 9.5, 16.0, 14.5);

    c.line(3.0, 19.0, 21.0, 19.0);
    c.line(11.0, 16.5, 11.0, 21.5);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide alert-triangle vector icon
pub fn draw_hero_alert(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(12.0, 3.5, 21.0, 19.0);
    c.line(21.0, 19.0, 3.0, 19.0);
    c.line(3.0, 19.0, 12.0, 3.5);

    c.line(12.0, 8.0, 12.0, 13.0);
    c.circle(12.0, 16.5, 0.6);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}
