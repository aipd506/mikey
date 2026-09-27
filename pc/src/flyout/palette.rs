//! Monochrome palette constants for the Mikey Flyout.
//! Fully black & white: luminance-based contrast with zero color tint.

#![cfg(windows)]

use super::types::rgb;

pub const MONO_WHITE: u32 = rgb(255, 255, 255);
pub const MONO_LIGHT: u32 = rgb(190, 190, 194);
pub const MONO_MID: u32 = rgb(120, 120, 124);
pub const MONO_DIM: u32 = rgb(72, 72, 76);
pub const MONO_DARK: u32 = rgb(38, 38, 42);
pub const MONO_DARKER: u32 = rgb(28, 28, 32);
pub const MONO_SURFACE: u32 = rgb(17, 17, 17);
pub const MONO_BLACK: u32 = rgb(0, 0, 0);
pub const MONO_BORDER: u32 = rgb(44, 44, 46);
pub const MONO_BTN_BG: u32 = rgb(26, 26, 28);
pub const MONO_BTN_HOVER: u32 = rgb(40, 40, 44);
pub const MONO_BTN_BORDER: u32 = rgb(48, 48, 52);
pub const MONO_BTN_BORDER_HI: u32 = rgb(80, 80, 86);
