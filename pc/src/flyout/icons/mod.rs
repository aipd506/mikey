//! Vector icons system for Mikey Flyout based on Lucide SVGs.

#![cfg(windows)]

pub mod canvas;
pub mod media;
pub mod path;
pub mod raster;
pub mod svg;
pub mod system;

pub use canvas::{create_vector_pen, SvgCanvas};
pub use media::{
    draw_hero_camera, draw_hero_flip, draw_hero_mic, draw_hero_preview, draw_hero_sound,
};
pub use raster::{create_mikey_tray_icon, generate_mikey_tray_rgba, TrayIconMode};
pub use svg::render_svg;
pub use system::{
    draw_hero_alert, draw_hero_disconnect, draw_hero_folder, draw_hero_power, draw_hero_sliders,
};
