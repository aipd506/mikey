# Wire Protocol

## Framing

Every frame on every transport:

```
┌──────────┬────────────────────┬─────────────────┐
│ 1 byte   │ 4 bytes (u32 BE)   │ N bytes         │
│ type     │ payload length     │ payload         │
└──────────┴────────────────────┴─────────────────┘
```

Receivers **reject any length > 4 MiB** and close the connection (protects against garbage or malicious input).

## Frame types

| Type | Name | Direction | Payload |
|---|---|---|---|
| `0x00` | HELLO | phone → PC | JSON: `proto`, `device_id`, `device_name`, `level` (1–4), `token?`, `resume?`, `caps` |
| `0x10` | WELCOME | PC → phone | JSON: `pc_id`, `pc_name`, `token` (new or same), `resumed`, `pc_caps` (e.g. `aec`, `rnnoise`, `vcam`) |
| `0x11` | PENDING | PC → phone | empty — waiting for user approval (phone dot turns amber) |
| `0x12` | REJECT | PC → phone | JSON: `reason` (`denied`, `busy`, `version`, `bad_token`) |
| `0x01` | AUDIO | phone → PC | media header + PCM s16le **or** Opus packet |
| `0x02` | VIDEO | phone → PC | media header + one JPEG image |
| `0x03` | HEARTBEAT | both | 8-byte send timestamp (µs) — echoed by PC for RTT |
| `0x04` | CONTROL | both | JSON: bidirectional common settings sync (audio: ns, ns_strength, aec, gate_db, muted; video: on, lens, preview, aspect, fps) |
| `0x05` | BYE | both | JSON: `reason` — clean shutdown, no timeout wait |

### JSON field formats

- All JSON payloads are UTF-8.
- `proto`: integer, the major protocol version. Currently `1`.
- `device_id`, `pc_id`: the random 128-bit id as 32 lowercase hex characters.
- `caps` (phone): list of what the phone can send right now. `["audio"]` in Phase 1, `["audio", "video"]` from Phase 3.
- Receivers ignore JSON fields and frame types they don't know, so either side can add new ones without breaking the other.

### Control Frame Schema (`0x04 CONTROL`)

Both phone and PC emit and receive `0x04 CONTROL` frames to keep common user settings synchronized in real-time. Payload is a JSON object with optional `audio` and `video` sections. Updates are partial and idempotent:

```json
{
  "audio": {
    "ns": true,
    "ns_strength": 0.8,
    "aec": true,
    "gate_db": -45.0,
    "muted": false
  },
  "video": {
    "on": true,
    "lens": "front",
    "preview": true,
    "aspect": "16:9",
    "fps": 30
  }
}
```

Fields:
- `audio.ns`: boolean, toggles RNNoise noise suppression on PC.
- `audio.ns_strength`: float 0.0–1.0, suppression aggressiveness.
- `audio.aec`: boolean, toggles SpeexDSP acoustic echo cancellation.
- `audio.gate_db`: float -60.0 to -20.0, noise gate threshold (or null/absent for off).
- `audio.muted`: boolean, soft-mute audio pipeline.
- `video.on`: boolean, requests turning phone camera capture on/off.
- `video.lens`: string, `"back"`, `"front"`, or `"flip"` (switch lens).
- `video.preview`: boolean, whether preview window is active.
- `video.aspect`: string, `"16:9"`, `"4:3"`, `"1:1"`.
- `video.fps`: integer, `15` or `30`.

## Media header (inside AUDIO/VIDEO payloads)

```
┌─────────────┬──────────────────┬───────────┬──────────────┐
│ seq (u32)   │ capture_ts (u64, │ codec (u8)│ reserved (u8)│ + data
│             │ µs, phone clock) │           │              │
└─────────────┴──────────────────┴───────────┴──────────────┘
codec: 0x01 PCM s16le 48k mono · 0x02 Opus · 0x10 JPEG
```

`seq` gives loss/reorder stats; `capture_ts` lets the PC measure latency trends, drive the drift resampler, and align video with audio.

## Handshake

```
phone                                PC
  │── HELLO(level, token?) ─────────►│  version ok? token valid? trust rules ([sessions-trust.md](sessions-trust.md))
  │◄──────────── WELCOME ────────────│  (or PENDING … then WELCOME / REJECT)
  │── CONTROL(current settings) ────►│
  │── AUDIO/VIDEO/HEARTBEAT … ──────►│
```

Version rule: `proto` major mismatch → `REJECT(version)` and both sides tell the user to update the older one.

## Liveness

- Heartbeat every **2 s** in both directions when no other frame was sent in that window.
- The phone sends its heartbeat every 2 s even while it streams audio, and the PC echoes each one. So each side hears from the other at least every 2 s, and the phone gets its round-trip time.
- No frame received for **6 s** → transport considered dead → handover/reconnect logic.
- Socket errors (e.g. cable pulled) trigger this immediately, without waiting.
- Reconnect backoff: 0.5 s → 1 s → 2 s → 4 s → 5 s cap; resets on success.

## Future: UDP media on Wi‑Fi (Phase 5, optional)

If testing shows TCP head-of-line blocking causes audible stalls on busy Wi‑Fi, audio moves to UDP on Level 4 (same media header, Opus in-band FEC + loss concealment), while HELLO/CONTROL/HEARTBEAT stay on TCP. Not built unless measured to be necessary.
