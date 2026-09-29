# Developer Playbooks, Workflows & Operational Skills

This document provides operational playbooks and engineering protocols for developers and AI agents working on the Mikey repository.

---

## 1. The 18 Non-Negotiable Coding Rules

These rules apply across all modules and must be adhered to without exception:

1. **Build Strictly What the Current Phase Needs**: Never build speculative abstractions, future-proof interfaces, or unused utility functions for future phases.
2. **One Responsibility per File**: A module has one clear concern. `tcp.rs` handles TCP streaming; `beacon.rs` handles discovery; `AudioCapture.kt` handles microphone input.
3. **No Dependency Without Authorization**: Every library, crate, or Gradle plugin must be approved and listed in the architecture documentation. Never introduce unvetted third-party packages.
4. **UI Never Owns Logic**: User interfaces are strictly dumb observers. Jetpack Compose observes `StateFlow` from `MikeyService`; the PC flyout reflects `SessionManager`.
5. **Real-Time Paths Never Block**: The audio capture callback and audio output sink must never wait on network sockets, file I/O, or locks held by slow threads. Drop stale data rather than stalling audio graphs.
6. **Bounded Queues and Explicit Timeouts Everywhere**: Every socket must configure read/write timeouts. Every queue and channel must enforce strict capacity bounds. Unbounded allocations are forbidden.
7. **Platform Code Behind Strict Abstractions**: OS-specific code (Win32, Linux BlueZ, PulseAudio) is strictly gated behind `#[cfg(windows)]`, `#[cfg(target_os = "linux")]`, or platform interfaces.
8. **Static Analysis & Formatting**: Android code must pass linting; PC code must pass `cargo fmt --check` and `cargo clippy -- -D warnings` with zero warnings before merging.
9. **English Everywhere**: Code identifiers, comments, documentation, and commit messages must be in concise English.
10. **Comments Explain "Why", Not "What"**: Do not narrate obvious syntax. Write comments only to document non-obvious hardware invariants, timing quirks, or architectural rationale.
11. **Keep Codebases Lean**: Inspect binary footprints with `cargo bloat` and analyze APK releases. Respect performance budgets before landing changes.
12. **Real-Device Testing**: Emulators cannot validate USB, audio HAL, or Bluetooth behavior; verify changes on physical hardware.
13. **Calm, Actionable User Copy**: Error messages must guide the user on what action to take (e.g. *"Tap Allow on your phone"* rather than *"ADB authorization error code 3"*).
14. **Anti-AI Slop**: No boilerplate wrapper layers, no speculative factories, and no conversational LLM comments.
15. **Zero Code Churn**: Keep diffs minimal and surgical. Never reformat or restructure working code outside the immediate task scope.
16. **Strict Git Safety**: Agents must never run destructive or history-rewriting Git commands (`git push -f`, `git reset --hard`, `git clean -fd`).
17. **Surgical Staging**: Never propose `git add .` or blind mass-staging. Stage only task-relevant files.
18. **Conventional Commits**: Format commit subjects as `<type>(<scope>): <imperative summary>` under 72 characters without trailing punctuation.

---

## 2. Playbook: Build & Run the Android App

### 2.1 Prerequisites
- Android Studio Iguana or newer, or JDK 17+ with the Android SDK.
- CMake 3.22+ and Android NDK (installed via Android Studio SDK Manager).
- Git submodules initialized (libopus resides in `android/app/src/main/cpp/opus`):
  ```bash
  git submodule update --init --recursive
  ```
- Important: Open the **`android/`** folder directly in Android Studio, not the repository root.

### 2.2 Compilation & Installation
From the `android/` directory:
```bash
# Build debug APK
./gradlew assembleDebug

# Install on a connected physical phone
./gradlew installDebug

# View real-time application logs
adb logcat --pid=$(adb shell pidof com.mikey)
```

### 2.3 Manual Testing over Level 1 (ADB Reverse)
While developing without the full PC tray app running, manually create the reverse tunnel:
```bash
adb reverse tcp:7653 tcp:7653
```
The phone’s connection to `127.0.0.1:7653` now routes directly to port `7653` on the PC.

---

## 3. Playbook: Add a Dependency

Before introducing any new library, crate, or Gradle plugin:

1. **Verify Architectural Fit**: Check whether the library is approved. Prohibit frameworks on the blacklist (Electron, Retrofit, Room, Hilt, Firebase, async on PC).
2. **Platform Native Priority**: Verify whether native OS APIs can achieve the goal (e.g. `SharedPreferences` instead of SQLite; `YuvImage` instead of heavy external image libraries).
3. **Measure Binary Cost**:
   - PC: Run `cargo bloat --release --crates` to verify executable overhead.
   - Android: Inspect the resulting release APK size to guarantee it remains within the ≤ 6 MB per ABI budget.
4. **Android Version Catalogs**: Always declare dependencies inside `android/gradle/libs.versions.toml`, never as hardcoded strings in build scripts.

---

## 4. Playbook: Modify the Wire Protocol

The wire protocol is the binary contract between two independent codebases (Android in Kotlin, PC in Rust):

1. **Specification First**: Update the markdown documentation in [`WIRE_PROTOCOL.md`](./WIRE_PROTOCOL.md) before altering code.
2. **Protocol Versioning**: If a change alters byte framing, packet layouts, or opcode semantics, bump the protocol major version (`PROTO_VERSION = 3`). Version mismatches must return `REJECT (reason: "version")`.
3. **Symmetric Implementation**: Ensure serializers and parsers are updated in lockstep across both `android/app/src/main/java/com/mikey/protocol/` and `pc/src/protocol/`.
4. **Regression Testing**: Execute unit tests on both sides to verify packet encoding:
   ```bash
   cd android && ./gradlew testDebugUnitTest
   cd pc && cargo test
   ```

---

## 5. Playbook: Change an Architectural Decision

When a design assumption proves incorrect or constraints change:

1. **Update Decision Documentation**: Record the rationale, options considered, and selected approach in [`PRODUCT_VISION_AND_SCOPE.md`](./PRODUCT_VISION_AND_SCOPE.md).
2. **Evaluate Cross-Platform Impact**: Determine whether the decision impacts transport priority, audio processing boundaries, or UI layouts.
3. **Update Instructions**: If the change affects agent constraints or coding boundaries, update [`AGENTS.md`](file:///e:/Programs/mikey/AGENTS.md).

---

## 6. Playbook: Update Agent Instructions (`AGENTS.md`)

`AGENTS.md` is the canonical instructions file for all coding agents. `CLAUDE.md` in the repository root references it via `@AGENTS.md`.

### 6.1 Maintenance Invariants
- `AGENTS.md` must point to `docs/` as the single source of truth for architectural and implementation decisions.
- Keep agent instructions concise, clear, and focused on boundaries, read-only Git rules, and anti-AI slop constraints.
- Detailed implementation specs belong in `docs/`, never in root rule files.
- Whenever code changes occur (even minute details like opcodes, buffer capacities, port numbers, or UI text), the corresponding specification in `docs/` must be updated in lockstep.

### 6.2 Pre-Commit Hook Enforcement
To guarantee `CLAUDE.md` and `AGENTS.md` never desynchronize, save the following as `.git/hooks/pre-commit` and run `chmod +x .git/hooks/pre-commit`:
```sh
#!/bin/sh
if ! grep -q "@AGENTS.md" CLAUDE.md 2>/dev/null; then
  echo "Commit blocked: CLAUDE.md must reference AGENTS.md via @AGENTS.md." >&2
  exit 1
fi
```
