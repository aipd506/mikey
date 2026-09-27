//! Win32 window message procedure and dispatch for Mikey Flyout.

#![cfg(windows)]

use super::win32;
use super::window::FlyoutWindow;

pub(crate) unsafe extern "system" fn flyout_wndproc(
    hwnd: win32::HWND,
    msg: u32,
    wparam: win32::WPARAM,
    lparam: win32::LPARAM,
) -> win32::LRESULT {
    let ptr = win32::GetWindowLongPtrW(hwnd, win32::GWLP_USERDATA) as *mut FlyoutWindow;
    if !ptr.is_null() {
        let flyout = &mut *ptr;
        match msg {
            win32::WM_PAINT => {
                flyout.on_paint();
                return 0;
            }
            win32::WM_TIMER => {
                if flyout.visible {
                    win32::InvalidateRect(hwnd, std::ptr::null(), 0);
                }
                return 0;
            }
            win32::WM_ACTIVATE => {
                let state = wparam & 0xFFFF;
                if state == win32::WA_INACTIVE {
                    // 350 ms grace period: ignores transient focus transfers during open
                    if let Some(shown) = flyout.shown_at {
                        if shown.elapsed().as_millis() > 350 {
                            flyout.hide();
                            return 0;
                        }
                    }
                }
            }
            win32::WM_KEYDOWN => {
                if wparam == win32::VK_ESCAPE {
                    flyout.hide();
                    return 0;
                }
            }
            win32::WM_MOUSEMOVE => {
                let x = (lparam & 0xFFFF) as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                flyout.on_mouse_move(x, y);
                return 0;
            }
            win32::WM_LBUTTONDOWN => {
                let x = (lparam & 0xFFFF) as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                flyout.on_lbutton_down(x, y);
                unsafe {
                    win32::SetFocus(hwnd);
                }
                return 0;
            }
            win32::WM_LBUTTONUP => {
                let x = (lparam & 0xFFFF) as i16 as i32;
                let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
                flyout.on_lbutton_up(x, y);
                return 0;
            }
            win32::WM_DESTROY => {
                return 0;
            }
            _ => {}
        }
    }
    win32::DefWindowProcW(hwnd, msg, wparam, lparam)
}
