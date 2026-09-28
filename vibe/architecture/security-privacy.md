# Security & Privacy

- **Local only.** No component ever talks to the internet. No update checker phoning home in v1 (users update via GitHub/F-Droid).
- **Mic & camera start off** on every launch (unless the user opts into *Remember state*).
- **Android's own privacy indicators** (green dot) always show when mic/camera are in use — Mikey never tries to hide them.
- **Trust:** pairing tokens + trust-on-first-use for Wi‑Fi ([sessions-trust.md](sessions-trust.md)). USB and Bluetooth imply physical proximity / OS pairing.
- **Input hardening:** max frame size, JSON size limits, strict parsing; a malformed frame closes that connection only.
- **Encryption:** Bluetooth links are encrypted by the OS; USB is physical. **Wi‑Fi traffic is unencrypted in v1**, which is acceptable on home networks and documented clearly. Phase 5 adds optional AES-GCM on Level 3 (Wi‑Fi) with a key established during the first USB/Bluetooth pairing (or displayed-code pairing).
- **Firewall:** the PC only opens ports on private networks.
- **No logs of media.** Logs contain events and device names only.
