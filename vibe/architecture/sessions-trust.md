# Sessions, Identity & Trust

## Identity

- **Phone:** `device_id` = random 128-bit, generated at first launch, stored in prefs. `device_name` = `Build.MODEL` (user-renamable later).
- **PC:** `pc_id` = random 128-bit in `config.toml`. `pc_name` = hostname.
- **Pairing token:** 32 random bytes issued by the PC the first time a device is accepted, stored on both sides. Sent in `HELLO` on every later connection.

## Session

- A session = one phone streaming to the PC. It survives transport changes ([connection-levels.md](connection-levels.md)) via a `session_token`.
- **One active session** per PC. Other known devices are listed in the tray flyout; switching is explicit.

## Trust rules

| Situation | Ask before joining = **Off** (default) | Ask before joining = **On** |
|---|---|---|
| Known device (valid token), any level | Join silently | Prompt |
| New device via **USB (L1/L2)** | Join silently, issue token (physical access = trust) | Prompt |
| New device via **Wi‑Fi (L3)** | **Prompt once**, then remembered | Prompt |
| New device via **Bluetooth (L4)** | Join silently, issue token (OS pairing already required) | Prompt |
| Another device while one is streaming | Prompt "Switch?" | Prompt "Switch?" |

Why Wi‑Fi asks once even with the toggle off: on shared networks (dorms, offices, cafés) anyone with the app could otherwise pipe audio into your microphone. For a single-device user this costs exactly one click, ever. Power users can disable it: *Advanced → Trust new Wi-Fi devices automatically*.

**Forget** (tray flyout or phone) deletes the token on that side; the next connection is treated as new.
