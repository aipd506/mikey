//! Media icons: microphone, video camera, flip lens, audio wave, and monitor preview.

#![cfg(windows)]

use super::super::win32::Gdiplus;
use super::canvas::{create_vector_pen, SvgCanvas};
use std::ffi::c_void;

/// Official Lucide mic / mic-off vector icon
pub fn draw_hero_mic(
    g: &Gdiplus,
    graphics: *mut c_void,
    cx: f32,
    cy: f32,
    color: u32,
    is_muted: bool,
) {
    let size = 20.0;
    let pen = create_vector_pen(g, color, 1.8);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    if is_muted {
        c.line(3.0, 3.0, 21.0, 21.0);
        c.arc(5.0, 5.0, 14.0, 14.0, 20.0, 70.0);
        c.arc(5.0, 5.0, 14.0, 14.0, 110.0, 50.0);
        c.arc(9.0, 3.0, 6.0, 6.0, 180.0, 180.0);
        c.line(15.0, 6.0, 15.0, 9.0);
        c.line(9.0, 7.0, 9.0, 12.0);
        c.line(12.0, 19.0, 12.0, 22.0);
    } else {
        c.round_rect(9.0, 2.5, 6.0, 11.5, 3.0);
        c.arc(5.0, 5.0, 14.0, 14.0, 0.0, 180.0);
        c.line(12.0, 19.0, 12.0, 22.0);
    }

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide video camera vector icon
pub fn draw_hero_camera(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 20.0;
    let pen = create_vector_pen(g, color, 1.8);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.round_rect(2.0, 6.0, 13.0, 12.0, 2.5);
    c.line(15.0, 11.0, 21.0, 7.5);
    c.line(21.0, 7.5, 21.0, 16.5);
    c.line(21.0, 16.5, 15.0, 13.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide switch-camera flip vector icon
pub fn draw_hero_flip(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(10.0, 19.0, 4.0, 19.0);
    c.arc(2.0, 15.0, 4.0, 4.0, 90.0, 90.0);
    c.line(2.0, 17.0, 2.0, 7.0);
    c.arc(2.0, 5.0, 4.0, 4.0, 180.0, 90.0);
    c.line(4.0, 5.0, 9.0, 5.0);

    c.line(14.0, 5.0, 20.0, 5.0);
    c.arc(18.0, 5.0, 4.0, 4.0, 270.0, 90.0);
    c.line(22.0, 7.0, 22.0, 17.0);
    c.arc(18.0, 15.0, 4.0, 4.0, 0.0, 90.0);
    c.line(20.0, 19.0, 14.0, 19.0);

    c.circle(12.0, 12.0, 3.0);

    c.line(3.0, 5.0, 6.0, 2.0);
    c.line(6.0, 2.0, 9.0, 5.0);

    c.line(21.0, 19.0, 18.0, 22.0);
    c.line(18.0, 22.0, 15.0, 19.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide volume-2 sound wave vector icon
pub fn draw_hero_sound(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 18.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.line(3.0, 9.0, 6.0, 9.0);
    c.line(6.0, 9.0, 11.0, 5.0);
    c.line(11.0, 5.0, 11.0, 19.0);
    c.line(11.0, 19.0, 6.0, 15.0);
    c.line(6.0, 15.0, 3.0, 15.0);
    c.line(3.0, 15.0, 3.0, 9.0);

    c.arc(7.0, 7.0, 10.0, 10.0, -45.0, 90.0);
    c.arc(5.0, 3.0, 16.0, 16.0, -45.0, 90.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}

/// Official Lucide monitor preview vector icon
pub fn draw_hero_preview(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    let size = 16.0;
    let pen = create_vector_pen(g, color, 1.6);
    if pen.is_null() {
        return;
    }
    let c = SvgCanvas::new(g, graphics, pen, cx, cy, size);

    c.round_rect(2.0, 3.0, 20.0, 14.0, 2.0);
    c.line(8.0, 21.0, 16.0, 21.0);
    c.line(12.0, 17.0, 12.0, 21.0);

    unsafe {
        (g.fn_delete_pen)(pen);
    }
}
