use crate::audio::pipeline::JitterBuffer;
use crate::flyout::FlyoutWindow;
use crate::session::SessionManager;
use crate::video::VideoPipeline;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::flyout::icons::{create_mikey_tray_icon, TrayIconMode};

pub struct TrayApp {
    tray: TrayIcon,
    session_manager: SessionManager,
    icon_grey: Icon,
    icon_green: Icon,
    icon_amber: Icon,
}

impl TrayApp {
    pub fn new(session_manager: SessionManager) -> Self {
        let icon_grey = create_mikey_tray_icon(TrayIconMode::Idle);
        let icon_green = create_mikey_tray_icon(TrayIconMode::Active);
        let icon_amber = create_mikey_tray_icon(TrayIconMode::Pending);

        // No OS popup context menu: clicking tray icon directly toggles custom Mikey Flyout
        let tray = TrayIconBuilder::new()
            .with_tooltip("Mikey — Phone Mic & Webcam")
            .with_icon(icon_grey.clone())
            .build()
            .expect("build tray icon");

        Self {
            tray,
            session_manager,
            icon_grey,
            icon_green,
            icon_amber,
        }
    }

    pub fn update_state(&self) {
        let has_pending = !self.session_manager.list_pending().is_empty();
        let is_active = self.session_manager.is_active();

        if has_pending {
            let _ = self.tray.set_icon(Some(self.icon_amber.clone()));
            let _ = self.tray.set_tooltip(Some("Mikey — Waiting for approval"));
        } else if is_active {
            let _ = self.tray.set_icon(Some(self.icon_green.clone()));
            if let Some(session) = self.session_manager.active_session() {
                let tip = format!(
                    "Mikey — {} streaming via L{}",
                    session.device_name, session.current_level
                );
                let _ = self.tray.set_tooltip(Some(tip));
            }
        } else {
            let _ = self.tray.set_icon(Some(self.icon_grey.clone()));
            let _ = self.tray.set_tooltip(Some("Mikey — Idle"));
        }
    }
}

/// Runs the system tray and flyout companion in a dedicated GUI thread.
pub fn start_tray_thread(
    session_manager: SessionManager,
    video_pipeline: Arc<VideoPipeline>,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let tray_app = TrayApp::new(session_manager.clone());
        let mut flyout = FlyoutWindow::new(
            session_manager,
            video_pipeline,
            jitter_buffer,
            Arc::clone(&running),
        );
        while running.load(Ordering::Relaxed) {
            tray_app.update_state();

            // Handle tray icon click events directly to toggle our custom dark flyout UI
            while let Ok(event) = TrayIconEvent::receiver().try_recv() {
                match event {
                    TrayIconEvent::Click {
                        button_state, rect, ..
                    } => {
                        if button_state == tray_icon::MouseButtonState::Up {
                            flyout.toggle(
                                rect.position.x as i32,
                                rect.position.y as i32,
                                rect.size.width as i32,
                                rect.size.height as i32,
                            );
                        }
                    }
                    TrayIconEvent::DoubleClick { rect, .. } => {
                        flyout.toggle(
                            rect.position.x as i32,
                            rect.position.y as i32,
                            rect.size.width as i32,
                            rect.size.height as i32,
                        );
                    }
                    _ => {}
                }
            }

            // Process Win32 message loop events on Windows
            #[cfg(windows)]
            {
                #[repr(C)]
                struct Point {
                    x: i32,
                    y: i32,
                }
                #[repr(C)]
                struct Msg {
                    hwnd: usize,
                    message: u32,
                    wparam: usize,
                    lparam: isize,
                    time: u32,
                    pt: Point,
                }
                #[link(name = "user32")]
                extern "system" {
                    fn PeekMessageW(
                        lpMsg: *mut Msg,
                        hWnd: usize,
                        wMsgFilterMin: u32,
                        wMsgFilterMax: u32,
                        wRemoveMsg: u32,
                    ) -> i32;
                    fn TranslateMessage(lpMsg: *const Msg) -> i32;
                    fn DispatchMessageW(lpMsg: *const Msg) -> isize;
                }
                unsafe {
                    let mut msg: Msg = std::mem::zeroed();
                    while PeekMessageW(&mut msg, 0, 0, 0, 1) != 0 {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }

            thread::sleep(Duration::from_millis(30));
        }
    })
}
