//! Design language palette constants for the Mikey Flyout.
//! Aligned with vibe/rules/design-language.md: OLED black, surface #111111, divider #2C2C2E.

#![cfg(windows)]

use super::types::rgb;

// Authentic Mikey design system tokens
pub const COLOR_BG_SURFACE: u32 = rgb(17, 17, 17); // #111111
pub const COLOR_BG_BLACK: u32 = rgb(0, 0, 0); // #000000
pub const COLOR_BORDER: u32 = rgb(44, 44, 46); // #2C2C2E
pub const COLOR_ICON_OFF: u32 = rgb(58, 58, 60); // #3A3A3C
pub const COLOR_MIC_ON: u32 = rgb(48, 209, 88); // #30D158 (Mic active / OK)
pub const COLOR_CAM_ON: u32 = rgb(10, 132, 255); // #0A84FF (Camera active)
pub const COLOR_STATUS_WAIT: u32 = rgb(255, 214, 10); // #FFD60A (Amber / Pending)
pub const COLOR_ALERT_RED: u32 = rgb(255, 69, 58); // #FF453A (Mute / Error)
pub const COLOR_TEXT_PRIMARY: u32 = rgb(255, 255, 255); // #FFFFFF
pub const COLOR_TEXT_SECONDARY: u32 = rgb(142, 142, 147); // #8E8E93
pub const COLOR_TEXT_MUTED: u32 = rgb(90, 90, 95);

pub const COLOR_BTN_BG: u32 = rgb(26, 26, 28); // #1A1A1C
pub const COLOR_BTN_HOVER: u32 = rgb(38, 38, 42); // #26262A
pub const COLOR_BTN_BORDER: u32 = rgb(44, 44, 46); // #2C2C2E
pub const COLOR_BTN_BORDER_HI: u32 = rgb(70, 70, 76);

// Backward-compatible aliases
pub const MONO_WHITE: u32 = COLOR_TEXT_PRIMARY;
pub const MONO_LIGHT: u32 = rgb(190, 190, 194);
pub const MONO_MID: u32 = COLOR_TEXT_SECONDARY;
pub const MONO_DIM: u32 = COLOR_ICON_OFF;
pub const MONO_DARK: u32 = rgb(38, 38, 42);
pub const MONO_DARKER: u32 = rgb(28, 28, 32);
pub const MONO_SURFACE: u32 = COLOR_BG_SURFACE;
pub const MONO_BLACK: u32 = COLOR_BG_BLACK;
pub const MONO_BORDER: u32 = COLOR_BORDER;
pub const MONO_BTN_BG: u32 = COLOR_BTN_BG;
pub const MONO_BTN_HOVER: u32 = COLOR_BTN_HOVER;
pub const MONO_BTN_BORDER: u32 = COLOR_BTN_BORDER;
pub const MONO_BTN_BORDER_HI: u32 = COLOR_BTN_BORDER_HI;
