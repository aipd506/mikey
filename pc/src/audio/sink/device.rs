use cpal::traits::{DeviceTrait, HostTrait};
use cpal::Device;
use std::io::{self, Error, ErrorKind};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::thread;

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

/// Whether the virtual mic was found: 0 not checked yet, 1 found, 2 missing. Listing audio
/// devices can take a second or more on some PCs, so the flyout reads this, never the devices.
static VIRTUAL_DEVICE: AtomicU8 = AtomicU8::new(0);
static CHECKING: AtomicBool = AtomicBool::new(false);

/// True unless a check found no virtual mic, so the setup banner never flashes before one.
pub fn virtual_device_ready() -> bool {
    VIRTUAL_DEVICE.load(Ordering::Relaxed) != 2
}

/// Looks for the virtual mic again and remembers the answer. Slow: keep it off UI threads.
pub fn update_virtual_device_status() -> bool {
    let (ready, _) = check_virtual_device_status();
    VIRTUAL_DEVICE.store(if ready { 1 } else { 2 }, Ordering::Relaxed);
    ready
}

/// Runs `update_virtual_device_status` on a new thread, unless one is already running.
pub fn refresh_virtual_device_status() {
    if !CHECKING.swap(true, Ordering::AcqRel) {
        thread::spawn(|| {
            update_virtual_device_status();
            CHECKING.store(false, Ordering::Release);
        });
    }
}

pub fn find_output_device() -> io::Result<(Device, String, bool)> {
    let host = cpal::default_host();
    let mut devices: Vec<_> = host
        .output_devices()
        .map_err(|e| Error::other(format!("failed to query output devices: {}", e)))?
        .filter_map(|dev| dev.name().ok().map(|name| (dev, name)))
        .filter(|(_, name)| !name.contains("AudioRelay"))
        .collect();

    for pattern in VIRTUAL_DEVICE_PATTERNS {
        if let Some(i) = devices.iter().position(|(_, name)| name.contains(pattern)) {
            let (dev, name) = devices.swap_remove(i);
            return Ok((dev, name, true));
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
