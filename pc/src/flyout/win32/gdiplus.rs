//! Dynamic loader for gdiplus.dll - zero link-time dependencies, 100% portable.

#![cfg(windows)]

use std::ffi::c_void;

#[repr(C)]
pub struct GdiplusStartupInput {
    pub gdiplus_version: u32,
    pub debug_event_callback: usize,
    pub suppress_background_thread: i32,
    pub suppress_external_codecs: i32,
}

impl Default for GdiplusStartupInput {
    fn default() -> Self {
        Self {
            gdiplus_version: 1,
            debug_event_callback: 0,
            suppress_background_thread: 0,
            suppress_external_codecs: 0,
        }
    }
}

pub type FnStartup =
    unsafe extern "system" fn(*mut usize, *const GdiplusStartupInput, *mut c_void) -> i32;
pub type FnShutdown = unsafe extern "system" fn(usize);
pub type FnCreateFromHDC = unsafe extern "system" fn(super::HDC, *mut *mut c_void) -> i32;
pub type FnDeleteGraphics = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnSetSmoothingMode = unsafe extern "system" fn(*mut c_void, i32) -> i32;
pub type FnSetTextRenderingHint = unsafe extern "system" fn(*mut c_void, i32) -> i32;
pub type FnCreateSolidFill = unsafe extern "system" fn(u32, *mut *mut c_void) -> i32;
pub type FnDeleteBrush = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnCreatePen1 = unsafe extern "system" fn(u32, f32, i32, *mut *mut c_void) -> i32;
pub type FnDeletePen = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnSetPenCap = unsafe extern "system" fn(*mut c_void, i32) -> i32;
pub type FnSetPenLineJoin = unsafe extern "system" fn(*mut c_void, i32) -> i32;
pub type FnDrawLine =
    unsafe extern "system" fn(*mut c_void, *mut c_void, f32, f32, f32, f32) -> i32;
pub type FnDrawArc =
    unsafe extern "system" fn(*mut c_void, *mut c_void, f32, f32, f32, f32, f32, f32) -> i32;
pub type FnFillEllipse =
    unsafe extern "system" fn(*mut c_void, *mut c_void, f32, f32, f32, f32) -> i32;
pub type FnDrawEllipse =
    unsafe extern "system" fn(*mut c_void, *mut c_void, f32, f32, f32, f32) -> i32;
pub type FnCreatePath = unsafe extern "system" fn(i32, *mut *mut c_void) -> i32;
pub type FnDeletePath = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnResetPath = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnAddPathArc = unsafe extern "system" fn(*mut c_void, f32, f32, f32, f32, f32, f32) -> i32;
pub type FnAddPathLine = unsafe extern "system" fn(*mut c_void, f32, f32, f32, f32) -> i32;
pub type FnClosePathFigure = unsafe extern "system" fn(*mut c_void) -> i32;
pub type FnFillPath = unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32;
pub type FnDrawPath = unsafe extern "system" fn(*mut c_void, *mut c_void, *mut c_void) -> i32;

pub struct Gdiplus {
    _hmod: usize,
    pub token: usize,
    pub fn_shutdown: FnShutdown,
    pub fn_create_from_hdc: FnCreateFromHDC,
    pub fn_delete_graphics: FnDeleteGraphics,
    pub fn_set_smoothing_mode: FnSetSmoothingMode,
    pub fn_set_text_rendering_hint: FnSetTextRenderingHint,
    pub fn_create_solid_fill: FnCreateSolidFill,
    pub fn_delete_brush: FnDeleteBrush,
    pub fn_create_pen: FnCreatePen1,
    pub fn_delete_pen: FnDeletePen,
    pub fn_set_pen_start_cap: FnSetPenCap,
    pub fn_set_pen_end_cap: FnSetPenCap,
    pub fn_set_pen_line_join: FnSetPenLineJoin,
    pub fn_draw_line: FnDrawLine,
    pub fn_draw_arc: FnDrawArc,
    pub fn_fill_ellipse: FnFillEllipse,
    pub fn_draw_ellipse: FnDrawEllipse,
    pub fn_create_path: FnCreatePath,
    pub fn_delete_path: FnDeletePath,
    pub fn_reset_path: FnResetPath,
    pub fn_add_path_arc: FnAddPathArc,
    pub fn_add_path_line: FnAddPathLine,
    pub fn_close_path_figure: FnClosePathFigure,
    pub fn_fill_path: FnFillPath,
    pub fn_draw_path: FnDrawPath,
}

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryA(lpLibFileName: *const u8) -> usize;
    fn GetProcAddress(hModule: usize, lpProcName: *const std::ffi::c_char) -> *const c_void;
}

