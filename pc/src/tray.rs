use crate::audio::pipeline::JitterBuffer;
use crate::flyout::FlyoutWindow;
use crate::session::SessionManager;
use crate::video::VideoPipeline;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};

// Icon colors matching vibe/rules/design-language.md
pub const COLOR_IDLE_GREY: (u8, u8, u8) = (142, 142, 147);
pub const COLOR_STREAMING_GREEN: (u8, u8, u8) = (48, 209, 88);
pub const COLOR_PENDING_AMBER: (u8, u8, u8) = (255, 214, 10);

pub fn create_dot_icon(r: u8, g: u8, b: u8) -> Icon {
    const SIZE: u32 = 16;
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    let center = (SIZE as f32 - 1.0) / 2.0;
    let radius = 6.0f32;

    for y in 0..SIZE {
        for x in 0..SIZE {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius - 0.5 {
                rgba.extend_from_slice(&[r, g, b, 255]);
            } else if dist <= radius + 0.5 {
                let alpha = ((radius + 0.5 - dist) * 255.0) as u8;
                rgba.extend_from_slice(&[r, g, b, alpha]);
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, SIZE, SIZE).expect("create RGBA icon")
}

pub struct TrayApp {
    tray: TrayIcon,
    session_manager: SessionManager,
    icon_grey: Icon,
    icon_green: Icon,
    icon_amber: Icon,
}

impl TrayApp {
    pub fn new(session_manager: SessionManager) -> Self {
        let icon_grey = create_dot_icon(COLOR_IDLE_GREY.0, COLOR_IDLE_GREY.1, COLOR_IDLE_GREY.2);
        let icon_green = create_dot_icon(
            COLOR_STREAMING_GREEN.0,
            COLOR_STREAMING_GREEN.1,
            COLOR_STREAMING_GREEN.2,
        );
        let icon_amber = create_dot_icon(
            COLOR_PENDING_AMBER.0,
            COLOR_PENDING_AMBER.1,
            COLOR_PENDING_AMBER.2,
        );

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
