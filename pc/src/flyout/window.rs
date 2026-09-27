//! Win32 window management, layout, and life cycle for Mikey Flyout.

#![cfg(windows)]

use super::layout::*;
use super::types::*;
use super::wndproc::flyout_wndproc;
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
    pub(crate) advanced_expanded: bool,
    pub(crate) is_muted: bool,
    pub(crate) ns_strength: f32,
    pub(crate) aec_enabled: bool,
    pub(crate) dsp_gate_enabled: bool,
    pub(crate) shown_at: Option<Instant>,
    pub(crate) last_toggled_at: Instant,
    pub(crate) hover_btn: Option<FlyoutButton>,
    pub(crate) is_dragging_ns: bool,
    pub(crate) button_rects: Vec<(FlyoutButton, win32::RECT)>,
}

impl FlyoutWindow {
    pub fn new(
        session_manager: SessionManager,
        video_pipeline: Arc<VideoPipeline>,
        jitter_buffer: Arc<JitterBuffer>,
        running: Arc<AtomicBool>,
    ) -> Box<Self> {
        let class_name = to_wide("MikeyFlyoutCompanionClass");

        unsafe {
            let wc = win32::WNDCLASSEXW {
                cbSize: std::mem::size_of::<win32::WNDCLASSEXW>() as u32,
                style: 0x0001 | 0x0002,
                lpfnWndProc: Some(flyout_wndproc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: 0,
                hIcon: 0,
                hCursor: 0,
                hbrBackground: 0,
                lpszMenuName: std::ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: 0,
            };
            win32::RegisterClassExW(&wc);
        }

        let hwnd = unsafe {
            win32::CreateWindowExW(
                win32::WS_EX_TOOLWINDOW | win32::WS_EX_TOPMOST,
                class_name.as_ptr(),
                to_wide("Mikey").as_ptr(),
                win32::WS_POPUP,
                -2000,
                -2000,
                FLYOUT_WIDTH,
                FLYOUT_HEIGHT_COLLAPSED,
                0,
                0,
                0,
                std::ptr::null_mut(),
            )
        };

        update_window_clip_region(hwnd, FLYOUT_HEIGHT_COLLAPSED);

        let mut flyout = Box::new(Self {
            hwnd,
            session_manager,
            video_pipeline,
            jitter_buffer,
            running,
            visible: false,
            advanced_expanded: false,
            is_muted: false,
            ns_strength: 1.0,
            aec_enabled: true,
            dsp_gate_enabled: false,
            shown_at: None,
            last_toggled_at: Instant::now() - std::time::Duration::from_secs(10),
            hover_btn: None,
            is_dragging_ns: false,
            button_rects: Vec::new(),
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
        compute_flyout_height(&self.session_manager, &self.video_pipeline)
    }

    pub fn update_window_region(&self, height: i32) {
        update_window_clip_region(self.hwnd, height);
    }

    pub fn hide(&mut self) {
        unsafe {
            win32::ShowWindow(self.hwnd, win32::SW_HIDE);
            win32::KillTimer(self.hwnd, 1);
        }
        self.visible = false;
        self.shown_at = None;
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

        let height = self.current_height();
        let (x, y) = calculate_flyout_position(tray_x, tray_y, tray_w, tray_h, height);

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
