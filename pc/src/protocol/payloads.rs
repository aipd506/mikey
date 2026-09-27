use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloPayload {
    pub proto: u32,
    pub device_id: String,
    pub device_name: String,
    pub level: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub caps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomePayload {
    pub pc_id: String,
    pub pc_name: String,
    pub token: String,
    pub resumed: bool,
    #[serde(default)]
    pub pc_caps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RejectPayload {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByePayload {
    pub reason: String,
}
