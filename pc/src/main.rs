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
    // First thing, so Windows drops the busy pointer it shows while Mikey starts.
    #[cfg(windows)]
    mikey::launch::end_busy_pointer();

    let args: Vec<String> = env::args().collect();
    let test_mode = args.iter().any(|a| a == "--test-tone");
    #[cfg(windows)]
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

    // A second launch opens the running Mikey instead of starting another.
    #[cfg(windows)]
    if !test_mode && mikey::instance::already_running() {
        mikey::instance::open_running();
        return;
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

    // --test-tone mode: play a 3-second tone to verify audio pipeline, then exit
    if test_mode {
        let _streams = sink::start_output(&jitter_buffer);
        test_tone::play_test_tone(&jitter_buffer, 3);
        thread::sleep(Duration::from_millis(500)); // drain buffer
        return;
    }

    // 2. The tray comes up first, so Mikey shows at once. Started by hand, not at login, it
    // opens its flyout too.
    #[cfg(windows)]
    let _tray_handle = mikey::tray::start_tray_thread(
        session_manager.clone(),
        Arc::clone(&video_pipeline),
        Arc::clone(&jitter_buffer),
        Arc::clone(&running),
        !args.iter().any(|a| a == "--autostart"),
    );

    // 3. Everything slow runs on one thread, in order: listing audio devices can take seconds on
    // some PCs, and the first softcam registration remaps HKCR for the whole process, which
    // audio setup must not see. The audio streams live as long as this thread.
    {
        let jitter_buffer = Arc::clone(&jitter_buffer);
        let video_pipeline = Arc::clone(&video_pipeline);
        let running = Arc::clone(&running);
        thread::spawn(move || {
            let _streams = sink::start_output(&jitter_buffer);
            #[cfg(windows)]
            if !sink::update_virtual_device_status() {
                println!(
                    "[mikey] Note: Virtual microphone not yet configured as 'Mikey Mic'. Setup available via flyout companion."
                );
            }
            video_pipeline.load_vcam();
            while running.load(Ordering::Relaxed) {
                thread::park_timeout(Duration::from_secs(1));
            }
        });
    }

    // Level 1: USB Debugging. The watcher checks adb itself, off the startup path.
    if cfg.levels.usb_debugging {
        let _adb_handle = adb::start_adb_watcher(PORT_TCP, Arc::clone(&running));
    }

    // 4. Start UDP Discovery Beacon (Level 2 & Level 3 discovery)
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

    // 5. Start Bluetooth RFCOMM listener (Level 4: Bluetooth)
    if cfg.levels.bluetooth {
        let _bt_handle = bt::start_bt_listener(
            session_manager.clone(),
            Arc::clone(&jitter_buffer),
            Arc::clone(&running),
        );
    }

    // 6. Bind and run TCP listener on 0.0.0.0:PORT_TCP (Level 1, Level 2, Level 3). A busy port
    // (often an older Mikey still running) is retried rather than quitting with no word.
    let listener = loop {
        match tcp::bind_listener() {
            Ok(l) => break l,
            Err(e) => eprintln!(
                "[tcp] Failed to bind TCP listener on port {}: {}. Retrying.",
                PORT_TCP, e
            ),
        }
        if !running.load(Ordering::Relaxed) {
            return;
        }
        thread::sleep(Duration::from_secs(2));
    };

    let _tcp_handle = tcp::start_tcp_listener(
        listener,
        Arc::clone(&jitter_buffer),
        Arc::clone(&video_pipeline),
        session_manager.clone(),
        Arc::clone(&running),
    );

    println!("[tcp] Listening on 0.0.0.0:{}", PORT_TCP);

    println!("[ready] Waiting for Mikey Android client to connect...");
    println!("[hint] Run with --test-tone to verify audio without a phone.");

    // Keep main thread alive
    while running.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_secs(1));
    }
}
