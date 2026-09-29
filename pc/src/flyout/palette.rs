//! Design language palette constants for the Mikey Flyout.

#![cfg(windows)]

use super::types::rgb;

#[inline]
pub const fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

// ARGB colors for smooth anti-aliased GDI+ rendering
pub const ARGB_BG: u32 = argb(255, 13, 13, 15);
pub const ARGB_SURFACE: u32 = argb(255, 20, 20, 24);
pub const ARGB_PILL: u32 = argb(255, 26, 26, 30);
pub const ARGB_PILL_HOVER: u32 = argb(255, 38, 38, 44);
pub const ARGB_BORDER: u32 = argb(255, 38, 38, 42);
pub const ARGB_BORDER_HI: u32 = argb(255, 65, 65, 75);
pub const ARGB_MIC_ON: u32 = argb(255, 48, 209, 88);
pub const ARGB_CAM_ON: u32 = argb(255, 10, 132, 255);
pub const ARGB_ALERT_RED: u32 = argb(255, 255, 69, 58);
pub const ARGB_STATUS_WAIT: u32 = argb(255, 255, 214, 10);
pub const ARGB_TEXT_PRIMARY: u32 = argb(255, 255, 255, 255);
pub const ARGB_TEXT_SECONDARY: u32 = argb(255, 142, 142, 147);
pub const ARGB_TEXT_MUTED: u32 = argb(255, 80, 80, 88);
pub const ARGB_ICON_OFF: u32 = argb(255, 75, 75, 82);

// GDI backward-compatible RGB constants
pub const COLOR_BG_SURFACE: u32 = rgb(13, 13, 15);
pub const COLOR_BG_BLACK: u32 = rgb(0, 0, 0);
pub const COLOR_BORDER: u32 = rgb(38, 38, 42);
pub const COLOR_ICON_OFF: u32 = rgb(75, 75, 82);
pub const COLOR_MIC_ON: u32 = rgb(48, 209, 88);
pub const COLOR_CAM_ON: u32 = rgb(10, 132, 255);
pub const COLOR_STATUS_WAIT: u32 = rgb(255, 214, 10);
pub const COLOR_ALERT_RED: u32 = rgb(255, 69, 58);
pub const COLOR_TEXT_PRIMARY: u32 = rgb(255, 255, 255);
pub const COLOR_TEXT_SECONDARY: u32 = rgb(142, 142, 147);
pub const COLOR_TEXT_MUTED: u32 = rgb(80, 80, 88);
pub const COLOR_BTN_BG: u32 = rgb(26, 26, 30);
pub const COLOR_BTN_HOVER: u32 = rgb(38, 38, 44);
pub const COLOR_BTN_BORDER: u32 = rgb(38, 38, 42);
pub const COLOR_BTN_BORDER_HI: u32 = rgb(65, 65, 75);
