use super::types::*;
use super::window::FlyoutWindow;
use crate::autostart;
use crate::config::Config;
use std::sync::atomic::Ordering;

impl FlyoutWindow {
    pub(crate) fn on_mouse_move(&mut self, x: i32, _y: i32) {
        if self.is_dragging_ns {
            if let Some((_, rect)) = self
                .button_rects
                .iter()
                .find(|(b, _)| *b == FlyoutButton::NsSlider)
            {
                let ratio = (x.clamp(rect.left, rect.right) - rect.left) as f32
                    / (rect.right - rect.left) as f32;
                self.ns_strength = ratio;
                self.jitter_buffer.set_ns_strength((ratio * 100.0) as u32);
                unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
            }
            return;
        }

        let mut found_hover = None;
        for (btn, rect) in &self.button_rects {
            if x >= rect.left && x <= rect.right && _y >= rect.top && _y <= rect.bottom {
                found_hover = Some(btn.clone());
                break;
            }
        }

        if self.hover_btn != found_hover {
            self.hover_btn = found_hover;
            unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
        }
    }

    pub(crate) fn on_lbutton_down(&mut self, x: i32, y: i32) {
        for (btn, rect) in &self.button_rects {
            if x >= rect.left
                && x <= rect.right
                && y >= rect.top
                && y <= rect.bottom
                && btn == &FlyoutButton::NsSlider
            {
                self.is_dragging_ns = true;
                let ratio = (x.clamp(rect.left, rect.right) - rect.left) as f32
                    / (rect.right - rect.left) as f32;
                self.ns_strength = ratio;
                self.jitter_buffer.set_ns_strength((ratio * 100.0) as u32);
                unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
                return;
            }
        }
    }

    pub(crate) fn on_lbutton_up(&mut self, x: i32, y: i32) {
        self.is_dragging_ns = false;

        let mut target_btn = None;
        for (btn, rect) in &self.button_rects {
            if x >= rect.left && x <= rect.right && y >= rect.top && y <= rect.bottom {
                target_btn = Some(btn.clone());
                break;
            }
        }

        if let Some(btn) = target_btn {
            match btn {
                FlyoutButton::Disconnect => {
                    self.session_manager.close_session("user disconnected");
                }
                FlyoutButton::AllowJoin(request_id) => {
                    self.session_manager.resolve_pending(request_id, true);
                }
                FlyoutButton::DenyJoin(request_id) => {
                    self.session_manager.resolve_pending(request_id, false);
                }
                FlyoutButton::MicToggle => {}
                FlyoutButton::MuteToggle => {
                    self.is_muted = !self.is_muted;
                }
                FlyoutButton::ToggleAec => {
                    self.aec_enabled = !self.aec_enabled;
                    self.jitter_buffer.set_aec_enabled(self.aec_enabled);
                }
                FlyoutButton::SetupVirtualMic => {
                    let candidates = [
                        std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|d| d.join("setup-audio-device.ps1"))),
                        std::env::current_exe().ok().and_then(|p| {
                            p.parent()
                                .map(|d| d.join("installer").join("setup-audio-device.ps1"))
                        }),
                        Some(std::path::PathBuf::from(
                            "pc/installer/setup-audio-device.ps1",
                        )),
                        Some(std::path::PathBuf::from("installer/setup-audio-device.ps1")),
                        Some(std::path::PathBuf::from("setup-audio-device.ps1")),
                    ];
                    if let Some(path) = candidates.into_iter().flatten().find(|p| p.exists()) {
                        let _ = std::process::Command::new("powershell.exe")
                            .arg("-ExecutionPolicy")
                            .arg("Bypass")
                            .arg("-File")
                            .arg(path)
                            .spawn();
                    }
                }
                FlyoutButton::NsSlider => {}
                FlyoutButton::ToggleGate => {
                    self.dsp_gate_enabled = !self.dsp_gate_enabled;
                }
                FlyoutButton::PopOutCamera => {
                    self.video_pipeline.toggle_preview();
                }
                FlyoutButton::FlipCamera => {}
                FlyoutButton::ToggleAdvanced => {
                    self.advanced_expanded = !self.advanced_expanded;
                    let height = if self.advanced_expanded {
                        FLYOUT_HEIGHT_EXPANDED
                    } else {
                        FLYOUT_HEIGHT_COLLAPSED
                    };
                    self.update_window_region(height);
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
                FlyoutButton::ToggleAskBeforeJoin => {
                    let mut cfg = self.session_manager.config();
                    cfg.ask_before_joining = !cfg.ask_before_joining;
                    let _ = cfg.save_to(&Config::default_config_path());
                }
                FlyoutButton::ToggleStartWithComputer => {
                    let cur = autostart::is_autostart_enabled();
                    let _ = autostart::set_autostart(!cur);
                }
                FlyoutButton::ToggleOpenPhone => {
                    let mut cfg = self.session_manager.config();
                    cfg.open_on_phone_when_plugged_in = !cfg.open_on_phone_when_plugged_in;
                    let _ = cfg.save_to(&Config::default_config_path());
                }
                FlyoutButton::ToggleTrustWifi => {
                    let mut cfg = self.session_manager.config();
                    cfg.trust_wifi_automatically = !cfg.trust_wifi_automatically;
                    let _ = cfg.save_to(&Config::default_config_path());
                }
                FlyoutButton::OpenLogs => {
                    let log_dir = Config::default_log_dir();
                    let _ = std::fs::create_dir_all(&log_dir);
                    let _ = std::process::Command::new("explorer.exe")
                        .arg(log_dir)
                        .spawn();
                }
                FlyoutButton::Quit => {
                    self.running.store(false, Ordering::Relaxed);
                    self.hide();
                }
            }
            unsafe { win32::InvalidateRect(self.hwnd, std::ptr::null(), 0) };
        }
    }
}
