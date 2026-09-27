# 14. Tech Stack

## 14.1 Android

| Layer | Choice | Why |
|---|---|---|
| Language | Kotlin | Native, no bridge |
| UI | Jetpack Compose (Material 3 **bottom sheet only**, otherwise plain foundation) | Declarative, small |
| State | `StateFlow` from service → UI | UI observes, never owns logic |
| Audio | AAudio (NDK, low-latency mode), fallback `AudioRecord` | Raw capture, lowest latency, no extra library |
| Codec | libopus via small JNI wrapper (built from source, BSD) | Near-transparent audio on Wi‑Fi/Bluetooth at tiny CPU cost |
| Camera | CameraX (`camera-core`, `camera-camera2`, `camera-lifecycle`) | Handles device quirks; ~1 MB |
| JPEG | `YuvImage.compressToJpeg` (platform) first; libjpeg-turbo JNI only if profiling demands | No dependency unless needed |
| Background | Foreground service, types `connectedDevice` \| `microphone` \| `camera` declared per state | Required on Android 14+ |
| Networking | `java.net.Socket` / `DatagramSocket`, `Network.bindSocket` for interface pinning | No HTTP, no libraries |
| Bluetooth | `BluetoothSocket` (RFCOMM) | Platform API |
| Settings | `SharedPreferences` | No database, no DataStore needed |
| Build | Min SDK 26, target = latest stable; R8 + resource shrinking; ABI splits (arm64-v8a, armeabi-v7a) | Small APK |
| Tests | JUnit 4, unit tests only (never in the APK) | Proves the protocol byte layout matches the spec |

**Permissions (all requested lazily, at the moment they are needed):**
`RECORD_AUDIO`, `CAMERA`, `POST_NOTIFICATIONS` (13+), `BLUETOOTH_CONNECT` (12+) / `BLUETOOTH` (≤ 11), `INTERNET`, `ACCESS_NETWORK_STATE`, `ACCESS_WIFI_STATE`, `CHANGE_NETWORK_STATE`, `WAKE_LOCK`, `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_MICROPHONE`, `FOREGROUND_SERVICE_CAMERA`, `FOREGROUND_SERVICE_CONNECTED_DEVICE`.
No location, no storage, no contacts, no phone state.

**Explicitly excluded:** Retrofit, OkHttp, Hilt/Dagger/Koin, Room, RxJava, Firebase, any analytics or crash-reporting SDK, any ads SDK.

## 14.2 PC (`mikey`, Rust)

| Concern | Crate / API | Notes |
|---|---|---|
| Tray icon | `tray-icon` (Windows); `ksni` or `tray-icon` (Linux) | System tray notification area icon + click events |
| Flyout UI (Mini-app) | `egui` + `eframe` (or borderless `winit` + `softbuffer`) | Native Rust GUI, zero webview/electron bloat, instant launch, < 20 MB RAM, dark theme |
| Notifications | `tauri-winrt-notification` (Windows, action buttons); `notify-rust` (Linux, actions) | Fallback: interactive alert banner in tray flyout |
| Preview window | Embedded in flyout or detached via `egui` / `minifb` | Live software blit of decoded JPEG frames |
| Audio I/O (Windows) | `cpal` (WASAPI output + loopback capture) | |
| Audio I/O (Linux) | `libpulse-simple-binding` (works on PipeWire via pipewire-pulse) + `pactl` for module setup | |
| Opus | `opus-decoder` (pure Rust, RFC 8251) | Zero C/CMake build dependency; pure Rust safe decoder |
| Resampling | `rubato` | Drift correction |
| Noise suppression | `nnnoiseless` (pure Rust RNNoise) | |
| AEC | SpeexDSP via small FFI (`aec.rs`, Cargo feature `aec`) | |
| JPEG decode | `zune-jpeg` | Pure Rust, fast |
| Virtual cam (Windows) | softcam DLL via FFI | |
| Virtual cam (Linux) | v4l2 ioctls via `libc` | |
| Bluetooth (Windows) | `windows` crate: Winsock `AF_BTH`, `WSASetService` | |
| Bluetooth (Linux) | `bluer` (needs Tokio **current-thread** runtime, confined to `bt.rs`'s thread) | Only async code in the project |
| ADB | external `adb` binary (PATH or bundled) driven via `std::process` | |
| Config | `serde` + `toml` | |
| JSON (control frames) | `serde_json` | |
| Logging | `log` + a ~50-line rotating file logger (no heavy logging framework) | |
| Autostart | Windows: HKCU `…\CurrentVersion\Run` via `windows` crate; Linux: `~/.config/autostart/mikey.desktop` | No admin needed |

**Explicitly excluded:** Electron, Tauri webviews, Python, Node, any HTTP server, Tokio anywhere outside `bt.rs` on Linux, GTK/Qt.
