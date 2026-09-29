# Video Pipeline & Virtual Camera Architecture

Mikey transforms an Android smartphone's camera sensors into an upright, plug-and-play webcam for PC videoconferencing. The video pipeline is engineered for **zero battery runaway, automatic gravity orientation, and sub-frame delivery latency** using a strict `KEEP_ONLY_LATEST` queue discipline.

---

## 1. End-to-End Video Architecture

```text
[Phone Camera Sensor]
        │
        ▼
[CameraX ImageAnalysis] (VideoCapture.kt)
    ├── Strategy: STRATEGY_KEEP_ONLY_LATEST (Zero queue backlog)
    ├── Output Rotation: setOutputImageRotationEnabled(true)
    └── Orientation: OrientationEventListener with 20° hysteresis threshold
        │
        ▼
[Color Conversion & Cropping] (Nv21.kt)
    ├── Plane linearization: YUV_420_888 to contiguous NV21
    └── Optional 1:1 Aspect Ratio: cropNv21Square()
        │
        ▼
[Adaptive JPEG Compression] (compressToJpeg)
    ├── Thermal Aware: scales 80% down to 50% under thermal throttling
    └── Backlog Aware: sheds quality if network buffer stalls
        │
        ▼
[Wire Protocol Socket (0x02 VIDEO Frame)]
        │
        ▼
[PC Background Decode Thread] (latest_jpeg slot + Condvar)
        │
        ▼
[TurboJPEG SIMD Decompression] (decoder.rs)
    ├── Decompresses directly to 32-bit BGRA buffer
    └── Aspect ratio scaling and letterboxing
        │
        ├──► [Windows DirectShow Virtual Camera Filter] (vcam/softcam.dll)
        │         │
        │         ▼
        │    [Videoconferencing Apps: Zoom, Teams, Meet, OBS]
        │
        └──► [Native Win32 Floating Preview Window] (preview.rs)
```

---

## 2. Android Video Capture Engine

### 2.1 CameraX Configuration (`VideoCapture.kt`)
- **No Local GPU Preview**: Mikey deliberately does not display the camera feed on the phone screen. Bypassing the local preview surface cuts device power consumption by ~40% and prevents rapid thermal throttling.
- **`STRATEGY_KEEP_ONLY_LATEST`**: CameraX automatically drops intermediate frames if the analyzer thread is busy. A frame is never queued behind another; the pipeline only operates on the freshest optical data.
- **Service Lifecycle**: Binds directly to a custom `ServiceLifecycle.kt` rather than an `Activity`, keeping the camera stream alive when the user switches to other phone apps.

### 2.2 Gravity & Orientation Tracking (`surfaceRotationFor`)
Smartphones are often placed on desktop stands or clamped in horizontal mounts. Mikey guarantees the image arrives upright on the PC using accelerometer hysteresis:

```kotlin
internal fun surfaceRotationFor(degrees: Int, current: Int): Int {
    val candidate = when (degrees) {
        in 45 until 135 -> Surface.ROTATION_270
        in 135 until 225 -> Surface.ROTATION_180
        in 225 until 315 -> Surface.ROTATION_90
        else -> Surface.ROTATION_0
    }
    if (candidate == current) return current
    val center = when (current) {
        Surface.ROTATION_270 -> 90
        Surface.ROTATION_180 -> 180
        Surface.ROTATION_90 -> 270
        else -> 0
    }
    val distance = minOf((degrees - center + 360) % 360, (center - degrees + 360) % 360)
    // Must swing 20° past the 45° boundary (65° total) to change rotation
    return if (distance > 65) candidate else current
}
```
This **20° hysteresis threshold** prevents the video feed from violently flickering between portrait and landscape when the phone is tilted near a 45° diagonal angle.

