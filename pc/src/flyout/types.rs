//! Win32 types, theme constants, and button definitions for the Mikey Flyout.

#![cfg(windows)]

pub use super::palette::*;
pub use super::win32;

#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

pub fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub const FLYOUT_WIDTH: i32 = 300;
pub const FLYOUT_HEIGHT_COLLAPSED: i32 = 254;
pub const FLYOUT_HEIGHT_EXPANDED: i32 = 360;
pub const FLYOUT_CORNER_RADIUS: i32 = 16;
pub const FLYOUT_CLASS: &str = "MikeyFlyoutCompanionClass";
/// Posted to the flyout by a second launch of Mikey (WM_APP + 1): open by the tray.
pub const WM_APP_SHOW: u32 = 0x8001;

#[derive(Clone, Copy)]
pub struct FlyoutFonts {
    pub title: win32::HFONT,
    pub heading: win32::HFONT,
    pub body: win32::HFONT,
    pub body_bold: win32::HFONT,
    pub icon: win32::HFONT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlyoutButton {
    Disconnect,
    AllowJoin(u64),
    DenyJoin(u64),
    MicToggle,
    MuteToggle,
    SetupVirtualMic,
    TogglePreview,
    FlipCamera,
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
