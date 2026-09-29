# Installing Mikey on Android

## Option 1: Download the APK (Recommended)

1. Go to [Releases](https://github.com/diveshpatil9104/mikey/releases)
2. Download the latest `mikey-vX.Y.Z.apk`
3. On your phone:
   - Open Settings → Security → enable "Install from unknown sources" for your browser (or Files app)
   - Open the downloaded APK and tap Install
4. Launch Mikey

> **Note:** A signed APK and Google Play release are planned for v1.0.0.

## Option 2: Build from Source

### Prerequisites

- **Android Studio** (latest stable) - [download](https://developer.android.com/studio)
  - Or: JDK 17+ with Android SDK command-line tools
- **CMake** - for the libopus native build
  - Android Studio: SDK Manager → SDK Tools → CMake
  - macOS: `brew install cmake`
  - Linux: `sudo apt install cmake`
- **Android phone** with USB debugging enabled (Settings → Developer Options → USB Debugging)
- Minimum SDK: **26** (Android 8.0 Oreo)

### Build

```bash
# Clone the repo (if you haven't already)
git clone https://github.com/diveshpatil9104/mikey.git
cd mikey

# Initialize submodules (libopus)
git submodule update --init

# Build the debug APK
cd android
./gradlew assembleDebug
```

The APK is output to:
```
android/app/build/outputs/apk/debug/app-debug.apk
```

### Install on Device

```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

Or drag the APK onto the Android Studio device manager.

## First Run

1. **Grant permissions** when prompted:
   - Microphone - required for audio streaming
   - Camera - required for video streaming
   - Notification - required for the foreground service indicator
2. **Connect to your PC:**
   - **USB:** Plug in with USB debugging enabled. Mikey connects automatically.
   - **Wi-Fi:** Join the same network as your PC. Mikey discovers the PC automatically.
   - **Bluetooth:** Pair phone and PC in OS Bluetooth settings first.
3. **Tap mic or camera** to start streaming.

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues and fixes.