#[allow(clippy::manual_c_str_literals)]
impl Gdiplus {
    pub fn init() -> Option<Self> {
        unsafe {
            let hmod = LoadLibraryA(b"gdiplus.dll\0".as_ptr());
            if hmod == 0 {
                return None;
            }
            macro_rules! sym {
                ($name:expr, $ty:ty) => {{
                    let p = GetProcAddress(hmod, $name.as_ptr() as *const std::ffi::c_char);
                    if p.is_null() {
                        return None;
                    }
                    let f: $ty = std::mem::transmute(p);
                    f
                }};
            }
            let fn_startup: FnStartup = sym!(b"GdiplusStartup\0", FnStartup);
            let fn_shutdown: FnShutdown = sym!(b"GdiplusShutdown\0", FnShutdown);
            let mut token: usize = 0;
            let input = GdiplusStartupInput::default();
            if fn_startup(&mut token, &input, std::ptr::null_mut()) != 0 {
                return None;
            }
            Some(Self {
                _hmod: hmod,
                token,
                fn_shutdown,
                fn_create_from_hdc: sym!(b"GdipCreateFromHDC\0", FnCreateFromHDC),
                fn_delete_graphics: sym!(b"GdipDeleteGraphics\0", FnDeleteGraphics),
                fn_set_smoothing_mode: sym!(b"GdipSetSmoothingMode\0", FnSetSmoothingMode),
                fn_set_text_rendering_hint: sym!(
                    b"GdipSetTextRenderingHint\0",
                    FnSetTextRenderingHint
                ),
                fn_create_solid_fill: sym!(b"GdipCreateSolidFill\0", FnCreateSolidFill),
                fn_delete_brush: sym!(b"GdipDeleteBrush\0", FnDeleteBrush),
                fn_create_pen: sym!(b"GdipCreatePen1\0", FnCreatePen1),
                fn_delete_pen: sym!(b"GdipDeletePen\0", FnDeletePen),
                fn_set_pen_start_cap: sym!(b"GdipSetPenStartCap\0", FnSetPenCap),
                fn_set_pen_end_cap: sym!(b"GdipSetPenEndCap\0", FnSetPenCap),
                fn_set_pen_line_join: sym!(b"GdipSetPenLineJoin\0", FnSetPenLineJoin),
                fn_draw_line: sym!(b"GdipDrawLine\0", FnDrawLine),
                fn_draw_arc: sym!(b"GdipDrawArc\0", FnDrawArc),
                fn_fill_ellipse: sym!(b"GdipFillEllipse\0", FnFillEllipse),
                fn_draw_ellipse: sym!(b"GdipDrawEllipse\0", FnDrawEllipse),
                fn_create_path: sym!(b"GdipCreatePath\0", FnCreatePath),
                fn_delete_path: sym!(b"GdipDeletePath\0", FnDeletePath),
                fn_reset_path: sym!(b"GdipResetPath\0", FnResetPath),
                fn_add_path_arc: sym!(b"GdipAddPathArc\0", FnAddPathArc),
                fn_add_path_line: sym!(b"GdipAddPathLine\0", FnAddPathLine),
                fn_close_path_figure: sym!(b"GdipClosePathFigure\0", FnClosePathFigure),
                fn_fill_path: sym!(b"GdipFillPath\0", FnFillPath),
                fn_draw_path: sym!(b"GdipDrawPath\0", FnDrawPath),
            })
        }
    }
}

impl Drop for Gdiplus {
    fn drop(&mut self) {
        unsafe {
            (self.fn_shutdown)(self.token);
        }
    }
}

#[test]
fn test_gdiplus_init() {
    let g = Gdiplus::init();
    assert!(g.is_some(), "Gdiplus::init() returned None!");
}
