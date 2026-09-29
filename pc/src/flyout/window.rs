//! Win32 window management, layout, and life cycle for Mikey Flyout.

#![cfg(windows)]

use super::layout::*;
use super::types::*;
use crate::audio::pipeline::JitterBuffer;
use crate::session::SessionManager;
use crate::video::VideoPipeline;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Instant;

pub struct FlyoutWindow {
    pub(crate) hwnd: win32::HWND,
    pub(crate) session_manager: SessionManager,
    pub(crate) video_pipeline: Arc<VideoPipeline>,
    pub(crate) jitter_buffer: Arc<JitterBuffer>,
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) visible: bool,
    pub(crate) height: i32,
    pub(crate) show_requested: bool,
    pub(crate) is_muted: bool,
    pub(crate) ns_strength: f32,
    pub(crate) shown_at: Option<Instant>,
    pub(crate) last_hidden_at: Option<Instant>,
    pub(crate) last_toggled_at: Instant,
    pub(crate) hover_btn: Option<FlyoutButton>,
    pub(crate) is_dragging_ns: bool,
    pub(crate) settings_expanded: bool,
    pub(crate) button_rects: Vec<(FlyoutButton, win32::RECT)>,
    pub(crate) gdiplus: Option<win32::Gdiplus>,
}

impl FlyoutWindow {
    pub fn new(
        session_manager: SessionManager,
        video_pipeline: Arc<VideoPipeline>,
        jitter_buffer: Arc<JitterBuffer>,
        running: Arc<AtomicBool>,
    ) -> Box<Self> {
        let hwnd = create_flyout_hwnd();
        update_window_clip_region(hwnd, FLYOUT_HEIGHT_COLLAPSED);

        let mut flyout = Box::new(Self {
            hwnd,
            session_manager,
            video_pipeline,
            jitter_buffer,
            running,
            visible: false,
            height: FLYOUT_HEIGHT_COLLAPSED,
            show_requested: false,
            is_muted: false,
            ns_strength: 1.0,
            shown_at: None,
            last_hidden_at: None,
            last_toggled_at: Instant::now() - std::time::Duration::from_secs(10),
            hover_btn: None,
            is_dragging_ns: false,
            settings_expanded: false,
            button_rects: Vec::new(),
            gdiplus: win32::Gdiplus::init(),
        });

        let raw_ptr: *mut FlyoutWindow = &mut *flyout;
        unsafe {
            win32::SetWindowLongPtrW(hwnd, win32::GWLP_USERDATA, raw_ptr as isize);
        }

        flyout
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn current_height(&self) -> i32 {
        compute_flyout_height(
            &self.session_manager,
            &self.video_pipeline,
            self.settings_expanded,
        )
    }

    pub fn update_window_region(&mut self, height: i32) {
        self.height = height;
        update_window_clip_region(self.hwnd, height);
        unsafe {
            win32::SetWindowPos(
                self.hwnd,
                0,
                0,
                0,
                FLYOUT_WIDTH,
                height,
                win32::SWP_NOMOVE | win32::SWP_NOZORDER | win32::SWP_NOACTIVATE,
            );
        }
    }

    /// Shows what the PC actually applies, so changes made on the phone appear here too.
    pub(crate) fn sync_settings(&mut self) {
        if !self.is_dragging_ns {
            let ns_on = self.jitter_buffer.is_ns_enabled();
            let strength = self.jitter_buffer.get_ns_strength() as f32 / 100.0;
            self.ns_strength = if ns_on { strength } else { 0.0 };
        }
        self.is_muted = self.session_manager.is_phone_muted();
    }

    /// True once after another launch of Mikey asked this one to open.
    pub fn take_show_request(&mut self) -> bool {
        std::mem::take(&mut self.show_requested)
    }

    pub fn hide(&mut self) {
        unsafe {
            win32::ShowWindow(self.hwnd, win32::SW_HIDE);
            win32::KillTimer(self.hwnd, 1);
        }
        self.visible = false;
        self.shown_at = None;
        self.last_hidden_at = Some(Instant::now());
        self.is_dragging_ns = false;
        self.hover_btn = None;
    }

    pub fn toggle(&mut self, tray_x: i32, tray_y: i32, tray_w: i32, tray_h: i32) {
        let now = Instant::now();
        if now.duration_since(self.last_toggled_at).as_millis() < 200 {
            return;
        }
        self.last_toggled_at = now;
        if self.visible {
            self.hide();
            return;
        }

        if let Some(hidden) = self.last_hidden_at {
            if hidden.elapsed().as_millis() < 400 {
                return;
            }
        }

        // Listing audio devices can take a second, so it never runs here; only a missing mic
        // is checked again, in the background, in case setup was just run.
        if !crate::audio::sink::virtual_device_ready() {
            crate::audio::sink::refresh_virtual_device_status();
        }
        let height = self.current_height();
        let (x, y) = calculate_flyout_position(tray_x, tray_y, tray_w, tray_h, height);
        self.height = height;
        update_window_clip_region(self.hwnd, height);

        unsafe {
            win32::SetWindowPos(
                self.hwnd,
                -1isize as usize,
                x,
                y,
                FLYOUT_WIDTH,
                height,
                win32::SWP_SHOWWINDOW,
            );
            win32::BringWindowToTop(self.hwnd);
            win32::SetForegroundWindow(self.hwnd);
            win32::SetFocus(self.hwnd);
            win32::SetTimer(self.hwnd, 1, 33, None);
            win32::InvalidateRect(self.hwnd, std::ptr::null(), 0);
        }

        self.shown_at = Some(Instant::now());
        self.visible = true;
    }
}
