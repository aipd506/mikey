//! Mikey Flyout companion window module.
//!
//! Provides a compact, borderless Win32 companion window anchored to the system tray,
//! matching the reference visual design system with double-buffered GDI rendering.

#![cfg(windows)]

pub mod blit;
pub mod gdi;
pub mod input;
pub mod layout;
pub mod palette;
pub mod render;
pub mod render_audio;
pub mod render_banners;
pub mod render_dsp;
pub mod render_footer;
pub mod render_header;
pub mod render_video;
pub mod types;
pub mod win32;
pub mod window;
pub mod wndproc;

pub use types::*;
pub use window::FlyoutWindow;
