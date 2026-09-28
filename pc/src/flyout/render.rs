//! Main double-buffered paint coordinator for Mikey Flyout.
//! Android-style vertical card layout with anti-aliased GDI+ rendering.

#![cfg(windows)]

use super::gdi::*;
use super::palette::*;
use super::render_audio::render_mic_section;
use super::render_banners::render_banners;
use super::render_dsp::render_dsp_section;
use super::render_footer::render_footer_section;
use super::render_header::render_header;
use super::render_video::render_video_section;
use super::types::*;
use super::win32::{self, Gdiplus};
use super::window::FlyoutWindow;
use std::ffi::c_void;

impl FlyoutWindow {
    pub(crate) fn on_paint(&mut self) {
        self.sync_settings();
        let mut ps = win32::PAINTSTRUCT::default();
        let hdc = unsafe { win32::BeginPaint(self.hwnd, &mut ps) };
        if hdc == 0 {
            return;
        }

        let height = self.current_height();
        let mem_dc = unsafe { win32::CreateCompatibleDC(hdc) };
        let mem_bmp = unsafe { win32::CreateCompatibleBitmap(hdc, FLYOUT_WIDTH, height) };
        let old_bmp = unsafe { win32::SelectObject(mem_dc, mem_bmp) };

        self.button_rects.clear();

        let g_opt = self.gdiplus.take();
        let mut graphics: *mut c_void = std::ptr::null_mut();
        if let Some(g) = &g_opt {
            unsafe {
                (g.fn_create_from_hdc)(mem_dc, &mut graphics);
                if !graphics.is_null() {
                    (g.fn_set_smoothing_mode)(graphics, 4); // AntiAlias
                    (g.fn_set_text_rendering_hint)(graphics, 5); // ClearType
                }
            }
        }

        draw_card_bg(mem_dc, g_opt.as_ref(), graphics, height);

        let fonts = FlyoutFonts {
            title: make_font(15, 700, "Segoe UI"),
            heading: make_font(12, 600, "Segoe UI"),
            body: make_font(11, 400, "Segoe UI"),
            body_bold: make_font(11, 600, "Segoe UI"),
            icon: make_font(11, 400, "Segoe MDL2 Assets"),
        };

        let g = g_opt.as_ref();
        let mut y = render_header(mem_dc, self, g, graphics, &fonts);
        y = render_banners(mem_dc, self, g, graphics, &fonts, y);
        y = render_video_section(mem_dc, self, g, graphics, &fonts, y);
        y = render_mic_section(mem_dc, self, g, graphics, &fonts, y);
        let _ = render_dsp_section(mem_dc, self, g, graphics, &fonts, y);
        render_footer_section(mem_dc, self, g, graphics, &fonts, height);

        unsafe {
            win32::BitBlt(
                hdc,
                0,
                0,
                FLYOUT_WIDTH,
                height,
                mem_dc,
                0,
                0,
                win32::SRCCOPY,
            );
            if let Some(g_inst) = &g_opt {
                if !graphics.is_null() {
                    (g_inst.fn_delete_graphics)(graphics);
                }
            }
            win32::SelectObject(mem_dc, old_bmp);
            win32::DeleteObject(mem_bmp);
            win32::DeleteDC(mem_dc);
            for f in [
                fonts.title,
                fonts.heading,
                fonts.body,
                fonts.body_bold,
                fonts.icon,
            ] {
                win32::DeleteObject(f);
            }
            win32::EndPaint(self.hwnd, &ps);
        }
        self.gdiplus = g_opt;
    }
}

fn draw_card_bg(dc: win32::HDC, g_opt: Option<&Gdiplus>, graphics: *mut c_void, h: i32) {
    if let Some(g) = g_opt {
        if !graphics.is_null() {
            draw_smooth_pill(
                g,
                graphics,
                0.5,
                0.5,
                (FLYOUT_WIDTH - 1) as f32,
                (h - 1) as f32,
                FLYOUT_CORNER_RADIUS as f32,
                ARGB_SURFACE,
                ARGB_BORDER,
            );
            return;
        }
    }
    let brush = unsafe { win32::CreateSolidBrush(COLOR_BG_SURFACE) };
    let pen = unsafe { win32::CreatePen(win32::PS_SOLID, 1, COLOR_BORDER) };
    unsafe {
        let ob = win32::SelectObject(dc, brush);
        let op = win32::SelectObject(dc, pen);
        win32::RoundRect(
            dc,
            0,
            0,
            FLYOUT_WIDTH,
            h,
            FLYOUT_CORNER_RADIUS * 2,
            FLYOUT_CORNER_RADIUS * 2,
        );
        win32::SelectObject(dc, ob);
        win32::SelectObject(dc, op);
        win32::DeleteObject(brush);
        win32::DeleteObject(pen);
    }
}
