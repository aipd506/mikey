use std::collections::HashSet;
use std::io::{self, BufReader, Error, ErrorKind, Read};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[inline]
fn new_adb_command() -> Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
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

/// Reads one update from `adb track-devices`: four hex digits giving a length, then that many
/// bytes of `serial<TAB>state` lines. Each update lists every device, so a phone missing from
/// it is gone. An empty list is just `0000`, with no newline after it.
fn read_device_list<R: Read>(reader: &mut R) -> io::Result<Vec<(String, String)>> {
    let mut hex = [0u8; 4];
    reader.read_exact(&mut hex)?;
    let len = std::str::from_utf8(&hex)
        .ok()
        .and_then(|h| usize::from_str_radix(h, 16).ok())
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, "bad adb track-devices length"))?;
    let mut list = vec![0u8; len];
    reader.read_exact(&mut list)?;
    Ok(String::from_utf8_lossy(&list)
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            Some((fields.next()?.to_string(), fields.next()?.to_string()))
        })
        .collect())
}

/// A phone that has only just been allowed can refuse the first try, so give it a few.
fn reverse_with_retries(serial: &str, port: u16) -> bool {
    for attempt in 1..=3 {
        match setup_adb_reverse(Some(serial), port) {
            Ok(()) => {
                println!("[adb] Reversed tcp:{} on device {}", port, serial);
                return true;
            }
            Err(e) if attempt == 3 => {
                eprintln!("[adb] Could not reverse tcp:{} on {}: {}", port, serial, e)
            }
            Err(_) => thread::sleep(Duration::from_millis(500)),
        }
    }
    false
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

/// Watches phones come and go with `adb track-devices` and sets up `adb reverse` for each one
/// that is ready. A reverse is lost whenever the phone reconnects or the adb server restarts,
/// so a phone is set up again every time it reappears.
pub fn start_adb_watcher(port: u16, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        match check_adb() {
            Ok(ver) => println!("[adb] Found {}", ver.lines().next().unwrap_or(&ver)),
            Err(e) => eprintln!(
                "[adb] Not available: {}. Ensure Android platform-tools are in PATH.",
                e
            ),
        }
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
            let Some(stdout) = child.stdout.take() else {
                let _ = child.kill();
                thread::sleep(Duration::from_secs(2));
                continue;
            };

            let mut reader = BufReader::new(stdout);
            let mut reversed: HashSet<String> = HashSet::new();
            while let Ok(devices) = read_device_list(&mut reader) {
                if !running.load(Ordering::Relaxed) {
                    let _ = child.kill();
                    return;
                }
                let ready: HashSet<String> = devices
                    .into_iter()
                    .filter(|(_, state)| state == "device")
                    .map(|(serial, _)| serial)
                    .collect();
                reversed.retain(|serial| ready.contains(serial));
                for serial in ready {
                    if !reversed.contains(&serial) && reverse_with_retries(&serial, port) {
                        reversed.insert(serial);
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

    /// CI machines have no adb, so a missing one must come back as NotFound, not as a failure.
    #[test]
    fn test_check_adb() {
        match check_adb() {
            Ok(ver) => assert!(ver.contains("Android Debug Bridge"), "{}", ver),
            Err(e) => assert_eq!(e.kind(), ErrorKind::NotFound, "{}", e),
        }
    }

    #[test]
    fn test_read_device_list() {
        // As adb sends it: one phone; then a second, unauthorized one; then none.
        let stream =
            b"0015emulator-5554\tdevice\n002dZD222XK2B8\tunauthorized\nemulator-5554\tdevice\n0000";
        let mut reader = &stream[..];
        let pair = |s: &str, st: &str| (s.to_string(), st.to_string());
        assert_eq!(
            read_device_list(&mut reader).unwrap(),
            vec![pair("emulator-5554", "device")]
        );
        assert_eq!(
            read_device_list(&mut reader).unwrap(),
            vec![
                pair("ZD222XK2B8", "unauthorized"),
                pair("emulator-5554", "device")
            ]
        );
        assert!(read_device_list(&mut reader).unwrap().is_empty());
        assert!(read_device_list(&mut reader).is_err());
    }
}
