use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, Error, ErrorKind};
use std::process::{Command, Stdio};
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
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("adb version failed: {}", err)));
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
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("adb devices failed: {}", err)));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let devices = text
        .lines()
        .skip(1)
        .filter_map(|l| {
            let parts: Vec<&str> = l.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "device" {
                Some(parts[0].to_string())
            } else {
                None
            }
        })
        .collect();
    Ok(devices)
}

/// Parses a single device entry from an `adb track-devices` line.
pub fn parse_track_device_line(line: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}

/// Sets up `adb reverse tcp:{port} tcp:{port}` for all connected authorized devices.
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

/// Runs a background loop that monitors for connected Android devices via `adb track-devices`.
pub fn start_adb_watcher(port: u16, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut configured_devices: HashSet<String> = HashSet::new();

        while running.load(Ordering::Relaxed) {
            let mut child = match new_adb_command()
                .arg("track-devices")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    eprintln!("[adb] Failed to spawn adb track-devices: {}", e);
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            let stdout = match child.stdout.take() {
                Some(s) => s,
                None => {
                    let _ = child.kill();
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }
            };

            let reader = BufReader::new(stdout);
            for line_res in reader.lines() {
                if !running.load(Ordering::Relaxed) {
                    let _ = child.kill();
                    return;
                }

                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => break,
                };

                if let Some((serial, state)) = parse_track_device_line(&line) {
                    if state == "device" {
                        if !configured_devices.contains(serial)
                            && setup_adb_reverse(Some(serial), port).is_ok()
                        {
                            println!("[adb] Reversed tcp:{} on device {}", port, serial);
                            configured_devices.insert(serial.to_string());
                        }
                    } else {
                        configured_devices.remove(serial);
                    }
                }
            }

            let _ = child.kill();
            let _ = child.wait();
            if running.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_secs(1));
            }
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

    #[test]
    fn test_parse_track_device_line() {
        assert_eq!(
            parse_track_device_line("emulator-5554\tdevice"),
            Some(("emulator-5554", "device"))
        );
        assert_eq!(
            parse_track_device_line("1234567890 offline"),
            Some(("1234567890", "offline"))
        );
        assert_eq!(parse_track_device_line(""), None);
        assert_eq!(parse_track_device_line("invalid"), None);
    }
}
