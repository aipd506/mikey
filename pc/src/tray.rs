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
    tooltip: String,
}

impl TrayApp {
    /// None only if Mikey quits first. At login the taskbar can come up after Mikey, and adding
    /// the icon fails until it does, so keep trying instead of giving up.
    pub fn new(session_manager: SessionManager, running: &AtomicBool) -> Option<Self> {
        let icon_grey = create_mikey_tray_icon(TrayIconMode::Idle);
        let icon_green = create_mikey_tray_icon(TrayIconMode::Active);
        let icon_amber = create_mikey_tray_icon(TrayIconMode::Pending);

        // No OS popup context menu: clicking tray icon directly toggles custom Mikey Flyout
        let tray = loop {
            match TrayIconBuilder::new()
                .with_tooltip("Mikey - Phone Mic & Webcam")
                .with_icon(icon_grey.clone())
                .build()
            {
                Ok(tray) => break tray,
                Err(e) => eprintln!("[tray] Taskbar not ready, retrying: {}", e),
            }
            if !running.load(Ordering::Relaxed) {
                return None;
            }
            thread::sleep(Duration::from_secs(1));
        };

        Some(Self {
            tray,
            session_manager,
            icon_grey,
            icon_green,
            icon_amber,
            tooltip: String::new(),
        })
    }

    /// Changes the icon only when the state does: every change makes Explorer redraw the tray.
    pub fn update_state(&mut self) {
        let has_pending = !self.session_manager.list_pending().is_empty();
        let session = self
            .session_manager
            .active_session()
            .filter(|_| self.session_manager.is_active());

        let (icon, tooltip) = if has_pending {
            (&self.icon_amber, "Mikey - Waiting for approval".to_string())
        } else if let Some(session) = session {
            let tip = format!(
                "Mikey - {} streaming via L{}",
                session.device_name, session.current_level
            );
            (&self.icon_green, tip)
        } else {
            (&self.icon_grey, "Mikey - Idle".to_string())
        };
        if tooltip != self.tooltip {
            let _ = self.tray.set_icon(Some(icon.clone()));
            let _ = self.tray.set_tooltip(Some(&tooltip));
            self.tooltip = tooltip;
        }
    }
}

/// Runs the system tray and flyout companion in a dedicated GUI thread. `open_now` shows the
/// flyout as soon as the icon is up, so a launch by hand visibly opens something.
pub fn start_tray_thread(
    session_manager: SessionManager,
    video_pipeline: Arc<VideoPipeline>,
    jitter_buffer: Arc<JitterBuffer>,
    running: Arc<AtomicBool>,
    open_now: bool,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let Some(mut tray_app) = TrayApp::new(session_manager.clone(), &running) else {
            return;
        };
        let mut flyout = FlyoutWindow::new(
            session_manager,
            video_pipeline,
            jitter_buffer,
            Arc::clone(&running),
        );
        let mut open = open_now;
        while running.load(Ordering::Relaxed) {
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

            if (std::mem::take(&mut open) | flyout.take_show_request()) && !flyout.is_visible() {
                // No rect (icon hidden in the overflow) opens it by the cursor instead.
                let r = tray_app.tray.rect().unwrap_or_default();
                flyout.toggle(
                    r.position.x as i32,
                    r.position.y as i32,
                    r.size.width as i32,
                    r.size.height as i32,
                );
            }
            tray_app.update_state();
            wait_and_dispatch_messages();
        }
    })
}

/// Sleeps until a window message comes in (tray clicks, flyout input and timers), at most
/// 250 ms so the icon still follows the session, then handles every waiting message.
fn wait_and_dispatch_messages() {
    use crate::launch::{Msg, PeekMessageW};
    #[link(name = "user32")]
    extern "system" {
        fn MsgWaitForMultipleObjects(
            nCount: u32,
            pHandles: *const usize,
            fWaitAll: i32,
            dwMilliseconds: u32,
            dwWakeMask: u32,
        ) -> u32;
        fn TranslateMessage(lpMsg: *const Msg) -> i32;
        fn DispatchMessageW(lpMsg: *const Msg) -> isize;
    }
    const QS_ALLINPUT: u32 = 0x04FF;
    const PM_REMOVE: u32 = 1;
    unsafe {
        MsgWaitForMultipleObjects(0, std::ptr::null(), 0, 250, QS_ALLINPUT);
        let mut msg: Msg = std::mem::zeroed();
        while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
