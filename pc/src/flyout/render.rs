//! Main double-buffered GDI paint coordinator for Mikey Flyout.
//! Ultra-compact, visual-first layout with crisp typography.

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
use super::window::FlyoutWindow;

impl FlyoutWindow {
    pub(crate) fn on_paint(&mut self) {
        let mut ps = win32::PAINTSTRUCT {
            hdc: 0,
            fErase: 0,
            rcPaint: win32::RECT::default(),
            fRestore: 0,
            fIncUpdate: 0,
            rgbReserved: [0; 32],
        };

        let hdc = unsafe { win32::BeginPaint(self.hwnd, &mut ps) };
        if hdc == 0 {
            return;
        }

        let height = self.current_height();

        let mem_dc = unsafe { win32::CreateCompatibleDC(hdc) };
        let mem_bmp = unsafe { win32::CreateCompatibleBitmap(hdc, FLYOUT_WIDTH, height) };
        let old_bmp = unsafe { win32::SelectObject(mem_dc, mem_bmp) };

        self.button_rects.clear();

        // ── Card Background with subtle rounded border ──
        let bg_brush = unsafe { win32::CreateSolidBrush(COLOR_BG_SURFACE) };
        let border_pen = unsafe { win32::CreatePen(win32::PS_SOLID, 1, COLOR_BORDER) };
        let old_brush = unsafe { win32::SelectObject(mem_dc, bg_brush) };
        let old_pen = unsafe { win32::SelectObject(mem_dc, border_pen) };
        unsafe {
            win32::RoundRect(
                mem_dc,
                0,
                0,
                FLYOUT_WIDTH,
                height,
                FLYOUT_CORNER_RADIUS * 2,
                FLYOUT_CORNER_RADIUS * 2,
            );
            win32::SetBkMode(mem_dc, win32::TRANSPARENT);
        }

        // ── Neat legible fonts (Segoe UI Variable & MDL2 Assets) ──
        let font_title = make_font(15, 700, "Segoe UI Variable Display");
        let font_heading = make_font(12, 600, "Segoe UI Variable Text");
        let font_body = make_font(11, 400, "Segoe UI Variable Text");
        let font_body_bold = make_font(11, 600, "Segoe UI Variable Text");
        let font_icon = make_font(13, 400, "Segoe MDL2 Assets");
        let font_icon_lg = make_font(15, 400, "Segoe MDL2 Assets");

        // 1. Header: "Mikey", device subtitle & status pill
        let mut y = render_header(
            mem_dc,
            self,
            font_title,
            font_body,
            font_body_bold,
            font_icon,
        );

        // 2. Banners: pending join & virtual mic warnings
        y = render_banners(mem_dc, self, font_body, font_body_bold, font_icon, y);

        // Divider
        draw_divider(mem_dc, y);
        y += 6;

        // 3. Audio section: microphone, mute toggle, and live VU meter
        y = render_mic_section(mem_dc, self, font_heading, font_body_bold, font_icon, y);

        // Divider
        draw_divider(mem_dc, y);
        y += 6;

        // 4. Video section: camera indicator, flip camera, and persistent preview option
        y = render_video_section(mem_dc, self, font_heading, font_body_bold, font_icon, y);

        // Divider
        draw_divider(mem_dc, y);
        y += 6;

        // 5. DSP section: compact NS slider, AEC toggle, Gate toggle
        let _ = render_dsp_section(mem_dc, self, font_body, font_body_bold, font_icon, y);

        // 6. Footer: telemetry, logs, disconnect, quit
        render_footer_section(mem_dc, self, font_body, font_icon_lg, height);

        // Blit backbuffer to screen
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
            win32::SelectObject(mem_dc, old_bmp);
            win32::SelectObject(mem_dc, old_brush);
            win32::SelectObject(mem_dc, old_pen);
            win32::DeleteObject(mem_bmp);
            win32::DeleteDC(mem_dc);
            win32::DeleteObject(bg_brush);
            win32::DeleteObject(border_pen);
            win32::DeleteObject(font_title);
            win32::DeleteObject(font_heading);
            win32::DeleteObject(font_body);
            win32::DeleteObject(font_body_bold);
            win32::DeleteObject(font_icon);
            win32::DeleteObject(font_icon_lg);
            win32::EndPaint(self.hwnd, &ps);
        }
    }
}

fn draw_divider(dc: win32::HDC, y: i32) {
    draw_pill_bg(
        dc,
        14,
        y,
        FLYOUT_WIDTH - 14,
        y + 1,
        COLOR_BORDER,
        COLOR_BORDER,
    );
}