### 2.3 Color Plane Mapping & Cropping (`Nv21.kt`)
- **YUV to NV21 Conversion (`yuv420ToNv21`)**: CameraX yields `ImageProxy` instances with three independent planar buffers (`Y`, `U`, `V`) featuring variable row strides and pixel strides. Mikey linearizes these planes into a single contiguous semi-planar NV21 buffer (`Y` plane followed by interleaved `VU` bytes).
- **Square Crop (`cropNv21Square`)**: For modern 1:1 portrait video feeds, `cropNv21Square` crops the central square region directly in native byte buffers before compression.

### 2.4 Thermal & Backlog Adaptive JPEG Compression
JPEG compression quality dynamically adapts to runtime conditions:
- **Thermal Monitoring**: Android's `PowerManager.OnThermalStatusChangedListener` reports thermal throttling events (`THERMAL_STATUS_MODERATE`, `SEVERE`).
- **Network Backlog**: If the socket output buffer experiences congestion, `backlog` is flagged.
- **Quality Adaptation**:
  - Normal condition: Quality = **80%** (sharp details, minimal artifacts).
  - Thermal / Congestion condition: Quality scales down to **50–60%**, reducing packet payload size by up to 60% and preserving 30 fps throughput.

---

## 3. PC Video Pipeline & Virtual Driver Integration

### 3.1 Worker Thread & Zero-Backlog Slot (`pc/src/video/pipeline.rs`)
1. **Latest JPEG Slot**: Incoming JPEG payloads from network frames (`0x02 VIDEO`) are placed in an atomic `Arc<(Mutex<JpegFrameSlot>, Condvar)>`.
2. **Immediate Overwrite**: If a frame is already waiting when a newer frame arrives, the older frame is **overwritten immediately**, ensuring zero backlog accumulation.
3. **SIMD TurboJPEG Decompression**: The background worker thread wakes, decompresses the JPEG directly into a reusable `BGRA` memory buffer, and updates the virtual camera driver.

### 3.2 Windows DirectShow Virtual Camera (`pc/src/video/vcam/`)
- **DirectShow Registration (`install.rs`)**: Registers a lightweight DirectShow source filter (`softcam.dll`) in the Windows Registry under `HKCR\CLSID` without requiring system reboots.
- **Universal Application Support**: Exposes the stream as a standard hardware webcam named *"Mikey Cam"*, compatible with Zoom, Microsoft Teams, Google Meet, Discord, and OBS Studio.
- **One Fixed Size (`vcam/mod.rs`)**: Mikey Cam is always 1920×1080 at 30 fps and is created once, when the PC app starts. Chrome, Edge and similar apps remember a camera's sizes from when they last listed cameras, and list a DirectShow camera again only after a real camera is added or removed; softcam only serves the size it was created at. Mikey Cam used to be recreated at the phone's size when the phone connected, so on PCs whose meeting app had opened a built-in camera first, switching to Mikey Cam showed nothing until a camera was toggled in Device Manager.
- **Scaling & Letterboxing (`DecodedFrame::letterbox_into`)**: Pictures of any other size or shape (720p, 4:3, square, portrait) are scaled to fit with bilinear scaling, with black bars where the shape differs. Each source row is scaled across once and reused, so 720p to 1080p costs about 3 ms a frame.

### 3.3 Privacy Placeholder Frame
When the user toggles the camera OFF from either the phone or the PC tray:
- Mikey **never closes the virtual camera driver**. Closing the driver causes videoconferencing apps to display "Camera Disconnected" errors or freeze video graphs.
- Instead, `DecodedFrame::placeholder(1920, 1080)` pushes a clean, dark neutral placeholder card featuring a calm privacy glyph, preserving the virtual device handle while guaranteeing complete user privacy.

### 3.4 Floating Native Preview Window (`pc/src/video/preview.rs`)
- Zero-dependency, lightweight Win32 floating window displaying the live camera stream.
- Allows users to check lighting, framing, and focus before joining meetings.
- Toggled instantly from the PC companion tray icon or flyout card.
