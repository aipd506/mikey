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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ControlAudioPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns_strength: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aec: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate_db: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ControlVideoPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lens: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ControlPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<ControlAudioPayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<ControlVideoPayload>,
}
