# Android Architecture & Engineering Style

The Android client (`android/`) is engineered for **ultra-low latency, maximum battery efficiency, and zero runtime bloat**. It rejects bloated enterprise Android conventions in favor of lean, deterministic systems programming in Kotlin, Compose, and C/NDK.

---

## 1. Architectural Philosophy

### 1.1 Strict Dependency Austerity
Mikey intentionally eliminates heavy, complex frameworks:
- **No Dependency Injection**: No Dagger, Hilt, or Koin. Dependencies are instantiated and passed directly via constructors.
- **No Heavy ORMs**: No Room or SQLite. Settings are persisted via lightweight Android `SharedPreferences` (`Settings.kt`).
- **No HTTP/REST Frameworks**: No Retrofit, OkHttp, or Ktor. All communication uses native TCP and RFCOMM sockets with custom binary framing.
- **No Analytics / Telemetry**: Zero Google Analytics, Firebase, Sentry, or ad SDKs.

### 1.2 Unidirectional State Flow (MVI Pattern)
```text
     User Interactions (Tap Mic, Tap Cam, Toggle Settings)
                             │
                             ▼
                    [MainActivity.kt]
                             │ Dispatches Intents (ACTION_MIC_ON, ACTION_FLIP)
                             ▼
                    [MikeyService.kt] ── owns capture & network
                             │ Emits immutable state
                             ▼
                 StateFlow<MikeyState>
                             │ Observes state
                             ▼
                    [MainScreen.kt] (Pure Jetpack Compose UI)
```

- **UI Never Owns Logic**: `MainActivity` and Compose composables are strictly visual renderers. They maintain zero business logic or streaming state.
- **Service as Single Source of Truth**: `MikeyService` holds the active session and exposes state via `StateFlow<MikeyState>` and audio levels via `StateFlow<Float>`.

---

## 2. Service Lifecycle & Permissions

### 2.1 Dynamic Foreground Service Types (`MikeyService.kt`)
On Android 11+ (API 30+), background microphone and camera access require explicit foreground declarations:
```kotlin
if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
    var types = 0
    if (state.micOn) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_MICROPHONE
    if (state.camera.on) types = types or ServiceInfo.FOREGROUND_SERVICE_TYPE_CAMERA
    startForeground(Notifier.ID, notification, types)
}
```
- When only the mic is streaming, only `FOREGROUND_SERVICE_TYPE_MICROPHONE` is requested.
- When only the camera is streaming, only `FOREGROUND_SERVICE_TYPE_CAMERA` is requested.
- When both are toggled off, `stopSelf()` is called immediately and the foreground notification dismisses.

### 2.2 `START_NOT_STICKY` & Clean Exit
- `onStartCommand` returns `START_NOT_STICKY`. If Android kills the service under memory pressure, it **must never automatically restart** with the microphone or camera active.
- `android:stopWithTask="true"` is declared in `AndroidManifest.xml`. Swiping the app away from Android Recents stops the service and terminates all sensor capture immediately.

---

## 3. Real-Time Threading & Native JNI Integration

### 3.1 Audio Ingest Thread
Audio capture must never experience garbage collection pauses or thread preemption:
- **Urgent Priority**: The capture loop executes on a dedicated OS thread configured with:
  ```kotlin
  Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
  ```
- **Native AAudio Engine (`app/src/main/cpp/aaudio_jni.c`)**:
  Interacts with the Android NDK AAudio API to open direct streams in `AAUDIO_PERFORMANCE_MODE_LOW_LATENCY` with `AAUDIO_SHARING_MODE_EXCLUSIVE`.
- **Native Opus Engine (`app/src/main/cpp/opus_jni.c`)**:
  Compiles the official reference C `libopus` library. 10 ms PCM chunks are compressed with zero Java-heap garbage generation.

### 3.2 Non-Blocking Capture Handoff
Capture threads (`mikey-capture`, `mikey-video`) must **never block on network sockets or synchronization locks**:
- Capture callbacks write into bounded queues (`ArrayBlockingQueue<AudioFrame>(8)` and `ArrayBlockingQueue<VideoFrame>(2)`).
- If network congestion prevents the sender thread from draining the queue in time, new frames drop older frames immediately (`offerDroppingOldest`).

---

## 4. Coding Conventions & Anti-AI-Slop Rules

1. **No Chatty Comments**:
   Comments that narrate syntax or state the obvious are forbidden.
   - *Bad*: `// increment sequence number by one`
   - *Good*: `// 20° hysteresis threshold prevents rotation flickering when tilted near diagonal`
2. **Lean Idiomatic Kotlin**:
   Favor standard library scope functions (`use`, `let`, `also`, `run`) over verbose boilerplate classes. Avoid premature abstractions, builder hierarchies, or single-method interface wrappers.
3. **Immutability First**:
   All protocol and state representations use immutable `data class` models with `val` properties. State modifications create copies via `.copy()`.
4. **Surgical Diffs**:
   Keep edits minimal, targeted, and focused strictly on the problem at hand without reformatting unrelated code.
