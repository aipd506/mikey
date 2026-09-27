#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use mikey::audio::pipeline::JitterBuffer;
use mikey::audio::{sink, test_tone};
use mikey::config::Config;
use mikey::protocol::PORT_TCP;
use mikey::session::SessionManager;
use mikey::transport::{adb, beacon, bt, tcp};
use mikey::video::VideoPipeline;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let test_mode = args.iter().any(|a| a == "--test-tone");
    let keep_console = args.iter().any(|a| a == "--console" || a == "--test-tone");

    // Hide and detach any console window immediately so Mikey runs silently in the tray
    #[cfg(windows)]
    if !keep_console {
        unsafe {
            #[link(name = "kernel32")]
            extern "system" {
                fn FreeConsole() -> i32;
                fn GetConsoleWindow() -> usize;
            }
            #[link(name = "user32")]
            extern "system" {
                fn ShowWindow(hWnd: usize, nCmdShow: i32) -> i32;
            }

            let hwnd = GetConsoleWindow();
            if hwnd != 0 {
                ShowWindow(hwnd, 0); // SW_HIDE
                FreeConsole();
            }
        }
    }

    println!("=== Mikey PC (Multi-Transport Engine) ===");

    let running = Arc::new(AtomicBool::new(true));
    let jitter_buffer = Arc::new(JitterBuffer::new());
    let video_pipeline = Arc::new(VideoPipeline::new());
    let _video_handle = video_pipeline.start_pipeline_thread(Arc::clone(&running));

    // 1. Initialize configuration and session manager
    let config_path = Config::default_config_path();
    let session_manager = SessionManager::new(config_path);
    let cfg = session_manager.config();
    println!("[pc] ID: {}, Name: {}", cfg.pc_id, cfg.pc_name);

    // 2. Check ADB status (Level 1: USB Debugging)
    if cfg.levels.usb_debugging {
        match adb::check_adb() {
            Ok(ver) => {
                let first_line = ver.lines().next().unwrap_or(&ver);
                println!("[adb] Found {}", first_line);
            }
            Err(e) => {
                eprintln!(
                    "[adb] Not available: {}. Ensure Android platform-tools are in PATH.",
                    e
                );
            }
        }
        let _adb_handle = adb::start_adb_watcher(PORT_TCP, Arc::clone(&running));
    }

    // 3. Check virtual microphone device status (setup is accessible anytime via flyout)
    #[cfg(windows)]
    {
        let (virt_ready, _) = sink::check_virtual_device_status();
        let is_branded = sink::is_mikey_branded();
        if !virt_ready || !is_branded {
            println!(
                "[mikey] Note: Virtual microphone not yet configured as 'Mikey Mic'. Setup available via flyout companion."
            );
        }
    }

    // 4. Initialize audio output device (Mikey Audio Bridge / VB-Cable / default output)
    let _audio_stream = match sink::find_output_device() {
        Ok((device, name, is_vb_cable)) => {
            if is_vb_cable {
                println!("[audio] Using virtual mic device: {}", name);
            } else {
                println!(
                    "[audio] No virtual mic found. Using fallback output: {}",
                    name
                );
            }

            match sink::start_audio_stream(&device, Arc::clone(&jitter_buffer)) {
                Ok(stream) => {
                    println!("[audio] Output stream initialized (48 kHz)");
                    Some(stream)
                }
                Err(e) => {
                    eprintln!("[audio] Failed to start audio playback stream: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            eprintln!("[audio] Error finding output device: {}", e);
            None
        }
    };

    // 4b. Initialize AEC loopback reference stream (speaker sound cancellation)
    let _loopback_stream = match sink::start_loopback_stream(Arc::clone(&jitter_buffer)) {
        Ok(stream) => {
            println!(
                "[audio] AEC loopback reference stream active (speaker sound cancellation enabled)"
            );
            Some(stream)
        }
        Err(e) => {
            eprintln!("[audio] Note: Loopback stream not available: {}", e);
            None
        }
    };

    // --test-tone mode: play a 3-second tone to verify audio pipeline, then exit
    if test_mode {
        test_tone::play_test_tone(&jitter_buffer, 3);
        thread::sleep(Duration::from_millis(500)); // drain buffer
        return;
    }

    // 4. Start UDP Discovery Beacon (Level 2 & Level 4 discovery)
    if cfg.levels.wifi || cfg.levels.usb_tethering {
        match beacon::start_beacon_responder(
            cfg.pc_id.clone(),
            cfg.pc_name.clone(),
            Arc::clone(&running),
        ) {
            Ok(_handle) => println!("[beacon] UDP discovery responder active on port 7654"),
            Err(e) => eprintln!("[beacon] Failed to bind discovery responder: {}", e),
        }
    }

    // 5. Start Bluetooth RFCOMM listener (Level 3: Bluetooth)
    if cfg.levels.bluetooth {
        let _bt_handle = bt::start_bt_listener(
            session_manager.clone(),
            Arc::clone(&jitter_buffer),
            Arc::clone(&running),
        );
    }

    // 6. Bind and run TCP listener on 0.0.0.0:PORT_TCP (Level 1, Level 2, Level 4)
    let listener = match tcp::bind_listener() {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "[tcp] Failed to bind TCP listener on port {}: {}",
                PORT_TCP, e
            );
            return;
        }
    };

    let _tcp_handle = tcp::start_tcp_listener(
        listener,
        Arc::clone(&jitter_buffer),
        Arc::clone(&video_pipeline),
        session_manager.clone(),
        Arc::clone(&running),
    );

    println!("[tcp] Listening on 0.0.0.0:{}", PORT_TCP);

    // 7. Start System Tray icon, menu & flyout companion
    #[cfg(windows)]
    let _tray_handle = mikey::tray::start_tray_thread(
        session_manager.clone(),
        Arc::clone(&video_pipeline),
        Arc::clone(&jitter_buffer),
        Arc::clone(&running),
    );

    println!("[ready] Waiting for Mikey Android client to connect...");
    println!("[hint] Run with --test-tone to verify audio without a phone.");

    // Keep main thread alive
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }
}
