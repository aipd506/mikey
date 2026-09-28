# Scope

## In scope (v1.0)

- Android 8.0+ (API 26+) phones, arm64-v8a and armeabi-v7a.
- Windows 10/11 x64 and mainstream Linux desktops (Ubuntu, Fedora, Arch; PipeWire or PulseAudio; X11 or Wayland).
- Mic streaming on all four connection levels.
- Camera streaming on Levels 1, 2 and 3.
- Front/back flip, aspect ratio, basic quality settings.
- Noise suppression, noise gate, echo cancellation — **all on the PC**; the phone sends raw audio.
- One active phone per PC at a time (multiple phones *known*, one *streaming*).

## Out of scope (v1.0)

- iOS app, macOS PC side.
- Phone as speaker (PC → phone audio).
- Screen mirroring, remote control, file transfer.
- Streaming over the internet / across networks. Mikey is local-only by design.
- Multiple simultaneous phones as separate virtual devices.
- Background blur/removal/replacement (meeting apps already do this well; see [open-decisions.md](open-decisions.md)).
