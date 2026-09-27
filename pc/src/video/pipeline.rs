use super::decoder::{decode_jpeg, DecodedFrame};
use super::preview::PreviewWindow;
use super::vcam::VirtualCamera;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub type JpegFrameSlot = Option<(Vec<u8>, u32, u64)>;
pub type LastFrameSlot = Option<DecodedFrame>;

pub struct VideoPipeline {
    vcam: Arc<VirtualCamera>,
    preview: Arc<PreviewWindow>,
    camera_on: Arc<AtomicBool>,
    last_frame: Arc<Mutex<LastFrameSlot>>,
    latest_jpeg: Arc<Mutex<JpegFrameSlot>>,
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
            latest_jpeg: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_vcam_available(&self) -> bool {
        self.vcam.is_available()
    }

    pub fn is_camera_on(&self) -> bool {
        self.camera_on.load(Ordering::Relaxed)
    }

    pub fn get_last_frame(&self) -> Option<DecodedFrame> {
        let guard = self.last_frame.lock().ok()?;
        guard.clone()
    }

    pub fn set_camera_active(&self, on: bool) {
        self.camera_on.store(on, Ordering::Relaxed);
        if !on {
            // Camera turned off: push neutral privacy placeholder frame per media-pipeline.md
            let placeholder = DecodedFrame::placeholder(1280, 720);
            self.vcam
                .push_bgr_frame(&placeholder.to_bgr(), 1280, 720, 30.0);
            self.preview.update_frame(1280, 720, placeholder.to_rgb32());
            let mut last_guard = self.last_frame.lock().unwrap();
            *last_guard = None;
        }
    }

    /// Pushes an incoming raw JPEG payload from a VIDEO frame (0x02).
    /// Implements KEEP_ONLY_LATEST: Overwrites any pending frame immediately to prevent latency backlog.
    pub fn push_jpeg_frame(&self, jpeg_bytes: Vec<u8>, seq: u32, capture_ts: u64) {
        self.camera_on.store(true, Ordering::Relaxed);
        let mut slot = self.latest_jpeg.lock().unwrap();
        *slot = Some((jpeg_bytes, seq, capture_ts));
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
        let _camera_on = Arc::clone(&self.camera_on);
        let last_frame = Arc::clone(&self.last_frame);
        let latest_jpeg = Arc::clone(&self.latest_jpeg);

        // Also start the preview window thread
        let _prev_handle = preview.start_thread(Arc::clone(&running));

        thread::spawn(move || {
            let mut last_processed_seq: Option<u32> = None;

            while running.load(Ordering::Relaxed) {
                // Take latest incoming JPEG frame if available
                let item = {
                    let mut slot = latest_jpeg.lock().unwrap();
                    slot.take()
                };

                if let Some((jpeg_bytes, seq, _ts)) = item {
                    if last_processed_seq == Some(seq) {
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    last_processed_seq = Some(seq);

                    match decode_jpeg(&jpeg_bytes) {
                        Ok(frame) => {
                            let bgr = frame.to_bgr();
                            let rgb32 = frame.to_rgb32();

                            // Push to virtual camera and preview
                            vcam.push_bgr_frame(&bgr, frame.width, frame.height, 30.0);
                            preview.update_frame(frame.width, frame.height, rgb32);

                            let mut last_guard = last_frame.lock().unwrap();
                            *last_guard = Some(frame);
                        }
                        Err(e) => {
                            eprintln!("[video] Failed to decode frame #{}: {}", seq, e);
                        }
                    }
                } else {
                    // No new frame available: sleep briefly (sub-frame delay ~2ms)
                    thread::sleep(Duration::from_millis(2));
                }
            }
        })
    }
}
