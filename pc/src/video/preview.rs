use minifb::{Key, Scale, Window, WindowOptions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub type FrameBufferData = Option<(usize, usize, Vec<u32>)>;

pub struct PreviewWindow {
    visible: Arc<AtomicBool>,
    current_frame: Arc<Mutex<FrameBufferData>>,
}

impl Default for PreviewWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewWindow {
    pub fn new() -> Self {
        Self {
            visible: Arc::new(AtomicBool::new(false)),
            current_frame: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn set_visible(&self, show: bool) {
        self.visible.store(show, Ordering::Relaxed);
    }

    pub fn toggle(&self) -> bool {
        let current = self.visible.load(Ordering::Relaxed);
        let next = !current;
        self.visible.store(next, Ordering::Relaxed);
        next
    }

    pub fn update_frame(&self, width: usize, height: usize, rgb32: Vec<u32>) {
        if self.visible.load(Ordering::Relaxed) {
            let mut guard = self.current_frame.lock().unwrap();
            *guard = Some((width, height, rgb32));
        }
    }

    /// Spawns the dedicated preview window thread.
    pub fn start_thread(&self, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
        let visible = Arc::clone(&self.visible);
        let current_frame = Arc::clone(&self.current_frame);

        thread::spawn(move || {
            let mut window: Option<Window> = None;
            let mut win_width = 640;
            let mut win_height = 360;

            while running.load(Ordering::Relaxed) {
                let is_vis = visible.load(Ordering::Relaxed);

                if is_vis {
                    // Check if we have an updated frame
                    let frame_data = {
                        let guard = current_frame.lock().unwrap();
                        guard.clone()
                    };

                    if let Some((w, h, buffer)) = frame_data {
                        if window.is_none() || win_width != w || win_height != h {
                            win_width = w;
                            win_height = h;
                            let opts = WindowOptions {
                                resize: true,
                                scale: Scale::FitScreen,
                                ..Default::default()
                            };

                            match Window::new("Mikey — Camera Preview", win_width, win_height, opts)
                            {
                                Ok(mut win) => {
                                    win.set_target_fps(60);
                                    window = Some(win);
                                }
                                Err(e) => {
                                    eprintln!("[preview] Failed to open preview window: {}", e);
                                    visible.store(false, Ordering::Relaxed);
                                    continue;
                                }
                            }
                        }

                        if let Some(ref mut win) = window {
                            if !win.is_open() || win.is_key_down(Key::Escape) {
                                visible.store(false, Ordering::Relaxed);
                                window = None;
                                thread::sleep(Duration::from_millis(100));
                                continue;
                            }

                            if let Err(e) = win.update_with_buffer(&buffer, win_width, win_height) {
                                eprintln!("[preview] Buffer update error: {}", e);
                            }
                        }
                    } else {
                        // Empty placeholder window while waiting for camera frames
                        if window.is_none() {
                            let opts = WindowOptions {
                                resize: true,
                                ..Default::default()
                            };
                            if let Ok(mut win) =
                                Window::new("Mikey — Camera Preview", 640, 360, opts)
                            {
                                win.set_target_fps(20);
                                let blank = vec![0x00111111u32; 640 * 360];
                                let _ = win.update_with_buffer(&blank, 640, 360);
                                window = Some(win);
                            }
                        } else if let Some(ref mut win) = window {
                            if !win.is_open() || win.is_key_down(Key::Escape) {
                                visible.store(false, Ordering::Relaxed);
                                window = None;
                            } else {
                                win.update();
                            }
                        }
                        thread::sleep(Duration::from_millis(30));
                    }
                } else {
                    // Hidden: destroy window and sleep
                    if window.is_some() {
                        window = None;
                    }
                    thread::sleep(Duration::from_millis(100));
                }
            }
        })
    }
}
