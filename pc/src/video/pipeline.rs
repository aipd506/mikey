use super::decoder::{decode_jpeg, DecodedFrame};
use super::preview::PreviewWindow;
use super::vcam::VirtualCamera;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub type JpegFrameSlot = Option<(Vec<u8>, u32, u64)>;
pub type LastFrameSlot = Option<Arc<DecodedFrame>>;

pub struct VideoPipeline {
    vcam: Arc<VirtualCamera>,
    preview: Arc<PreviewWindow>,
    camera_on: Arc<AtomicBool>,
    last_frame: Arc<Mutex<LastFrameSlot>>,
    latest_jpeg: Arc<(Mutex<JpegFrameSlot>, Condvar)>,
}

impl Default for VideoPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoPipeline {
    pub fn new() -> Self {
        Self {
            vcam: Arc::new(VirtualCamera::new()),
            preview: Arc::new(PreviewWindow::new()),
            camera_on: Arc::new(AtomicBool::new(false)),
            last_frame: Arc::new(Mutex::new(None)),
            latest_jpeg: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }

    /// Loads the virtual camera. Slow on first run, so call it off the startup path.
    pub fn load_vcam(&self) {
        self.vcam.load();
    }

    pub fn is_vcam_available(&self) -> bool {
        self.vcam.is_available()
    }

    pub fn is_camera_on(&self) -> bool {
        self.camera_on.load(Ordering::Relaxed)
    }

    pub fn get_last_frame(&self) -> Option<Arc<DecodedFrame>> {
        self.last_frame.lock().ok()?.clone()
    }

    pub fn set_camera_active(&self, on: bool) {
        self.camera_on.store(on, Ordering::Relaxed);
        if !on {
            // Camera turned off: push neutral privacy placeholder frame per media-pipeline.md
            self.vcam.show_off_frame();
            self.preview
                .update_frame(&Arc::new(DecodedFrame::placeholder(1280, 720)));
            *self.last_frame.lock().unwrap() = None;
        }
    }

    /// Pushes an incoming raw JPEG payload from a VIDEO frame (0x02).
    /// Implements KEEP_ONLY_LATEST: Overwrites any pending frame immediately to prevent latency backlog.
    pub fn push_jpeg_frame(&self, jpeg_bytes: Vec<u8>, seq: u32, capture_ts: u64) {
        self.camera_on.store(true, Ordering::Relaxed);
        let (slot, ready) = &*self.latest_jpeg;
        *slot.lock().unwrap() = Some((jpeg_bytes, seq, capture_ts));
        ready.notify_one();
    }

    pub fn toggle_preview(&self) -> bool {
        self.preview.toggle()
    }

    pub fn set_preview_visible(&self, visible: bool) {
        self.preview.set_visible(visible);
    }

    pub fn is_preview_visible(&self) -> bool {
        self.preview.is_visible()
    }

    /// Starts the background decoding and dispatch loop on a dedicated thread.
    pub fn start_pipeline_thread(&self, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
        let vcam = Arc::clone(&self.vcam);
        let preview = Arc::clone(&self.preview);
        let last_frame = Arc::clone(&self.last_frame);
        let latest_jpeg = Arc::clone(&self.latest_jpeg);

        // Also start the preview window thread
        let _prev_handle = preview.start_thread(Arc::clone(&running));

        thread::spawn(move || {
            let (slot, ready) = &*latest_jpeg;
            let mut last_processed_seq: Option<u32> = None;
            let mut spare = Vec::new();

            while running.load(Ordering::Relaxed) {
                let item = {
                    let guard = slot.lock().unwrap();
                    let (mut guard, _) = ready
                        .wait_timeout_while(guard, Duration::from_millis(500), |s| s.is_none())
                        .unwrap();
                    guard.take()
                };
                let Some((jpeg_bytes, seq, _ts)) = item else {
                    continue;
                };
                if last_processed_seq == Some(seq) {
                    continue;
                }
                last_processed_seq = Some(seq);

                match decode_jpeg(&jpeg_bytes, std::mem::take(&mut spare)) {
                    Ok(frame) => {
                        let frame = Arc::new(frame);
                        preview.update_frame(&frame);
                        let previous = last_frame.lock().unwrap().replace(Arc::clone(&frame));
                        vcam.push_frame(&frame);
                        // Reuse the previous frame's memory unless the flyout or preview still holds it.
                        if let Some(Ok(old)) = previous.map(Arc::try_unwrap) {
                            spare = old.bgr;
                        }
                    }
                    Err(e) => {
                        eprintln!("[video] Failed to decode frame #{}: {}", seq, e);
                    }
                }
            }
        })
    }
}
