mod random;
#[cfg(test)]
mod tests;
mod types;

pub use random::generate_random_hex;
pub use types::{ConnectionLevelsConfig, TrustedDevice, WindowPos};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub pc_id: String,
    pub pc_name: String,
    #[serde(default)]
    pub ask_before_joining: bool,
    #[serde(default = "types::default_true")]
    pub start_with_computer: bool,
    #[serde(default = "types::default_true")]
    pub open_on_phone_when_plugged_in: bool,
    #[serde(default)]
    pub trust_wifi_automatically: bool,
    #[serde(default)]
    pub levels: ConnectionLevelsConfig,
    #[serde(default)]
    pub trusted_devices: HashMap<String, TrustedDevice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_pos: Option<WindowPos>,
}

impl Default for Config {
    fn default() -> Self {
        let pc_id = generate_random_hex(16);
        let pc_name = env::var("COMPUTERNAME")
            .or_else(|_| env::var("HOSTNAME"))
            .unwrap_or_else(|_| "Mikey-Host".to_string());

        Self {
            pc_id,
            pc_name,
            ask_before_joining: false,
            start_with_computer: true,
            open_on_phone_when_plugged_in: true,
            trust_wifi_automatically: false,
            levels: ConnectionLevelsConfig::default(),
            trusted_devices: HashMap::new(),
            window_pos: None,
        }
    }
}

impl Config {
    pub fn default_config_path() -> PathBuf {
        #[cfg(windows)]
        {
            if let Ok(appdata) = env::var("APPDATA") {
                return PathBuf::from(appdata).join("Mikey").join("config.toml");
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(home) = env::var("HOME") {
                return PathBuf::from(home)
                    .join(".config")
                    .join("mikey")
                    .join("config.toml");
            }
        }

        PathBuf::from("config.toml")
    }

    pub fn default_log_dir() -> PathBuf {
        Self::default_config_path()
            .parent()
            .map(|p| p.join("logs"))
            .unwrap_or_else(|| PathBuf::from("logs"))
    }

    pub fn load_or_default(path: &Path) -> Self {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match toml::from_str::<Config>(&content) {
                    Ok(cfg) => return cfg,
                    Err(e) => {
                        eprintln!("[config] Failed to parse config at {:?}: {}", path, e);
                    }
                },
                Err(e) => {
                    eprintln!("[config] Failed to read config at {:?}: {}", path, e);
                }
            }
        }

        let cfg = Config::default();
        if let Err(e) = cfg.save_to(path) {
            eprintln!(
                "[config] Failed to write initial config to {:?}: {}",
                path, e
            );
        }
        cfg
    }

    pub fn save_to(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let toml_str = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let tmp_path = path.with_extension("toml.tmp");
        {
            let mut file = File::create(&tmp_path)?;
            file.write_all(toml_str.as_bytes())?;
            file.flush()?;
        }
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    pub fn is_trusted(&self, device_id: &str, token: &str) -> bool {
        self.trusted_devices
            .get(device_id)
            .map(|d| d.token == token)
            .unwrap_or(false)
    }

    pub fn add_or_update_device(
        &mut self,
        device_id: String,
        device_name: String,
        token: String,
        transport: Option<u8>,
    ) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .ok();

        self.trusted_devices.insert(
            device_id.clone(),
            TrustedDevice {
                device_id,
                device_name,
                token,
                last_transport: transport,
                last_seen: now,
            },
        );
    }

    pub fn forget_device(&mut self, device_id: &str) -> bool {
        self.trusted_devices.remove(device_id).is_some()
    }
}
