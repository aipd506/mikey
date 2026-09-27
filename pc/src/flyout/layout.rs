//! Window sizing, rounding, and screen positioning calculations for the flyout.

#![cfg(windows)]

use super::types::*;
use super::win32;
use crate::session::SessionManager;
use crate::video::VideoPipeline;

pub fn compute_flyout_height(session_mgr: &SessionManager, video_pipe: &VideoPipeline) -> i32 {
    let mut h = if video_pipe.is_camera_on() {
        FLYOUT_HEIGHT_EXPANDED
    } else {
        FLYOUT_HEIGHT_COLLAPSED
    };
    if !session_mgr.list_pending().is_empty() {
        h += 38;
    }
    let (virt_ready, _) = crate::audio::sink::check_virtual_device_status();
    if !virt_ready {
        h += 34;
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
