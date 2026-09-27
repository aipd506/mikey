use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConnectionLevelsConfig {
    #[serde(default = "default_true")]
    pub usb_debugging: bool,
    #[serde(default = "default_true")]
    pub usb_tethering: bool,
    #[serde(default = "default_true")]
    pub bluetooth: bool,
    #[serde(default = "default_true")]
    pub wifi: bool,
}

impl Default for ConnectionLevelsConfig {
    fn default() -> Self {
        Self {
            usb_debugging: true,
            usb_tethering: true,
            bluetooth: true,
            wifi: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustedDevice {
    pub device_id: String,
    pub device_name: String,
    pub token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_transport: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowPos {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn default_true() -> bool {
    true
}
