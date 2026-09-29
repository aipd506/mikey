//! System icons: power quit, logs folder, disconnect, sliders, and alert.

#![cfg(windows)]

use super::svg::render_svg;
use crate::flyout::win32::Gdiplus;
use std::ffi::c_void;

const SVG_POWER: &str = include_str!("../../../assets/icons/power.svg");
const SVG_FOLDER: &str = include_str!("../../../assets/icons/folder.svg");
const SVG_DISCONNECT: &str = include_str!("../../../assets/icons/unlink.svg");
const SVG_SLIDERS: &str = include_str!("../../../assets/icons/sliders.svg");
const SVG_ALERT: &str = include_str!("../../../assets/icons/alert.svg");

/// Official Lucide power quit vector icon
pub fn draw_hero_power(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_POWER, (cx, cy), 16.0, color, 1.8);
}

/// Official Lucide folder logs vector icon
pub fn draw_hero_folder(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_FOLDER, (cx, cy), 16.0, color, 1.6);
}

/// Official Lucide unlink disconnect vector icon
pub fn draw_hero_disconnect(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_DISCONNECT, (cx, cy), 16.0, color, 1.8);
}

/// Official Lucide sliders-horizontal vector icon
pub fn draw_hero_sliders(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_SLIDERS, (cx, cy), 16.0, color, 1.6);
}

/// Official Lucide alert-triangle vector icon
pub fn draw_hero_alert(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_ALERT, (cx, cy), 16.0, color, 1.6);
}
