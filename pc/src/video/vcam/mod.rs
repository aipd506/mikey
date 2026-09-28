//! Thread-safe Virtual Camera manager for Mikey.

pub mod api;
#[cfg(windows)]
pub mod install;

use api::SoftcamApi;
use std::ffi::c_void;
use std::sync::Mutex;

/// Thread-safe Virtual Camera manager.
pub struct VirtualCamera {
    api: Option<SoftcamApi>,
    camera_ptr: Mutex<Option<usize>>,
    current_width: Mutex<usize>,
    current_height: Mutex<usize>,
}

impl Default for VirtualCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualCamera {
    pub fn new() -> Self {
        let api = SoftcamApi::load();
        if api.is_some() {
            println!("[video] Virtual camera backend loaded (softcam.dll found)");
        } else {
            println!(
                "[video] Virtual camera not installed. Camera preview available; install softcam.dll to use with Zoom/Teams."
            );
        }

        Self {
            api,
            camera_ptr: Mutex::new(None),
            current_width: Mutex::new(0),
            current_height: Mutex::new(0),
        }
    }

    pub fn is_available(&self) -> bool {
        self.api.is_some()
    }

    /// Pushes a BGR frame (3 bytes per pixel) to the virtual camera.
    /// If dimensions change, re-initializes or maintains resolution.
    pub fn push_bgr_frame(&self, bgr: &[u8], width: usize, height: usize, fps: f32) {
        let api = match &self.api {
            Some(a) => a,
            None => return,
        };

        let mut handle_guard = self.camera_ptr.lock().unwrap();
        let mut w_guard = self.current_width.lock().unwrap();
        let mut h_guard = self.current_height.lock().unwrap();

        // Recreate camera handle if dimensions change or uninitialized
        if handle_guard.is_none() || *w_guard != width || *h_guard != height {
            if let Some(old_ptr) = handle_guard.take() {
                unsafe { (api.delete_camera)(old_ptr as *mut c_void) };
            }
            let new_handle = unsafe { (api.create_camera)(width as i32, height as i32, fps) };
            if !new_handle.is_null() {
                *handle_guard = Some(new_handle as usize);
                *w_guard = width;
                *h_guard = height;
            }
        }

        if let Some(ptr) = *handle_guard {
            unsafe {
                (api.send_frame)(ptr as *mut c_void, bgr.as_ptr());
            }
        }
    }
}

impl Drop for VirtualCamera {
    fn drop(&mut self) {
        if let Some(api) = &self.api {
            if let Ok(mut guard) = self.camera_ptr.lock() {
                if let Some(ptr) = guard.take() {
                    unsafe { (api.delete_camera)(ptr as *mut c_void) };
                }
            }
        }
    }
}
