//! Media icons: microphone, video camera, flip lens, audio wave, and monitor preview.

#![cfg(windows)]

use super::svg::render_svg;
use crate::flyout::win32::Gdiplus;
use std::ffi::c_void;

const SVG_MIC: &str = include_str!("../../../assets/icons/mic.svg");
const SVG_MIC_OFF: &str = include_str!("../../../assets/icons/mic-off.svg");
const SVG_CAMERA: &str = include_str!("../../../assets/icons/camera.svg");
const SVG_FLIP: &str = include_str!("../../../assets/icons/flip.svg");
const SVG_SOUND: &str = include_str!("../../../assets/icons/sound.svg");
const SVG_PREVIEW: &str = include_str!("../../../assets/icons/preview.svg");

/// Official Lucide mic / mic-off vector icon
pub fn draw_hero_mic(
    g: &Gdiplus,
    graphics: *mut c_void,
    cx: f32,
    cy: f32,
    color: u32,
    is_muted: bool,
) {
    let svg = if is_muted { SVG_MIC_OFF } else { SVG_MIC };
    render_svg(g, graphics, svg, (cx, cy), 20.0, color, 1.8);
}

/// Official Lucide video camera vector icon
pub fn draw_hero_camera(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_CAMERA, (cx, cy), 20.0, color, 1.8);
}

/// Official Lucide switch-camera flip vector icon
pub fn draw_hero_flip(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_FLIP, (cx, cy), 16.0, color, 1.6);
}

/// Official Lucide volume-2 sound wave vector icon
pub fn draw_hero_sound(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_SOUND, (cx, cy), 18.0, color, 1.6);
}

/// Official Lucide monitor preview vector icon
pub fn draw_hero_preview(g: &Gdiplus, graphics: *mut c_void, cx: f32, cy: f32, color: u32) {
    render_svg(g, graphics, SVG_PREVIEW, (cx, cy), 16.0, color, 1.6);
}
