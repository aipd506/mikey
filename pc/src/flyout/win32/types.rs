//! Raw Win32 GDI and User32 types and constants for Mikey Flyout.

#![cfg(windows)]
#![allow(non_snake_case)]

pub type HWND = usize;
pub type HDC = usize;
pub type HBRUSH = usize;
pub type HPEN = usize;
pub type HFONT = usize;
pub type HBITMAP = usize;
pub type HGDIOBJ = usize;
pub type HMODULE = usize;
pub type HRGN = usize;
pub type LPARAM = isize;
pub type WPARAM = usize;
pub type LRESULT = isize;
pub type WNDPROC = Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>;

pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_POPUP: u32 = 0x80000000;
pub const SW_HIDE: i32 = 0;
pub const SWP_SHOWWINDOW: u32 = 0x0040;
pub const SWP_NOMOVE: u32 = 0x0002;
pub const SWP_NOSIZE: u32 = 0x0001;
pub const SWP_NOZORDER: u32 = 0x0004;
pub const SWP_NOACTIVATE: u32 = 0x0010;
pub const SRCCOPY: u32 = 0x00CC0020;
pub const TRANSPARENT: i32 = 1;
pub const PS_SOLID: i32 = 0;
pub const GWLP_USERDATA: i32 = -21;

pub const WM_DESTROY: u32 = 0x0002;
pub const WM_PAINT: u32 = 0x000F;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WA_INACTIVE: usize = 0;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_KILLFOCUS: u32 = 0x0008;
pub const WM_MOUSEMOVE: u32 = 0x0200;
pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const VK_ESCAPE: usize = 0x1B;

pub const DT_SINGLELINE: u32 = 0x00000020;
pub const DT_CENTER: u32 = 0x00000001;
pub const DT_VCENTER: u32 = 0x00000004;
pub const DT_NOPREFIX: u32 = 0x00000800;
pub const DIB_RGB_COLORS: u32 = 0;
pub const BI_RGB: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: i32,
    pub rcPaint: RECT,
    pub fRestore: i32,
    pub fIncUpdate: i32,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HMODULE,
    pub hIcon: usize,
    pub hCursor: usize,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: usize,
}

#[repr(C)]
pub struct MONITORINFO {
    pub cbSize: u32,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: u32,
}

#[repr(C)]
pub struct BITMAPINFOHEADER {
    pub biSize: u32,
    pub biWidth: i32,
    pub biHeight: i32,
    pub biPlanes: u16,
    pub biBitCount: u16,
    pub biCompression: u32,
    pub biSizeImage: u32,
    pub biXPelsPerMeter: i32,
    pub biYPelsPerMeter: i32,
    pub biClrUsed: u32,
    pub biClrImportant: u32,
}

#[repr(C)]
pub struct BITMAPINFO {
    pub bmiHeader: BITMAPINFOHEADER,
    pub bmiColors: [u32; 1],
}
