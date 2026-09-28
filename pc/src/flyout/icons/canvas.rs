//! Anti-aliased SVG Canvas mapping 24x24 standard SVG coordinate space to arbitrary target bounds.

#![cfg(windows)]

use super::super::win32::Gdiplus;
use std::ffi::c_void;

#[inline]
pub fn create_vector_pen(g: &Gdiplus, color: u32, width: f32) -> *mut c_void {
    unsafe {
        let mut pen: *mut c_void = std::ptr::null_mut();
        (g.fn_create_pen)(color, width, 2, &mut pen);
        if !pen.is_null() {
            (g.fn_set_pen_start_cap)(pen, 2); // LineCapRound
            (g.fn_set_pen_end_cap)(pen, 2); // LineCapRound
            (g.fn_set_pen_line_join)(pen, 2); // LineJoinRound
        }
        pen
    }
}

pub struct SvgCanvas<'a> {
    g: &'a Gdiplus,
    graphics: *mut c_void,
    pen: *mut c_void,
    ox: f32,
    oy: f32,
    scale: f32,
}

impl<'a> SvgCanvas<'a> {
    #[inline]
    pub fn new(
        g: &'a Gdiplus,
        graphics: *mut c_void,
        pen: *mut c_void,
        cx: f32,
        cy: f32,
        size: f32,
    ) -> Self {
        let scale = size / 24.0;
        let ox = cx - size / 2.0;
        let oy = cy - size / 2.0;
        Self {
            g,
            graphics,
            pen,
            ox,
            oy,
            scale,
        }
    }

    #[inline]
    pub fn pt(&self, x: f32, y: f32) -> (f32, f32) {
        (self.ox + x * self.scale, self.oy + y * self.scale)
    }

    #[inline]
    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32) {
        let (px1, py1) = self.pt(x1, y1);
        let (px2, py2) = self.pt(x2, y2);
        unsafe {
            (self.g.fn_draw_line)(self.graphics, self.pen, px1, py1, px2, py2);
        }
    }

    #[inline]
    pub fn arc(&self, x: f32, y: f32, w: f32, h: f32, start: f32, sweep: f32) {
        let (px, py) = self.pt(x, y);
        let pw = w * self.scale;
        let ph = h * self.scale;
        unsafe {
            (self.g.fn_draw_arc)(self.graphics, self.pen, px, py, pw, ph, start, sweep);
        }
    }

    #[inline]
    pub fn circle(&self, cx: f32, cy: f32, r: f32) {
        let (px, py) = self.pt(cx - r, cy - r);
        let d = r * 2.0 * self.scale;
        unsafe {
            (self.g.fn_draw_ellipse)(self.graphics, self.pen, px, py, d, d);
        }
    }

    #[inline]
    pub fn round_rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32) {
        let d = r * 2.0;
        self.arc(x, y, d, d, 180.0, 90.0);
        self.line(x + r, y, x + w - r, y);
        self.arc(x + w - d, y, d, d, 270.0, 90.0);
        self.line(x + w, y + r, x + w, y + h - r);
        self.arc(x + w - d, y + h - d, d, d, 0.0, 90.0);
        self.line(x + w - r, y + h, x + r, y + h);
        self.arc(x, y + h - d, d, d, 90.0, 90.0);
        self.line(x, y + h - r, x, y + r);
    }
}
