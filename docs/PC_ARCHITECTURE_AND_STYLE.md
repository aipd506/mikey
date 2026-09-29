# PC Architecture & Rust Engineering Style

The PC companion daemon (`pc/`) is a high-performance systems executable written in **Rust**. It runs silently in the system tray, operates multi-transport network listeners, decodes real-time media streams, executes DSP, and feeds Windows virtual audio and video drivers.

---

## 1. Concurrency Architecture & Threading Model

### 1.1 Pure `std` Threads + Bounded Channels
The PC architecture rejects complex asynchronous runtimes in favor of **deterministic OS threads and bounded channels**:
- Dedicated threads run blocking event loops with explicit socket timeouts.
- Inter-thread communication relies strictly on standard library synchronization primitives (`std::sync::{Arc, Mutex, Condvar}`) and lock-free atomic flags.

### 1.2 The Strict Async Prohibition
> **Core Invariant**: Asynchronous runtimes (Tokio, async-std) are **strictly forbidden** throughout the PC codebase.

There is exactly **one isolated exception**:
- `pc/src/transport/bt/server.rs` on **Linux**: The underlying Linux Bluetooth library (`bluer`) depends directly on Tokio for D-Bus integration. A minimal Tokio runtime is spawned solely inside this file.
- Everywhere else (all networking, audio pipelines, video pipelines, Win32 message pumps, and file I/O on Windows and Linux), standard blocking I/O with timeouts is mandated.

---

## 2. Real-Time Safety & Memory Discipline

### 2.1 Zero Allocations in Media Hot Paths
The audio processing loop operates at 10 ms intervals (100 times per second), and the video decoder processes up to 30–60 frames per second:
- Memory buffers for audio processing (`VecDeque<i16>`, scratch arrays for normalizer and RNNoise) and video decoding (`spare: Vec<u8>`) are pre-allocated during pipeline startup.
- No dynamic heap allocations occur during active streaming loops.

### 2.2 Audio Callback Non-Blocking Invariant
The WASAPI audio playback thread is driven by the Windows kernel audio engine at high real-time priority:
- The callback **must never block** on network I/O, file access, or mutexes held by slow rendering threads.
- Audio handoff from `JitterBuffer` uses non-blocking try-locks. If contention occurs or the buffer runs dry, digital silence (zeroes) is emitted rather than stalling the OS audio engine.

### 2.3 Strict Upper Bounds & Socket Timeouts
- **Socket Timeouts**: All network sockets configure `set_read_timeout(Some(Duration::from_secs(15)))` and `set_write_timeout(Some(Duration::from_secs(5)))`. Dead sockets are terminated promptly.
- **Queue Limits**: Network queues and jitter buffers enforce hard sample ceilings (e.g. `MAX_LATENCY_MS = 200 ms`). If packet transit halts, stale samples are dropped to maintain real-time responsiveness.

---

## 3. Silent Tray Lifecycle & Windows Integration

```text
[User launches mikey.exe]
           │
           ├──► launch::end_busy_pointer() (Instantly clears Windows wait cursor)
           ├──► instance::already_running()
           │         │
           │         ├── Yes: Signals existing instance to open flyout, then exits (0)
           │         └── No: Continues boot
           ├──► FreeConsole() & ShowWindow(SW_HIDE) (Runs invisibly in background)
           ├──► tray::start_tray_thread() (Instantly shows tray icon in taskbar)
           ├──► Background Audio & Virtual Cam Initialization (Slow init off startup)
           └──► Listeners Spawn (TCP :7653, UDP :7654, ADB Watcher, BT Server)
```

### 3.1 Instant Startup
1. **Busy Pointer Dismissal (`launch.rs`)**: Windows shows an hourglass/spinning cursor when launching an executable. Mikey immediately calls `end_busy_pointer()` to ensure seamless background entry.
2. **Console Detachment**: In release builds (`windows_subsystem = "windows"`), and in debug builds without `--console`, Mikey calls `FreeConsole()` and hides any lingering console handle.
3. **Single-Instance Mutex (`instance.rs`)**: Uses a named Win32 Mutex (`Global\MikeySingleInstanceMutex`). If another instance is running, it brings the existing instance's flyout to the front and terminates cleanly.

---

## 4. Rust Coding Conventions & Anti-AI-Slop Rules

1. **Zero Unused Abstractions**:
   Do not introduce generic traits, visitor patterns, or factory wrappers for single structs. Write direct, explicit implementations.
2. **Clippy & Compiler Warnings**:
   Code must pass `cargo clippy -- -D warnings` and `cargo fmt --check` with zero warnings.
3. **Explicit Error Handling**:
   Use `std::io::Result` or dedicated enums for error handling. Avoid `unwrap()` in production execution paths where recoverable errors can occur.
4. **Conditional Compilation Hygiene**:
   Platform-specific Win32 code is gated strictly with `#[cfg(windows)]`, ensuring cross-platform compilation remains clean.
