//! Thread-safe Virtual Camera manager for Mikey.

pub mod api;
#[cfg(windows)]
pub mod install;

use super::decoder::DecodedFrame;
use api::SoftcamApi;
use std::ffi::c_void;
use std::sync::{Mutex, OnceLock};

const FPS: f32 = 30.0;

#[derive(Clone, Copy)]
struct Camera {
    ptr: usize,
    width: usize,
    height: usize,
}

/// Thread-safe Virtual Camera manager.
pub struct VirtualCamera {
    api: OnceLock<Option<SoftcamApi>>,
    camera: Mutex<Option<Camera>>,
}

impl Default for VirtualCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualCamera {
    pub fn new() -> Self {
        Self {
            api: OnceLock::new(),
            camera: Mutex::new(None),
        }
    }

    /// Loads softcam, extracting and registering it on first run, then puts the off frame up so
    /// apps find a working camera before the phone connects. The first run writes files and the
    /// registry, so this runs on a background thread, never on the startup path.
    pub fn load(&self) {
        let api = self.api.get_or_init(|| {
            let api = SoftcamApi::load();
            if api.is_some() {
                println!("[video] Virtual camera backend loaded (softcam.dll found)");
            } else {
                println!(
                    "[video] Virtual camera not installed. Camera preview available; install softcam.dll to use with Zoom/Teams."
                );
            }
            api
        });
        if api.is_some() {
            self.show_off_frame();
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.api.get(), Some(Some(_)))
    }

    /// The neutral frame for when the camera is off (media-pipeline.md), at the size apps already have.
    pub fn show_off_frame(&self) {
        let (w, h) = self
            .camera
            .lock()
            .unwrap()
            .map_or((1280, 720), |c| (c.width, c.height));
        self.push_frame(&DecodedFrame::placeholder(w, h));
    }

    /// Sends a frame to the virtual camera. softcam paces sends to FPS, so this can block for up
    /// to a frame.
    pub fn push_frame(&self, frame: &DecodedFrame) {
        let Some(Some(api)) = self.api.get() else {
            return;
        };
        let mut guard = self.camera.lock().unwrap();
        let same_size = |c: &Camera| (c.width, c.height) == (frame.width, frame.height);

        // An app that has the camera open keeps the size it first saw (media-pipeline.md), so a
        // new size only recreates the camera while nothing is using it.
        let recreate = match &*guard {
            Some(c) => !same_size(c) && !unsafe { (api.is_connected)(c.ptr as *mut c_void) },
            None => true,
        };
        if recreate {
            if let Some(old) = guard.take() {
                unsafe { (api.delete_camera)(old.ptr as *mut c_void) };
            }
            let ptr = unsafe { (api.create_camera)(frame.width as i32, frame.height as i32, FPS) };
            if !ptr.is_null() {
                *guard = Some(Camera {
                    ptr: ptr as usize,
                    width: frame.width,
                    height: frame.height,
                });
            }
        }

        let Some(cam) = *guard else {
            return;
        };
        let ptr = cam.ptr as *mut c_void;
        if same_size(&cam) {
            unsafe { (api.send_frame)(ptr, frame.bgr.as_ptr()) };
        } else {
            let boxed = frame.letterbox(cam.width, cam.height);
            unsafe { (api.send_frame)(ptr, boxed.bgr.as_ptr()) };
        }
    }
}

impl Drop for VirtualCamera {
    fn drop(&mut self) {
        if let (Some(Some(api)), Ok(mut guard)) = (self.api.get(), self.camera.lock()) {
            if let Some(cam) = guard.take() {
                unsafe { (api.delete_camera)(cam.ptr as *mut c_void) };
            }
        }
    }
}
