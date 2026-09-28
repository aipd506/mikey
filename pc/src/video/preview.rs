use super::decoder::DecodedFrame;
use minifb::{Key, Scale, Window, WindowOptions};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

pub type FrameSlot = Option<Arc<DecodedFrame>>;

const TITLE: &str = "Mikey — Camera Preview";

pub struct PreviewWindow {
    visible: Arc<AtomicBool>,
    current_frame: Arc<Mutex<FrameSlot>>,
    thread: OnceLock<thread::Thread>,
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
            thread: OnceLock::new(),
        }
    }

    pub fn is_visible(&self) -> bool {
        self.visible.load(Ordering::Relaxed)
    }

    pub fn set_visible(&self, show: bool) {
        self.visible.store(show, Ordering::Relaxed);
        self.wake();
    }

    pub fn toggle(&self) -> bool {
        let next = !self.visible.load(Ordering::Relaxed);
        self.set_visible(next);
        next
    }

    /// Hands the newest frame to the window, when it's open. The window thread converts it.
    pub fn update_frame(&self, frame: &Arc<DecodedFrame>) {
        if self.visible.load(Ordering::Relaxed) {
            *self.current_frame.lock().unwrap() = Some(Arc::clone(frame));
            self.wake();
        }
    }

    fn wake(&self) {
        if let Some(t) = self.thread.get() {
            t.unpark();
        }
    }

    /// Spawns the dedicated preview window thread.
    pub fn start_thread(&self, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
        let visible = Arc::clone(&self.visible);
        let current_frame = Arc::clone(&self.current_frame);

        let handle = thread::spawn(move || {
            let mut window: Option<Window> = None;
            // minifb repaints from this buffer between updates, so it lives as long as the window
            // and is only resized once the window is gone.
            let mut pixels: Vec<u32> = Vec::new();
            let (mut win_width, mut win_height) = (0, 0);

            while running.load(Ordering::Relaxed) {
                if !visible.load(Ordering::Relaxed) {
                    window = None;
                    thread::park_timeout(Duration::from_millis(500));
                    continue;
                }

                let frame = current_frame.lock().unwrap().take();
                if let Some(frame) = frame {
                    if window.is_none() || (win_width, win_height) != (frame.width, frame.height) {
                        drop(window.take());
                        (win_width, win_height) = (frame.width, frame.height);
                        window = open_window(win_width, win_height, Scale::FitScreen);
                    }
                    frame.fill_rgb32(&mut pixels);
                    if let Some(win) = window.as_mut() {
                        if let Err(e) = win.update_with_buffer(&pixels, win_width, win_height) {
                            eprintln!("[preview] Buffer update error: {}", e);
                        }
                    }
                } else if window.is_none() {
                    // Empty placeholder window while waiting for camera frames
                    (win_width, win_height) = (640, 360);
                    pixels.clear();
                    pixels.resize(win_width * win_height, 0x00111111);
                    window = open_window(win_width, win_height, Scale::X1);
                    if let Some(win) = window.as_mut() {
                        let _ = win.update_with_buffer(&pixels, win_width, win_height);
                    }
                } else if let Some(win) = window.as_mut() {
                    // Handles input and repaints; minifb paces it to the target fps.
                    win.update();
                }

                match window.as_ref() {
                    Some(win) if win.is_open() && !win.is_key_down(Key::Escape) => {}
                    _ => {
                        visible.store(false, Ordering::Relaxed);
                        window = None;
                    }
                }
            }
        });
        let _ = self.thread.set(handle.thread().clone());
        handle
    }
}

fn open_window(width: usize, height: usize, scale: Scale) -> Option<Window> {
    let opts = WindowOptions {
        resize: true,
        scale,
        ..Default::default()
    };
    match Window::new(TITLE, width, height, opts) {
        Ok(mut win) => {
            win.set_target_fps(60);
            Some(win)
        }
        Err(e) => {
            eprintln!("[preview] Failed to open preview window: {}", e);
            None
        }
    }
}
