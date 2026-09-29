# Installing and Running Mikey on PC

Mikey for PC runs as a standalone tray application (`mikey`) that acts as the receiver server for audio and webcam streaming from your Android phone.

---

## 1. Quick Start (Pre-Built Binaries)

### Windows 10 / 11
1. Download `Mikey-Setup-x.y.z.exe` from Releases.
2. Run the installer (requires administrator privilege to register the local private firewall rule for TCP port 7653 and UDP port 7654).
3. Install **[VB-Audio Virtual Cable](https://vb-audio.com/Cable/)** (free). Mikey outputs audio to **"CABLE Input"**, and your meeting apps (Zoom, Teams, Google Meet, Discord) select **"CABLE Output"** as their microphone.
4. Launch Mikey. A small dot appears in your system tray (Notification Area).

### Linux (Ubuntu / Debian / Arch / Fedora)
- **Debian / Ubuntu:** `sudo dpkg -i mikey_x.y.z_amd64.deb`
- **AppImage:** `chmod +x Mikey-x.y.z.AppImage && ./Mikey-x.y.z.AppImage`
- **Virtual Microphone on Linux:** Mikey automatically creates a virtual null-sink remap source using PipeWire/PulseAudio (`mikey_sink`), appearing as **"Mikey Microphone"** in your audio settings.

---

## 2. Building from Source

### Prerequisites
- **Rust Toolchain:** Stable Rust (1.80+) with `cargo`.
- **Windows:** Microsoft Visual C++ Build Tools or MinGW GNU toolchain.
- **Linux:** `build-essential`, `libasound2-dev`, `libdbus-1-dev`.

### Build & Run
```bash
# Clone the repository
git clone https://github.com/diveshpatil9104/mikey.git
cd mikey/pc

# Build release binary
cargo build --release

# Run Mikey tray app
cargo run --release

# Run self-test mode (plays a 3-second test tone into virtual mic)
cargo run -- --test-tone
```

---

## 3. Firewall Configuration

If running without the installer, ensure incoming connections on private networks are allowed:
- **TCP port 7653:** Streaming protocol (HELLO, CONTROL, AUDIO, VIDEO).
- **UDP port 7654:** Discovery beacon responder.

### Windows (PowerShell as Admin)
```powershell
netsh advfirewall firewall add rule name="Mikey TCP" dir=in action=allow protocol=TCP localport=7653 profile=private
netsh advfirewall firewall add rule name="Mikey UDP Beacon" dir=in action=allow protocol=UDP localport=7654 profile=private
```

### Linux (UFW)
```bash
sudo ufw allow 7653/tcp comment "Mikey TCP"
sudo ufw allow 7654/udp comment "Mikey UDP Beacon"
```

---

## 4. Connection Levels

1. **Level 1 (USB Debugging):** Plug in phone with USB debugging enabled. Mikey's background ADB watcher automatically runs `adb reverse tcp:7653 tcp:7653`.
2. **Level 2 (USB Tethering):** Plug in USB and enable USB tethering in Android settings.
3. **Level 3 (Wi-Fi):** Connect phone to the same local network or phone's mobile hotspot. The phone discovers the PC automatically via UDP beacon.
4. **Level 4 (Bluetooth):** Pair phone and PC via Bluetooth. Audio streams over RFCOMM.
