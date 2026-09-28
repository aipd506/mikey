# The Four Connection Levels

Priority order (1 = best). Mikey always uses the best level currently available and upgrades automatically when a better one appears.

| | Level | Needs from the user | Bandwidth (practical) | Added latency | Audio | Video |
|---|---|---|---|---|---|---|
| **1** | **USB debugging (ADB)** | Developer options → USB debugging on; tap *Allow* once | 100+ Mbps | lowest (~5–15 ms) | PCM 48 kHz lossless | MJPEG up to 1080p30 |
| **2** | **USB tethering** | Toggle USB tethering each time it's plugged in | 50–300 Mbps | low (~5–20 ms) | PCM 48 kHz lossless | MJPEG up to 1080p30 |
| **3** | **Wi‑Fi / LAN** | Same network (or PC on phone's hotspot); allow once | 10–300 Mbps, variable | variable (~10–80 ms + jitter) | Opus 96 kbps, 10 ms (raw PCM optional) | MJPEG 720p30, adaptive |
| **4** | **Bluetooth** | Pair phone & PC once in OS settings | ~0.5–1.5 Mbps | medium (~40–120 ms) | Opus 32–48 kbps | **Not supported** |

## Level 1 — USB debugging (ADB)

**How it works.** The PC app runs `adb reverse tcp:7653 tcp:7653` for each authorized device. From then on, when the phone app connects to `127.0.0.1:7653`, ADB tunnels that connection over USB to the PC's `localhost:7653`. No IP addresses, no network, no discovery.

**PC side (`adb.rs`):**
- Uses the `adb` found on `PATH` if present (avoids killing a developer's own ADB server with a version mismatch); otherwise a bundled `adb` from Android platform-tools shipped next to `mikey` (verify redistribution terms before release — scrcpy ships the same way).
- Starts the ADB server only if the "USB debugging" level is enabled, and runs `adb track-devices` (event stream, no polling).
- On `device` state: `adb -s <serial> reverse tcp:7653 tcp:7653`. If *Open Mikey on phone when plugged in* is on and the app isn't in the foreground: `adb -s <serial> shell am start -n com.mikey/.MainActivity`.
- On `unauthorized` state: tray icon turns amber, notification *"Tap Allow on your phone"*, and the flyout displays an actionable hint card.
- Windows note: most phones work with the generic WinUSB/MTP-composite driver; some OEMs need their USB driver. Link to it from the flyout's "Needs attention" card if `adb` sees no device while a phone is plugged in.

**Phone side:** `AdbTransport.probe()` = try TCP connect to `127.0.0.1:7653` with a 300 ms timeout. Succeeds only if a reverse tunnel exists. Cheap enough to run on every trigger ([connection-levels.md](connection-levels.md)).

**Why it's #1:** zero configuration per session, fastest, most reliable, doesn't touch the PC's networking at all.

**Considered and rejected — Android Open Accessory (AOA):** would allow USB without debugging or tethering, but on Windows the accessory needs a WinUSB driver installed via a tool like Zadig. That's worse friction than turning on tethering.

## Level 2 — USB tethering

**How it works.** With tethering on, the phone becomes a USB network adapter (RNDIS or NCM) and the PC gets an IP on a private subnet. Mikey runs its normal TCP session over that link.

**Phone side (`TetherTransport`):**
- Detect the tether interface by enumerating `NetworkInterface`s that are up and named `rndis*`, `usb*` or `ncm*` with an IPv4 address. **Never assume `192.168.42.x`** — many Android versions randomize the subnet.
- Send a UDP discovery probe to that interface's broadcast address ([connection-levels.md](connection-levels.md)). The PC answers with its IP and port.
- Connect TCP, bound to that interface's local address so traffic can't leak onto Wi‑Fi.

**Friendly nudge:** if the phone sees it is USB-connected to a computer (`ACTION_POWER_CONNECTED` with `BATTERY_PLUGGED_USB`) but neither Level 1 nor Level 2 is available after 3 s, the drawer shows a one-line hint *"Turn on USB tethering for a wired connection ›"* that opens the tethering settings page. Shown at most once per plug-in.

**Caveats to document:**
- Android can't programmatically enable tethering; the user toggles it.
- The PC may start routing internet traffic through the phone. Android's USB tethering shares whatever the phone is using upstream — if the phone is on Wi‑Fi, that's Wi‑Fi (no mobile data cost); if not, it's mobile data. Document this; the phone app shows a one-time tip.
- Some carriers disable tethering on certain plans. Level 1 or 3 still works.

## Level 3 — Wi‑Fi / LAN

**How it works.** The phone broadcasts a UDP discovery probe on its Wi‑Fi (or hotspot) interface; the PC answers; the phone opens a TCP session. This also covers **the PC being connected to the phone's hotspot**.

- The phone tries the last known PC IP directly first (instant reconnect), then broadcasts.
- A `WifiManager.WifiLock` in low-latency mode (API 29+, high-perf below) is held while streaming, to stop Wi‑Fi power-save from adding latency spikes.
- Manual PC address (Advanced) is used when broadcast is blocked (client isolation, corporate networks).
- The PC installer adds a firewall rule for TCP 7653 / UDP 7654 on **private** networks only.

## Level 4 — Bluetooth

**How it works.** The PC runs an RFCOMM server and publishes an SDP record with Mikey's service UUID. The phone connects to the paired PC using `createRfcommSocketToServiceRecord(MIKEY_UUID)` (secure, bonded). The result is a byte stream that carries the same protocol.

**Phone side (`BluetoothTransport`):**
- Only considers **already-bonded** devices whose Bluetooth class is *Computer*. No scanning → no location permission needed.
- First time: tries each bonded computer (connect attempt with UUID, 4 s timeout each), caches the MAC that answered. After that: only the cached MAC.
- Permission `BLUETOOTH_CONNECT` (Android 12+) is requested the first time Level 4 is actually attempted — not at install.

**PC side (`bt.rs`):**
- Windows: Winsock `AF_BTH` socket, `bind` to any port, `WSASetService` to register the SDP record.
- Linux: BlueZ over D-Bus (`ProfileManager1.RegisterProfile`) — uses the `bluer` crate, which requires a small Tokio runtime **confined to this module's thread**.

**Media profile on Level 4:** audio only, Opus 32 kbps (48 kbps if link quality allows), 20 ms frames (larger frames suit Bluetooth's packet timing). The camera half shows the "unavailable" state; tapping it explains *"Camera isn't available over Bluetooth. Connect with USB or Wi‑Fi."*

**Caveats:** throughput drops if the PC is also streaming to Bluetooth headphones on the same radio; some PC Bluetooth stacks (especially cheap dongles) have unreliable RFCOMM. Level 4 is a fallback, not a flagship.

## When the phone probes (event-driven, battery-friendly)

The phone does **not** poll everything constantly. Probing is triggered by:

| Trigger | Probes |
|---|---|
| App/service start | All enabled levels, in priority order, in parallel with a short stagger |
| USB power connected | L1 immediately, again at +1 s, +3 s (ADB reverse takes a moment); L2 on interface change |
| Network interface added/removed (`ConnectivityManager.NetworkCallback` + interface enumeration) | L2, L3 |
| Bluetooth adapter on / bond change | L4 |
| Current transport dropped | All levels below and above it |
| While a USB cable to a computer is in and we're not on a USB level: every 1 s for the first 4 s (the PC's adb reverse takes a moment), then every 5 s | L1 (localhost connect) and a look for a tether interface (no traffic). Negligible cost. |

## Upgrade & downgrade (make-before-break)

```
 live on level N ──(better level M<N becomes reachable)──► open M, send HELLO(resume=session_token)
        ▲                                                         │
        │                                                  PC: same device id + valid token
        │                                                  → swap the session's transport
        │                                                  → reply WELCOME(resumed)
        │                                                         │
        └──── close level N ◄──── phone switches senders to M ◄───┘

 live on level N ──(N fails: socket error / heartbeat timeout)──► try the next available
                                                                   level in priority order;
                                                                   resume session if < 30 s
```

- The PC keeps the session (virtual devices stay "open" and output silence / hold the last video frame) for up to 30 s after a transport loss, so apps like Zoom never see the device vanish.
- Media profile is renegotiated on handover (e.g. Wi‑Fi Opus → USB PCM; USB video → Bluetooth no-video).
- Target glitch on upgrade: < 300 ms. On unplugging a USB cable mid-call with Wi‑Fi available: < 2 s.
- Hysteresis: a level that failed is not re-promoted for 10 s, to avoid flapping on a bad cable.

## Discovery beacon (UDP 7654)

Replaces mDNS. Tiny, interface-pinnable, no dependency.

```
Phone → broadcast   "MIKEY?1" | device_id(16 bytes) | name_len(1) | name(utf8)
PC    → unicast     "MIKEY!1" | pc_id(16) | tcp_port(u16 BE) | proto_ver(u8) | name_len(1) | name
```

- The PC only replies on the interface the probe arrived on.
- If several PCs answer: connect to the last used one; if none is known, show a one-time picker in the drawer.
