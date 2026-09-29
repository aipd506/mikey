
<p align="center">
  <img src="docs/images/banner.png" alt="Mikey: your Android phone as a webcam and mic for your PC, over USB, Wi-Fi or Bluetooth" width="100%">
</p>
<p align="center">
  <a href="https://github.com/diveshpatil9104/mikey/releases"><img src="https://img.shields.io/github/v/release/diveshpatil9104/mikey?include_prereleases&style=for-the-badge&color=D71921&label=Download" alt="Download"></a>
  <a href="https://github.com/diveshpatil9104/mikey/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/diveshpatil9104/mikey/ci.yml?branch=main&style=for-the-badge&label=CI" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=for-the-badge" alt="License: MIT"></a>
  <a href="https://github.com/diveshpatil9104/mikey/discussions"><img src="https://img.shields.io/badge/Discussions-Join-blue?style=for-the-badge&logo=github" alt="Discussions"></a>
  <a href="https://github.com/diveshpatil9104/mikey/stargazers"><img src="https://img.shields.io/github/stars/diveshpatil9104/mikey?style=for-the-badge&color=FFD60A&logo=github" alt="Stars"></a>
</p>




<p align="center"><strong>Plug in. Tap once. Forget it exists.</strong></p>

The camera and microphone in your phone are far better than the ones built into most laptops. Mikey turns them into a webcam and mic that every app on your PC can use: Google Meet, Zoom, Teams, Discord and OBS. No accounts, no cloud, no setup ritual.

> [!NOTE]
> **v0.1.0 is an early preview** for Windows 10 / 11 and Android 8.0+. If something doesn't work, please [open an issue](https://github.com/diveshpatil9104/mikey/issues).

---

## Features

- **Mic and webcam in one app.** Your phone shows up on the PC as a microphone and as *Mikey Cam*, in any app.
- **Connects by itself.** Mikey picks the best link (USB, Wi-Fi or Bluetooth) and moves to a better one when it appears, without dropping your call.
- **Clear voice.** Raw 48 kHz audio from the phone, lossless over USB, with noise suppression on the PC.
- **Low delay.** A 20 ms audio buffer over USB and 40 ms over Wi-Fi, growing only when the link is unsteady.
- **Private by design.** No accounts and no cloud: everything stays on your own cable or network. Mic and camera start off, and a new phone needs your OK on the PC.
- **Light.** A small native tray app written in Rust. No Electron, no web view.

## Works on

- **PC:** Windows 10 and 11. Linux is in progress: the PC app builds there, but has no virtual mic, camera, tray or Bluetooth yet.
- **Phone:** Android 8.0 or newer.

---

## Get started

### 1. Install VB-CABLE on your PC

