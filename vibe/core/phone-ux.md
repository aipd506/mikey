# 6. Phone App — UX Specification

## 6.1 Layout

The screen is split into two equal halves. Positions never change — the layout is locked to portrait; only glyphs and text rotate ([§ 6.4](phone-ux.md)).

```
┌──────────────────────────────────┐
│ [⟲]                              │  ← flip-camera button, top-left
│                                  │     (visible only when camera is on)
│                                  │
│           ( CAM ICON )           │  ← top half: camera
│                                  │     off  = dimmed outline icon on black
│                                  │     on   = white circle, red on-air dot, lens label (no video)
│                                  │
├──────────────── ˄ ───────────────┤  ← chevron, centered on the split line
│                                  │     tap → settings drawer
│                                  │
│           ( MIC ICON )           │  ← bottom half: microphone
│                                  │     off  = dimmed outline icon
│                                  │     on   = white circle, red on-air dot, level ring
│                                  │
│                              [●] │  ← status dot, bottom-right
└──────────────────────────────────┘
```

- **Each half is one giant tap target.** Tap anywhere in the top half toggles the camera; anywhere in the bottom half toggles the mic.
- **Flip button** (top-left) sits diagonally opposite the status dot (bottom-right). It is a small circular button (40 dp, black with a hairline border) and only appears while the camera is on. Tapping flips front ↔ back live.
- **Mic level ring:** when the mic is on, a thin ring around the mic icon reflects input level. This is the one allowed "live" element — it proves audio is actually flowing, which is the core honesty promise.
- **Short labels.** Under each circle a label in capitals says the state (*MIC LIVE*, *CAMERA OFF · TAP TO START*), plus one sentence when the user has to act. No timer, no camera preview, and no text next to the status dot. TalkBack reads the labels in sentence case.

## 6.2 States

| Element | Off | On | Unavailable |
|---|---|---|---|
| Camera half | Dim outline circle, *"Camera off · tap to start"* | White circle with the red on-air dot and the lens (*"Back camera"*); a white outline with *"· waiting for PC"* or *"· not reaching PC"* until the PC receives. The video shows only on the PC | Dashed circle, slashed icon, *"Camera · unavailable"*, and the reason on tap (*"Camera isn't available over Bluetooth. Connect with USB or Wi-Fi."*) |
| Mic half | Dim outline circle, *"Mic off · tap to start"* | White circle, red on-air dot and the level ring (*"Mic live"*); a slashed icon while muted; a white outline and one sentence while it isn't reaching the PC | Dashed circle, slashed icon, *"Mic · no permission"* and *"Microphone access is off. Tap to allow."* |
| Status dot | — | Green = streaming path live | Red = no PC · Amber = waiting for PC approval |

The status dot has three colors (not two) because *waiting for approval* is a real state the user must be told about; everything else stays binary.

## 6.3 Settings drawer (chevron)

A bottom sheet that slides up to ~60% height. One scroll, no nested screens. Closes on tap-outside or chevron. The **Advanced** section is collapsed by default — expanding it is the "extra click" for power users.

```
  ─────────────  (drag handle)
  Connected to  DESKTOP-ANU  ·  USB (debugging)

  Camera
    Aspect ratio          [16:9] [4:3] [1:1]
  Mic
    Mute                  [ toggle ]            ← only while the mic is on
  Audio
    Noise suppression     [ toggle ]            ← runs on the PC
    Echo cancellation     [ toggle ]            ← runs on the PC; greyed if PC lacks it

  ▸ Advanced
      Noise suppression strength ──────●──  (default: high)
      Noise gate threshold      ──●──────  (−60 dB … −20 dB, Off at far left)
      Lossless audio on Wi-Fi   [ toggle ]  (off)  ← raw PCM instead of Opus
      Video quality             [Auto] [720p] [1080p]
      Frame rate                [Auto] [30] [15]
      Keep screen on            [ toggle ]  (off)
      Remember mic/camera state [ toggle ]  (off)  ← see § 6.6
      Connection levels         USB debugging ✓  USB tethering ✓  Wi-Fi ✓  Bluetooth ✓
      Manual PC address         [ 192.168.1.20 ]   ← only needed on locked-down networks
      Paired computers          DESKTOP-ANU  [Forget]
      Open USB tethering settings  ›

  ─────────────
  Mikey  v1.0.0 · github.com/[username]/mikey
```

