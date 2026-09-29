# Troubleshooting

Common issues and solutions for Mikey. If your problem isn't listed here, [open a Discussion](https://github.com/diveshpatil9104/mikey/discussions) or [file a bug report](https://github.com/diveshpatil9104/mikey/issues/new?template=bug_report.yml).

---

## Connection Issues

### Phone can't find the PC (Wi-Fi)

**Symptoms:** Phone shows "Searching..." indefinitely on Wi-Fi.

**Solutions:**
1. Ensure phone and PC are on the **same network** (or phone's hotspot).
2. Check the PC firewall allows incoming connections on:
   - **TCP 7653** (streaming protocol)
   - **UDP 7654** (discovery beacon)
3. Some routers enable "AP isolation" or "client isolation" which blocks device-to-device traffic. Check your router settings.
4. If on a corporate/university network, local device discovery may be blocked. Try USB or phone hotspot instead.

### USB debugging connection not working

**Symptoms:** Phone is plugged in via USB but Mikey doesn't connect via Level 1.

**Solutions:**
1. Ensure **USB Debugging** is enabled: Settings → Developer Options → USB Debugging.
2. If you see an "Allow USB debugging?" prompt on the phone, tap **Allow** (and check "Always allow from this computer").
3. Verify ADB sees the device: `adb devices` - should show your device as "device" (not "unauthorized").
4. Try a different USB cable. Data-only cables (not charge-only) are required.
5. On Windows, you may need [Google USB Driver](https://developer.android.com/studio/run/win-usb) or your phone manufacturer's driver.

### Bluetooth not connecting

**Symptoms:** Phone won't connect via Bluetooth RFCOMM.

**Solutions:**
1. **Pair first:** Mikey does not handle Bluetooth pairing. Pair your phone and PC through OS Bluetooth settings.
2. Bluetooth provides audio only (Level 4) - video is not available over Bluetooth.
3. On Linux, ensure BlueZ is running: `systemctl status bluetooth`.
4. Some PCs have unreliable Bluetooth adapters. Try a USB Bluetooth 5.0 dongle.

---

## Audio Issues

### No audio on the PC side

**Symptoms:** Phone shows "Live" / mic is on, but no audio comes through on the PC.

**Solutions:**
1. **Check virtual mic setup:**
   - **Windows:** Ensure [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) is installed. In your meeting app, select **"CABLE Output"** as the microphone.
   - **Linux:** Mikey creates "Mikey Microphone" automatically via PipeWire/PulseAudio. Select it in your app's audio settings.
2. **Check Mikey is receiving:** Look at the VU meter in the PC tray flyout. If it's showing activity, the audio is reaching the PC - the issue is in your meeting app's mic selection.
3. **Check mic permission on Android:** Settings → Apps → Mikey → Permissions → Microphone must be "Allowed".

### Echo or feedback

**Symptoms:** The remote person hears themselves echoed back.

**Solutions:**
1. **Enable AEC:** In the Mikey phone app settings, enable "Echo Cancellation".
2. **Check echo reference device:** In the PC tray flyout, ensure the echo reference is set to your speakers/headphones output.
3. **Use headphones:** The simplest fix. AEC works best with speakers, but headphones eliminate echo entirely.

### Audio sounds robotic or choppy

**Symptoms:** Audio has artifacts, glitches, or sounds like a robot.

**Solutions:**
1. **Check connection quality:** Wi-Fi and Bluetooth may have packet loss. Switch to USB if possible.
2. **Check CPU usage:** If the PC is under heavy load, the audio pipeline may drop frames. Close unnecessary apps.
3. **Bluetooth quality:** Bluetooth (Level 4) uses lower bitrate Opus (48 kbps). Some quality degradation is normal.

---

## Video Issues

### Meeting app doesn't show Mikey camera

**Solutions:**
1. **Windows:** The softcam virtual camera must be registered. If you used the installer, this is automatic. If building from source, run `regsvr32 softcam.dll` from an admin command prompt.
2. **Linux:** Ensure `v4l2loopback` is loaded: `sudo modprobe v4l2loopback`.
3. Restart your meeting app after setting up the virtual camera. Some apps only enumerate cameras at startup.
4. Video is not available on Bluetooth (Level 4).

### Camera preview is sideways or upside down

**Solutions:**
1. Mikey handles rotation automatically. If the image is wrong, check that your phone's auto-rotate is not locked.
2. File a bug report with your phone model - some devices report incorrect sensor orientation.

---

## Build Issues

### Android: CMake error during build

**Solution:** Install CMake via Android Studio → SDK Manager → SDK Tools → CMake. Or install system CMake (`brew install cmake` / `apt install cmake`).

### Android: "git submodule" errors

**Solution:** Run `git submodule update --init` from the repo root to pull libopus.

### PC: Missing system libraries on Linux

**Solution:**
```bash
# Ubuntu/Debian
sudo apt install build-essential libasound2-dev libdbus-1-dev

# Bluetooth support
sudo apt install libbluez-dev
```

### PC: Build fails on Windows

**Solution:** Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the "Desktop development with C++" workload.

---

## Still stuck?

1. Check the [open issues](https://github.com/diveshpatil9104/mikey/issues) - someone may have reported the same problem.
2. Ask in [Discussions](https://github.com/diveshpatil9104/mikey/discussions).
3. [File a bug report](https://github.com/diveshpatil9104/mikey/issues/new?template=bug_report.yml) with your environment details and logs.
