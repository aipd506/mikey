use std::ffi::c_void;
use std::sync::Mutex;

type FnCreateCamera = unsafe extern "C" fn(width: i32, height: i32, framerate: f32) -> *mut c_void;
type FnSendFrame = unsafe extern "C" fn(camera: *mut c_void, frame: *const u8);
type FnDeleteCamera = unsafe extern "C" fn(camera: *mut c_void);

struct SoftcamApi {
    _dll: usize, // HMODULE
    create_camera: FnCreateCamera,
    send_frame: FnSendFrame,
    delete_camera: FnDeleteCamera,
}

#[cfg(windows)]
impl SoftcamApi {
    fn load() -> Option<Self> {
        #[link(name = "kernel32")]
        extern "system" {
            fn LoadLibraryW(lpLibFileName: *const u16) -> usize;
            fn GetProcAddress(hModule: usize, lpProcName: *const std::ffi::c_char)
                -> *const c_void;
            fn FreeLibrary(hModule: usize) -> i32;
        }

        let dll_name: Vec<u16> = "softcam.dll"
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let h_module = unsafe { LoadLibraryW(dll_name.as_ptr()) };
        if h_module == 0 {
            return None;
        }

        unsafe {
            let p_create = GetProcAddress(h_module, c"scCreateCamera".as_ptr());
            let p_send = GetProcAddress(h_module, c"scSendFrame".as_ptr());
            let p_delete = GetProcAddress(h_module, c"scDeleteCamera".as_ptr());

            if p_create.is_null() || p_send.is_null() || p_delete.is_null() {
                FreeLibrary(h_module);
                return None;
            }

            Some(Self {
                _dll: h_module,
                create_camera: std::mem::transmute::<*const c_void, FnCreateCamera>(p_create),
                send_frame: std::mem::transmute::<*const c_void, FnSendFrame>(p_send),
                delete_camera: std::mem::transmute::<*const c_void, FnDeleteCamera>(p_delete),
            })
        }
    }
}

#[cfg(not(windows))]
impl SoftcamApi {
    fn load() -> Option<Self> {
        None
    }
}

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
            println!("[video] Virtual camera not installed. Camera preview available; install softcam.dll to use with Zoom/Teams.");
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
