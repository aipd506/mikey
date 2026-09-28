//! One Mikey per user session. Launching it again opens the running one's flyout, instead of
//! a second copy that can't bind the ports and quits without a word.

#![cfg(windows)]

use crate::flyout::{to_wide, FLYOUT_CLASS, WM_APP_SHOW};
use std::ffi::c_void;

#[link(name = "kernel32")]
extern "system" {
    fn CreateMutexW(attributes: *const c_void, initial_owner: i32, name: *const u16) -> usize;
    fn GetLastError() -> u32;
}

#[link(name = "user32")]
extern "system" {
    fn FindWindowW(class_name: *const u16, window_name: *const u16) -> usize;
    fn GetWindowThreadProcessId(hwnd: usize, process_id: *mut u32) -> u32;
    fn AllowSetForegroundWindow(process_id: u32) -> i32;
    fn PostMessageW(hwnd: usize, msg: u32, wparam: usize, lparam: isize) -> i32;
}

const ERROR_ALREADY_EXISTS: u32 = 183;

/// True when Mikey already runs in this session. The mutex handle is never closed, so it marks
/// this process until it exits.
pub fn already_running() -> bool {
    let name = to_wide("Local\\MikeyPcTray");
    unsafe {
        CreateMutexW(std::ptr::null(), 0, name.as_ptr()) != 0
            && GetLastError() == ERROR_ALREADY_EXISTS
    }
}

/// Asks the running Mikey to open its flyout. Windows only lets the process the user just
/// started take focus, so this one passes that right on first.
pub fn open_running() {
    let class_name = to_wide(FLYOUT_CLASS);
    unsafe {
        let hwnd = FindWindowW(class_name.as_ptr(), std::ptr::null());
        if hwnd == 0 {
            return;
        }
        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, &mut process_id);
        AllowSetForegroundWindow(process_id);
        PostMessageW(hwnd, WM_APP_SHOW, 0, 0);
    }
}
