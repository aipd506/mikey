use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use std::io::{self, Error, ErrorKind};

pub const SAMPLE_RATE: u32 = 48_000;

const VIRTUAL_DEVICE_PATTERNS: &[&str] = &[
    "Mikey Mic Bridge",
    "Mikey Audio Bridge",
    "Mikey",
    "CABLE In 16 Ch",
    "CABLE Input",
    "CABLE In",
    "VB-Audio",
    "CABLE",
    "Virtual Speaker",
];

pub fn is_virtual_device(name: &str) -> bool {
    if name.contains("AudioRelay") {
        return false;
    }
    VIRTUAL_DEVICE_PATTERNS.iter().any(|p| name.contains(p))
}

pub fn find_output_device() -> io::Result<(Device, String, bool)> {
    let host = cpal::default_host();
    let devices: Vec<_> = host
        .output_devices()
        .map_err(|e| Error::other(format!("failed to query output devices: {}", e)))?
        .filter_map(|dev| dev.name().ok().map(|name| (dev, name)))
        .filter(|(_, name)| !name.contains("AudioRelay"))
        .collect();

    for pattern in VIRTUAL_DEVICE_PATTERNS {
        if let Some((_dev, name)) = devices.iter().find(|(_, name)| name.contains(pattern)) {
            let host = cpal::default_host();
            let target_name = name.clone();
            if let Ok(devs) = host.output_devices() {
                for d in devs {
                    if let Ok(n) = d.name() {
                        if n == target_name {
                            return Ok((d, target_name, true));
                        }
                    }
                }
            }
        }
    }

    if let Some(dev) = host.default_output_device() {
        let name = dev.name().unwrap_or_else(|_| "Default Output".to_string());
        Ok((dev, name, false))
    } else {
        Err(Error::new(
            ErrorKind::NotFound,
            "no audio output device available",
        ))
    }
}

pub fn check_virtual_device_status() -> (bool, &'static str) {
    let host = cpal::default_host();
    let has_output = if let Ok(devices) = host.output_devices() {
        devices
            .filter_map(|d| d.name().ok())
            .any(|name| is_virtual_device(&name))
    } else {
        false
    };

    let has_input = if let Ok(devices) = host.input_devices() {
        devices.filter_map(|d| d.name().ok()).any(|name| {
            !name.contains("AudioRelay")
                && (name.contains("Mikey")
                    || name.contains("CABLE Output")
                    || name.contains("VB-Audio")
                    || name.contains("CABLE"))
        })
    } else {
        false
    };

    if has_output && has_input {
        (true, "Microphone: Ready ✓")
    } else {
        (false, "Microphone: Not found")
    }
}

pub fn is_mikey_branded() -> bool {
    let host = cpal::default_host();
    if let Ok(devices) = host.input_devices() {
        devices
            .filter_map(|d| d.name().ok())
            .any(|name| name.contains("Mikey Mic") || name.contains("Mikey"))
    } else {
        false
    }
}
