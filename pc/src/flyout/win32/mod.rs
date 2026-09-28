//! Raw Win32 GDI and User32 bindings for custom flyout companion window.

#![cfg(windows)]

pub mod gdiplus;
pub mod types;
pub use gdiplus::*;
pub use types::*;

use std::ffi::c_void;

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: HWND,
        menu: usize,
        instance: HMODULE,
        param: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hwnd: HWND, cmd_show: i32) -> i32;
    pub fn SetWindowPos(
        hwnd: HWND,
        insert_after: HWND,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    pub fn SetForegroundWindow(hwnd: HWND) -> i32;
    pub fn GetForegroundWindow() -> HWND;
    pub fn BringWindowToTop(hwnd: HWND) -> i32;
    pub fn SetFocus(hwnd: HWND) -> HWND;
    pub fn GetCursorPos(point: *mut POINT) -> i32;
    pub fn LoadCursorW(instance: HMODULE, name: *const u16) -> usize;
    pub fn SetWindowRgn(hwnd: HWND, hrgn: HRGN, redraw: i32) -> i32;
    pub fn InvalidateRect(hwnd: HWND, rect: *const RECT, erase: i32) -> i32;
    pub fn BeginPaint(hwnd: HWND, paint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hwnd: HWND, paint: *const PAINTSTRUCT) -> i32;
    pub fn SetTimer(
        hwnd: HWND,
        id_event: usize,
        elapse: u32,
        timer_func: Option<unsafe extern "system" fn(HWND, u32, usize, u32)>,
    ) -> usize;
    pub fn KillTimer(hwnd: HWND, id_event: usize) -> i32;
    pub fn MonitorFromPoint(pt: POINT, flags: u32) -> usize;
    pub fn GetMonitorInfoW(monitor: usize, mi: *mut MONITORINFO) -> i32;
    pub fn SetWindowLongPtrW(hwnd: HWND, index: i32, new_long: isize) -> isize;
    pub fn GetWindowLongPtrW(hwnd: HWND, index: i32) -> isize;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn CreateCompatibleBitmap(hdc: HDC, cx: i32, cy: i32) -> HBITMAP;
    pub fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteObject(obj: HGDIOBJ) -> i32;
    pub fn DeleteDC(hdc: HDC) -> i32;
    pub fn BitBlt(
        hdc_dest: HDC,
        x_dest: i32,
        y_dest: i32,
        width: i32,
        height: i32,
        hdc_src: HDC,
        x_src: i32,
        y_src: i32,
        rop: u32,
    ) -> i32;
    pub fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    pub fn StretchDIBits(
        hdc: HDC,
        x_dest: i32,
        y_dest: i32,
        dest_width: i32,
        dest_height: i32,
        x_src: i32,
        y_src: i32,
        src_width: i32,
        src_height: i32,
        bits: *const c_void,
        bmi: *const BITMAPINFO,
        usage: u32,
        rop: u32,
    ) -> i32;
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn CreatePen(style: i32, width: i32, color: u32) -> HPEN;
    pub fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> HRGN;
    pub fn RoundRect(
        hdc: HDC,
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
        width: i32,
        height: i32,
    ) -> i32;
    pub fn Ellipse(hdc: HDC, left: i32, top: i32, right: i32, bottom: i32) -> i32;
    pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        char_set: u32,
        output_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face_name: *const u16,
    ) -> HFONT;
    pub fn DrawTextW(hdc: HDC, text: *const u16, length: i32, rect: *mut RECT, format: u32) -> i32;
}
