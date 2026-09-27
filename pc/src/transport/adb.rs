use std::collections::HashSet;
use std::io::{self, Error, ErrorKind};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[inline]
fn new_adb_command() -> Command {
    let mut cmd = Command::new("adb");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd
}

/// Verifies that the adb binary is accessible and returns its version string.
pub fn check_adb() -> io::Result<String> {
    let output = new_adb_command()
        .arg("version")
        .output()
        .map_err(|e| Error::new(ErrorKind::NotFound, format!("adb binary not found: {}", e)))?;

    if !output.status.success() {
        return Err(Error::other(format!(
            "adb version failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Lists serial numbers of all authorized connected devices ("device" status).
pub fn list_devices() -> io::Result<Vec<String>> {
    let output = new_adb_command()
        .arg("devices")
        .output()
        .map_err(|e| Error::new(ErrorKind::NotFound, format!("adb failed: {}", e)))?;

    if !output.status.success() {
        return Err(Error::other(format!(
            "adb devices failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in text.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            devices.push(parts[0].to_string());
        }
    }

    Ok(devices)
}

/// Sets up `adb reverse tcp:{port} tcp:{port}` for all connected authorized devices,
/// or for a specific device serial if specified.
pub fn setup_adb_reverse(serial: Option<&str>, port: u16) -> io::Result<()> {
    let mut cmd = new_adb_command();
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd.arg("reverse")
        .arg(format!("tcp:{}", port))
        .arg(format!("tcp:{}", port));

    let output = cmd
        .output()
        .map_err(|e| Error::other(format!("adb reverse failed to execute: {}", e)))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!(
            "adb reverse failed: {}",
            err_msg.trim()
        )));
    }

    Ok(())
}

/// Runs a background loop that monitors for connected Android devices and automatically
/// ensures ADB reverse port forwarding is configured when plugged in.
pub fn start_adb_watcher(port: u16, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut configured_devices: HashSet<String> = HashSet::new();

        while running.load(Ordering::Relaxed) {
            match list_devices() {
                Ok(current_devices) => {
                    let current_set: HashSet<String> = current_devices.into_iter().collect();

                    // Detect newly attached devices
                    for serial in &current_set {
                        if !configured_devices.contains(serial) {
                            match setup_adb_reverse(Some(serial), port) {
                                Ok(()) => {
                                    println!("[adb] Reversed tcp:{} on device {}", port, serial);
                                    configured_devices.insert(serial.clone());
                                }
                                Err(e) => {
                                    eprintln!("[adb] Failed to reverse port on {}: {}", serial, e);
                                }
                            }
                        }
                    }

                    // Remove unplugged devices
                    configured_devices.retain(|s| current_set.contains(s));
                }
                Err(_) => {
                    // ADB daemon may be restarting or unavailable; retry after backoff
                }
            }

            thread::sleep(Duration::from_secs(2));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_adb() {
        let ver = check_adb();
        assert!(ver.is_ok());
    }
}
