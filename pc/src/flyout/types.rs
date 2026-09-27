//! Win32 types, theme constants, and button definitions for the Mikey Flyout.

#![cfg(windows)]

pub use super::win32;

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

// Design system colors (vibe/rules/design-language.md)
pub const COLOR_BG_SURFACE: u32 = rgb(17, 17, 17); // #111111
pub const COLOR_BG_BLACK: u32 = rgb(0, 0, 0); // #000000
pub const COLOR_BORDER_DIVIDER: u32 = rgb(44, 44, 46); // #2C2C2E
pub const COLOR_ICON_OFF: u32 = rgb(58, 58, 60); // #3A3A3C
pub const COLOR_MIC_ON: u32 = rgb(48, 209, 88); // #30D158
pub const COLOR_CAM_ON: u32 = rgb(10, 132, 255); // #0A84FF
pub const COLOR_STATUS_OK: u32 = rgb(48, 209, 88); // #30D158
pub const COLOR_STATUS_WAIT: u32 = rgb(255, 214, 10); // #FFD60A
pub const COLOR_STATUS_ERR: u32 = rgb(255, 69, 58); // #FF453A
pub const COLOR_TEXT_PRIMARY: u32 = rgb(255, 255, 255); // #FFFFFF
pub const COLOR_TEXT_SECONDARY: u32 = rgb(142, 142, 147); // #8E8E93
pub const COLOR_AMBER_BG: u32 = rgb(35, 28, 10);
pub const COLOR_ACCENT_HOVER: u32 = rgb(35, 35, 38);

pub const FLYOUT_WIDTH: i32 = 330;
pub const FLYOUT_HEIGHT_COLLAPSED: i32 = 240;
pub const FLYOUT_HEIGHT_EXPANDED: i32 = 360;
pub const FLYOUT_CORNER_RADIUS: i32 = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlyoutButton {
    Disconnect,
    AllowJoin(u64),
    DenyJoin(u64),
    MicToggle,
    MuteToggle,
    SetupVirtualMic,
    NsSlider,
    ToggleAec,
    ToggleGate,
    PopOutCamera,
    FlipCamera,
    ToggleAdvanced,
    ToggleAskBeforeJoin,
    ToggleStartWithComputer,
    ToggleOpenPhone,
    ToggleTrustWifi,
    OpenLogs,
    Quit,
}