[VB-CABLE](https://vb-audio.com/Cable/) (free, by VB-Audio) is the virtual mic Mikey sends your voice to. Install it once and restart when it asks; Mikey finds it by itself.

### 2. Download Mikey

From the [latest release](https://github.com/diveshpatil9104/mikey/releases):

| File | Where it goes |
|---|---|
| `Mikey-v0.1.0-windows-x64.exe` | Your PC. Run it and Mikey appears in the system tray. |
| `Mikey-v0.1.0-android.apk` | Your phone. Android asks to allow installs from your browser or files app the first time. |

Mikey isn't code-signed yet, so Windows SmartScreen may warn you: click **More info**, then **Run anyway**.

### 3. Connect

- **USB:** turn on USB debugging on the phone, plug it in, and tap *Allow* on the phone. The PC needs [Android platform-tools](https://developer.android.com/tools/releases/platform-tools) (`adb`) on its `PATH`.
- **Wi-Fi:** put the phone and PC on the same network, and let Mikey through the firewall ([how](docs/INSTALL_PC.md#3-firewall-configuration)).

Open Mikey on the phone and tap the mic or the camera. The first time, click **Allow** on the PC to trust your phone.

### 4. Pick Mikey in your app

In Meet, Zoom or Teams, choose **CABLE Output (VB-Audio Virtual Cable)** as the microphone and **Mikey Cam** as the camera.

Something not working? See [Troubleshooting](docs/TROUBLESHOOTING.md).

<details>
<summary><b>Build from source instead</b></summary>

You need [Rust](https://rustup.rs), [Android Studio](https://developer.android.com/studio) (Quail 4 or newer) and VB-CABLE.

```powershell
git clone --recursive https://github.com/diveshpatil9104/mikey.git
cd mikey/pc
cargo run --release
```

Then open the `android` folder in Android Studio and press **Run** to install the phone app ([more ways](docs/INSTALL_ANDROID.md)).

Running from the source folder also enables the panel's **Setup Mic** button: click it once and approve the administrator prompt, and VB-CABLE's mic is renamed *Mikey Mic*.

</details>

---

## Connection levels

Mikey tries these in order and switches up as soon as a better one is available.

| Level | Link | Audio | Video |
|:-:|---|---|---|
| **1** | USB debugging | Lossless PCM, 48 kHz | Up to 1080p30 |
| **2** | USB tethering | Lossless PCM, 48 kHz | Up to 1080p30 |
| **3** | Wi-Fi (same network or the phone's hotspot) | Opus 96 kbps, or lossless | 720p30 by default, up to 1080p30 |
| **4** | Bluetooth | Opus 48 kbps | Audio only |

## How it works

```
 Phone (Android app)                               PC (Mikey tray app)

 Mic: raw 48 kHz, 10 ms frames ──┐           ┌── Opus decode ▸ noise suppression (RNNoise)
                                 ├── link ───┤     ▸ jitter buffer ▸ drift correction ▸ virtual mic
 Camera: CameraX ▸ JPEG ─────────┘           └── JPEG decode ▸ scale to 1920×1080 ▸ Mikey Cam

            link = USB debugging · USB tethering · Wi-Fi · Bluetooth
```

The phone is always the client and the PC is always the server: TCP port `7653` for the stream and UDP port `7654` for finding the PC on your network. The full design lives in [`docs/`](docs/README.md), starting with the [System Architecture](docs/SYSTEM_ARCHITECTURE.md) and the [Wire Protocol](docs/WIRE_PROTOCOL.md).

| Side | Built with |
|---|---|
| **Android** | Kotlin, Jetpack Compose, AAudio, CameraX, libopus |
| **PC** | Rust, `cpal`, `nnnoiseless` (RNNoise), `opus-decoder`, `zune-jpeg`, softcam |

---

## Roadmap

| Phase | Goal | Status |
|:-:|---|:-:|
| **1** | Mic over USB | Done |
| **2** | Four connection levels, trust, phone UI | Done |
| **3** | Phone camera as a virtual webcam | Done |
| **4** | Audio quality: noise suppression, jitter buffer, drift correction | Done |
| **5** | Release: v0.1.0 preview is out; next are the Windows installer (with VB-CABLE built in), Linux packages and v1.0 | In progress |

Details and the device test matrix are in the [Roadmap & Test Matrix](docs/ROADMAP_AND_TEST_MATRIX.md). Every release is listed in the [Changelog](docs/CHANGELOG.md).

---

## Documentation

| Document | What's in it |
|---|---|
| [Install on PC](docs/INSTALL_PC.md) | Building, running and the firewall |
| [Install on Android](docs/INSTALL_ANDROID.md) | Building and installing the phone app |
| [Troubleshooting](docs/TROUBLESHOOTING.md) | Fixes for connection, audio and camera problems |
| [Architecture](docs/ARCHITECTURE.md) | A short tour of how Mikey fits together |
| [All docs](docs/README.md) | Protocol, audio and video pipelines, design language, budgets |

---

## Contributing

Contributions are welcome, from typo fixes to new features. Read [CONTRIBUTING.md](.github/CONTRIBUTING.md) first, and look for issues labeled [`good first issue`](https://github.com/diveshpatil9104/mikey/labels/good%20first%20issue).

1. Fork the repo and create a branch, like `feat/your-idea`.
2. Read the [`docs/`](docs/README.md) for the part you're changing.
3. Make your change and run the checks below.
4. Open a pull request using the template, with proof that it works (a screenshot, recording or test output).

<details>
<summary><b>Build and check</b></summary>

**Android**

```bash
git submodule update --init    # libopus
cd android
./gradlew assembleDebug        # APK in android/app/build/outputs/apk/debug/
./gradlew testDebugUnitTest lintDebug
```

**PC**

```bash
cd pc
cargo build --release
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/), for example `fix(pc): skip unknown frame types`.

</details>

Questions and ideas go in [Discussions](https://github.com/diveshpatil9104/mikey/discussions). Please report security problems privately, as described in the [security policy](.github/SECURITY.md).

---

## License

[MIT](LICENSE). Copyright © 2026 Divesh Patil.