Notes:
- Flip camera is on the main screen button `[⟲]` and also triggered remotely from the PC flyout. The last used lens is remembered.
- **All audio processing runs on the PC.** The phone only captures raw audio (and compresses it where the link needs it). This keeps the phone cool and battery-light, gives the echo canceller a clean signal, and makes quality identical on every phone brand.
- **Bidirectional Common Settings:** All audio DSP and video controls (Noise suppression toggle & strength, Echo cancellation toggle, Noise gate threshold, Soft mute, Camera lens flip) are common settings, shared with the PC flyout; the preview window is PC only. The phone sends a change the moment it's made (a slider when the finger lifts), and a change from the PC updates the open sheet at once. Modifying them on the phone transmits a `0x04 CONTROL` frame to the PC; modifying them on the PC flyout transmits a `0x04 CONTROL` frame to the phone. Both sides stay in exact sync. Settings are disabled or greyed out if either device lacks the capability.

## 6.4 Rotation behaviour

- The activity is locked to portrait (`android:screenOrientation="portrait"`), so the layout never reflows and the activity is never recreated mid-stream.
- An `OrientationEventListener` snaps a `rotation` value to 0/90/180/270 (with ~20° hysteresis to avoid flicker). All icons and the drawer's text container rotate by that value. Positions stay fixed.
- Camera frames are rotated using the device orientation so the **PC always receives an upright image** regardless of how the phone is held.

## 6.5 Notification & lifecycle

A foreground service (`MikeyService`) owns the connection and all capture. The activity is only a remote control.

- **While connected**, the notification is always present (Android requires it) and shows: `Mic on · Camera off · USB` with actions **Mute/Unmute mic**, **Stop**.
- **Minimize / lock screen:** streaming continues. The notification reflects live state.
- **Swipe away from Recents:** `android:stopWithTask="true"` + `onTaskRemoved()` → send `BYE`, close transport, release mic and camera, stop service. The PC marks the device disconnected immediately (it received `BYE`) rather than waiting for a heartbeat timeout.
- **Stop in notification:** same as swipe-away, activity finishes if open.
- **Disconnected by the PC** (BYE `disconnect`): release mic and camera and stop the service, like Stop. No reconnecting until the user turns something on again.
- **Idle (connected, nothing streaming):** service stays up with the `connectedDevice` type so switching the mic on is instant. If the app is backgrounded and idle for 10 minutes, the service disconnects and stops to save battery.

**Android 14+ constraint (important):** a microphone or camera foreground service can only be *started* while the app is in the foreground. Therefore:
- Turning the mic/camera **on** happens only from the app UI.
- The notification's mic action is a **soft mute** (audio keeps being captured but silence is sent / sending pauses), which is allowed from the background. Unmute is also soft. Full release happens when the user turns the mic off in the app or stops the session.

## 6.6 Persistence ("local cache")

Everything the user sets is saved immediately to `SharedPreferences` and restored at next launch:

| Key | Default |
|---|---|
| `camera.lens` | back |
| `camera.aspect` | 16:9 |
| `camera.quality` / `camera.fps` | auto / auto |
| `audio.ns` / `audio.nsStrength` | on / high |
| `audio.wifiLossless` | off |
| `audio.aec` | on |
| `audio.gateDb` | off |
| `ui.keepScreenOn` | off |
| `ui.rememberState` | off |
| `ui.lastMicOn`, `ui.lastCameraOn` | off (what *Remember mic/camera state* restores) |
| `levels.enabled` | all four |
| `pc.lastId`, `pc.lastIp`, `pc.btAddress`, `pc.token`, `pc.lastLevel` | learned from the PC |
| `device.id` | random 128-bit, generated once |

**Mic and camera always start off** — this is a privacy promise. If *Remember mic/camera state* is turned on, Mikey restores the previous on/off state at launch (still only from the foreground, per Android rules).
