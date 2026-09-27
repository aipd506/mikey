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
pub use super::palette::*;

pub const FLYOUT_WIDTH: i32 = 300;
pub const FLYOUT_HEIGHT_COLLAPSED: i32 = 215;
pub const FLYOUT_HEIGHT_EXPANDED: i32 = 330;
pub const FLYOUT_CORNER_RADIUS: i32 = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlyoutButton {
    Disconnect,
    AllowJoin(u64),
    DenyJoin(u64),
    MicToggle,
    MuteToggle,
    SetupVirtualMic,
    TogglePreview, // Persistent camera preview option (open/toggle preview window)
    FlipCamera,    // Flip front/back lens remotely
    NsSlider,
    ToggleAec,
    ToggleGate,
    PopOutCamera,
    ToggleAdvanced,
    ToggleAskBeforeJoin,
    ToggleStartWithComputer,
    ToggleOpenPhone,
    ToggleTrustWifi,
    OpenLogs,
    Quit,
}
