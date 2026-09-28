//! Automatic per-user DirectShow virtual camera installation and registry management.

#![cfg(windows)]

use std::ffi::c_void;
use std::path::{Path, PathBuf};

static EMBEDDED_SOFTCAM_DLL: &[u8] = include_bytes!("../../../softcam.dll");

const HKEY_CLASSES_ROOT: usize = 0x8000_0000;
const HKEY_CURRENT_USER: usize = 0x8000_0001;

#[link(name = "advapi32")]
extern "system" {
    fn RegCreateKeyExW(
        hKey: usize,
        lpSubKey: *const u16,
        reserved: u32,
        lpClass: *mut u16,
        dwOptions: u32,
        samDesired: u32,
        lpSecurityAttributes: *mut c_void,
        phkResult: *mut usize,
        lpdwDisposition: *mut u32,
    ) -> i32;
    fn RegOpenKeyExW(
        hKey: usize,
        lpSubKey: *const u16,
        ulOptions: u32,
        samDesired: u32,
        phkResult: *mut usize,
    ) -> i32;
    fn RegSetValueExW(
        hKey: usize,
        lpValueName: *const u16,
        reserved: u32,
        dwType: u32,
        lpData: *const u8,
        cbData: u32,
    ) -> i32;
    fn RegOverridePredefKey(hKey: usize, hNewHKey: usize) -> i32;
    fn RegCloseKey(hKey: usize) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryW(lpLibFileName: *const u16) -> usize;
    fn GetProcAddress(hModule: usize, lpProcName: *const std::ffi::c_char) -> *const c_void;
    fn FreeLibrary(hModule: usize) -> i32;
}

pub fn ensure_softcam_installed() -> Option<PathBuf> {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(dir) = exe_path.parent() {
            let next_to_exe = dir.join("softcam.dll");
            if next_to_exe.exists() {
                return Some(next_to_exe);
            }
        }
    }

    let local_app_data = std::env::var_os("LOCALAPPDATA")?;
    let mikey_dir = PathBuf::from(local_app_data).join("Mikey");
    let target_dll = mikey_dir.join("softcam.dll");

    let needs_write = match std::fs::metadata(&target_dll) {
        Ok(meta) => meta.len() != EMBEDDED_SOFTCAM_DLL.len() as u64,
        Err(_) => true,
    };

    if needs_write {
        let _ = std::fs::create_dir_all(&mikey_dir);
        if let Err(e) = std::fs::write(&target_dll, EMBEDDED_SOFTCAM_DLL) {
            eprintln!(
                "[vcam] Failed to extract softcam.dll to {:?}: {}",
                target_dll, e
            );
            return None;
        }
    }

    Some(target_dll)
}

pub fn ensure_directshow_registered(dll_path: &Path) {
    let instance_path =
        "Software\\Classes\\CLSID\\{860BB310-5D01-11D0-BD3B-00A0C911CE86}\\Instance\\DirectShow Softcam\0";
    let subkey_instance: Vec<u16> = instance_path.encode_utf16().collect();
    let mut h_check = 0usize;

    let already_registered = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_instance.as_ptr(),
            0,
            0x20019,
            &mut h_check,
        ) == 0
    };

    if already_registered {
        unsafe { RegCloseKey(h_check) };
        return;
    }

    let classes_subkey: Vec<u16> = "Software\\Classes\0".encode_utf16().collect();
    let mut hkcu_classes = 0usize;
    let create_res = unsafe {
        RegCreateKeyExW(
            HKEY_CURRENT_USER,
            classes_subkey.as_ptr(),
            0,
            std::ptr::null_mut(),
            0,
            0x2001F,
            std::ptr::null_mut(),
            &mut hkcu_classes,
            std::ptr::null_mut(),
        )
    };
    if create_res != 0 {
        return;
    }

    unsafe {
        RegOverridePredefKey(HKEY_CLASSES_ROOT, hkcu_classes);
    }

    let wide_dll_path: Vec<u16> = dll_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let h_mod = unsafe { LoadLibraryW(wide_dll_path.as_ptr()) };
    if h_mod != 0 {
        unsafe {
            let p_reg = GetProcAddress(h_mod, c"DllRegisterServer".as_ptr());
            if !p_reg.is_null() {
                type FnDllRegisterServer = unsafe extern "system" fn() -> i32;
                let reg_fn: FnDllRegisterServer = std::mem::transmute(p_reg);
                let _ = reg_fn();
            }
            FreeLibrary(h_mod);
        }
    }

    unsafe {
        RegOverridePredefKey(HKEY_CLASSES_ROOT, 0);
        RegCloseKey(hkcu_classes);
    }

    let mut h_instance = 0usize;
    let open_res = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey_instance.as_ptr(),
            0,
            0x20006,
            &mut h_instance,
        )
    };
    if open_res == 0 {
        let friendly_name: Vec<u16> = "Mikey Camera\0".encode_utf16().collect();
        let friendly_val_name: Vec<u16> = "FriendlyName\0".encode_utf16().collect();
        unsafe {
            RegSetValueExW(
                h_instance,
                friendly_val_name.as_ptr(),
                0,
                1,
                friendly_name.as_ptr() as *const u8,
                (friendly_name.len() * 2) as u32,
            );
            RegCloseKey(h_instance);
        }
    }
}
