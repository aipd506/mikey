# Skill: Build and run the Android app

**Use when:** building, installing or debugging the phone app.

## Setup

- Android Studio, or JDK 17+ and the Android SDK.
- `android/local.properties` with `sdk.dir=…` — Android Studio writes it on first open. It is gitignored.
- Open the **`android/`** folder in Android Studio, not the repo root.

## Build and install

From `android/`:

```sh
./gradlew assembleDebug     # APK → app/build/outputs/apk/debug/
./gradlew installDebug      # install on the connected phone
```

## Phone

1. Developer options → USB debugging on.
2. Plug in → tick *Always allow from this computer* → Allow.
3. Check: `adb devices` lists the phone as `device`.
4. Logs: `adb logcat --pid=$(adb shell pidof com.mikey)`

## Level 1 without the PC app

The PC app normally sets up the tunnel ([connection-levels.md](../architecture/connection-levels.md)). When testing by hand:

```sh
adb reverse tcp:7653 tcp:7653
```

The phone's `127.0.0.1:7653` now reaches `localhost:7653` on the computer.

## Wi-Fi testing (debug builds)

Long-press the status dot and type the PC's address. Leave it empty to go back to USB. On the emulator, your computer is `10.0.2.2`.

The emulator's network doesn't pass broadcasts or their replies, so discovery can't find a PC there. Type the address once instead: it is remembered as the last known address and used automatically from then on. Real discovery needs a real phone on the same Wi-Fi as the PC.

## Before committing

- Format with `ktlint` ([coding-rules.md](../rules/coding-rules.md), rule 8).
- APK size stays within budget — ≤ 6 MB per ABI on a release build ([performance-budgets.md](../rules/performance-budgets.md)).
