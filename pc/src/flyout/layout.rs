//! Window sizing, rounding, and screen positioning calculations for the flyout.

#![cfg(windows)]

use super::types::*;
use super::win32;
use crate::session::SessionManager;
use crate::video::VideoPipeline;

pub fn compute_flyout_height(
    session_mgr: &SessionManager,
    video_pipe: &VideoPipeline,
    settings_expanded: bool,
) -> i32 {
    let mut h = FLYOUT_HEIGHT_COLLAPSED;
    if settings_expanded {
        h += 80;
    }
    if video_pipe.is_camera_on() && !video_pipe.is_preview_visible() {
        h += 110;
    }
    if !session_mgr.list_pending().is_empty() {
        h += 32;
    }
    if !crate::audio::sink::virtual_device_ready() {
        h += 30;
    }
    h
}

pub fn update_window_clip_region(hwnd: win32::HWND, height: i32) {
    unsafe {
        let rgn = win32::CreateRoundRectRgn(
            0,
            0,
            FLYOUT_WIDTH + 1,
            height + 1,
            FLYOUT_CORNER_RADIUS * 2,
            FLYOUT_CORNER_RADIUS * 2,
        );
        win32::SetWindowRgn(hwnd, rgn, 1);
    }
}

pub fn create_flyout_hwnd() -> win32::HWND {
    let class_name = to_wide(FLYOUT_CLASS);
    unsafe {
        let wc = win32::WNDCLASSEXW {
            cbSize: std::mem::size_of::<win32::WNDCLASSEXW>() as u32,
            style: 0x0001 | 0x0002,
            lpfnWndProc: Some(super::wndproc::flyout_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: 0,
        };
        win32::RegisterClassExW(&wc);
        win32::CreateWindowExW(
            win32::WS_EX_TOOLWINDOW | win32::WS_EX_TOPMOST,
            class_name.as_ptr(),
            to_wide("Mikey").as_ptr(),
            win32::WS_POPUP,
            -2000,
            -2000,
            FLYOUT_WIDTH,
            FLYOUT_HEIGHT_COLLAPSED,
            0,
            0,
            0,
            std::ptr::null_mut(),
        )
    }
}

pub fn calculate_flyout_position(
    mut tray_x: i32,
    mut tray_y: i32,
    mut tray_w: i32,
    mut tray_h: i32,
    height: i32,
) -> (i32, i32) {
    // If coordinates default/unknown (0, 0), fallback dynamically to cursor position
    if tray_x == 0 && tray_y == 0 {
        let mut pt = win32::POINT { x: 0, y: 0 };
        unsafe {
            win32::GetCursorPos(&mut pt);
        }
        tray_x = pt.x - 8;
        tray_y = pt.y - 8;
        tray_w = 16;
        tray_h = 16;
    }

    let pt = win32::POINT {
        x: tray_x + tray_w / 2,
        y: tray_y + tray_h / 2,
    };

    let hmon = unsafe {
        win32::MonitorFromPoint(pt, 2 /* MONITOR_DEFAULTTONEAREST */)
    };
    let mut mi = win32::MONITORINFO {
        cbSize: std::mem::size_of::<win32::MONITORINFO>() as u32,
        rcMonitor: win32::RECT::default(),
        rcWork: win32::RECT::default(),
        dwFlags: 0,
    };
    unsafe {
        win32::GetMonitorInfoW(hmon, &mut mi);
    }

    let work = mi.rcWork;

    // Center horizontally on tray icon, clamped to current monitor work area
    let mut x = (tray_x + tray_w / 2) - FLYOUT_WIDTH / 2;
    x = x.clamp(work.left + 8, work.right - FLYOUT_WIDTH - 8);

    // Position vertically: above taskbar if taskbar is bottom, below if top
    let y = if tray_y > (work.top + work.bottom) / 2 {
        (tray_y - height - 8).clamp(work.top + 8, work.bottom - height - 8)
    } else {
        (tray_y + tray_h + 8).clamp(work.top + 8, work.bottom - height - 8)
    };

    (x, y)
}
