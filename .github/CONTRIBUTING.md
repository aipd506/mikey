# Contributing to Mikey

Thank you for your interest in contributing to Mikey! Whether you're fixing a typo, squashing a bug, or building a new feature - every contribution matters and is deeply appreciated.

> **New to open source?** Welcome! This guide is written with you in mind. Follow the steps and you'll have your first PR merged in no time.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Proof of Change (Required)](#proof-of-change-required)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Architecture Guide](#architecture-guide)
- [Getting Help](#getting-help)

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you agree to uphold a welcoming, inclusive, and harassment-free environment. Please read it before contributing.

---

## How Can I Contribute?

### Report Bugs

Found something broken? [Open a bug report](https://github.com/diveshpatil9104/mikey/issues/new?template=bug_report.yml). Include steps to reproduce, your environment, and any logs or screenshots.

### Suggest Features

Have an idea? [Open a feature request](https://github.com/diveshpatil9104/mikey/issues/new?template=feature_request.yml). Check the [roadmap](../docs/ROADMAP_AND_TEST_MATRIX.md) first to see if it's already planned.

### Improve Documentation

Documentation improvements are always welcome. Fix typos, clarify instructions, add examples - no change is too small.

### Fix Bugs or Implement Features

Browse [open issues](https://github.com/diveshpatil9104/mikey/issues) for something that interests you. Issues labeled **`good first issue`** are ideal for newcomers.

### Test on Your Devices

We need real-device testing across different phones, OS versions, and connection types. See the [test matrix](../docs/ROADMAP_AND_TEST_MATRIX.md) for what we're looking for.

---

## Getting Started

### 1. Fork and clone

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/<your-username>/mikey.git
cd mikey
git submodule update --init    # pulls libopus
```

### 2. Create a branch

```bash
git checkout -b feat/your-feature-name
# or: fix/describe-the-bug
# or: docs/what-you-changed
```

Branch naming convention: `<type>/<short-kebab-description>`

| Type | Use for |
|------|---------|
| `feat/` | New features |
| `fix/` | Bug fixes |
| `docs/` | Documentation changes |
| `refactor/` | Code restructuring (no behavior change) |
| `test/` | Adding or updating tests |
| `chore/` | Build, CI, tooling changes |

---

## Development Setup

### Android

**Requirements:**
- Android Studio (latest stable) or JDK 17+ with Android SDK
- CMake (for libopus native build) - install via SDK Manager → SDK Tools → CMake, or `brew install cmake`
- An Android phone with USB debugging enabled (min SDK 26 / Android 8.0)

```bash
cd android
./gradlew assembleDebug
# APK → android/app/build/outputs/apk/debug/
```

**Install on device:**
```bash
adb install -r android/app/build/outputs/apk/debug/app-debug.apk
```

### PC (Rust)

**Requirements:**
- Rust stable (1.80+) via [rustup](https://rustup.rs/)
- **Windows:** Visual C++ Build Tools (or full Visual Studio)
- **Linux:** `build-essential`, `libasound2-dev`, `libdbus-1-dev`
- **Linux Bluetooth:** BlueZ dev headers (`libbluez-dev` / `bluez-libs-devel`)

```bash
cd pc
cargo build --release
cargo run --release
```

### Virtual audio/video devices

- **Windows:** Install [VB-Audio Virtual Cable](https://vb-audio.com/Cable/) (free) for virtual mic. The [softcam](https://github.com/niconicolet/softcam) DLL is bundled for virtual webcam.
- **Linux:** PipeWire/PulseAudio creates the virtual mic automatically. For virtual webcam, load `v4l2loopback`.

See [INSTALL_PC.md](../docs/INSTALL_PC.md) for detailed platform setup.

---

## Project Structure

```
mikey/
├── android/           # Android app (Kotlin, Jetpack Compose)
│   └── app/
│       └── src/main/
│           ├── java/com/mikey/    # Kotlin sources
│           └── cpp/               # Native code (libopus JNI)
├── pc/                # PC tray app (Rust)
│   └── src/
│       ├── main.rs               # Entry point, tray, event loop
│       ├── transport/            # TCP, Bluetooth, beacon
│       ├── audio/                # DSP pipeline, virtual mic
│       ├── video/                # JPEG decode, virtual camera
│       └── session.rs            # Trust, pairing, session management
├── docs/              # Master documentation, architecture, protocols, and guides
└── .github/           # Community guidelines, issue & PR templates
```

> **Important:** The `docs/` directory is the source of truth for all architectural decisions. Read the relevant `docs/` files before making changes to any component.

---

## Making Changes

### Before you write code

1. **Check existing issues** - someone may already be working on it.
2. **Read the relevant `docs/`** - understand the architecture before modifying it.
3. **Start small** - a focused PR is easier to review and merge.

### While writing code

- Follow the [coding standards](#coding-standards) below.
- Keep changes focused on one thing. Don't mix refactors with feature work.
- Run tests and linters before pushing (see [verification commands](#verification-commands)).

### Verification commands

**Android:**
```bash
cd android
./gradlew testDebugUnitTest lintDebug
```

**PC:**
```bash
cd pc
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

---

## Proof of Change (Required)

**Every PR must include evidence that the change works.** This is non-negotiable - it protects you, the reviewers, and the users.

Include **at least one** of the following in your PR:

| Evidence type | When to use |
|--------------|-------------|
| **Screenshot** (before/after) | UI changes, layout fixes |
| **Screen recording / GIF** | Interactive features, animations, connection flows |
| **Test output** | Paste terminal output of passing tests |
| **Log output** | Relevant log lines showing correct behavior |
| **Build output** | Proof that the project builds successfully |

The PR template has a dedicated section for this. **PRs without proof of change will not be reviewed.**

### Why?

Mikey is a real-time audio/video system. "It compiles" is not enough - we need to see that it actually works on a real device or in a real scenario. This also serves as documentation for future contributors.

---

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <short imperative summary>

[optional body explaining why]
```

### Rules

- **Imperative mood:** `add`, `fix`, `drop`, `stream` (not `added`, `fixing`, `adds`)
- **All lowercase**, concise (≤72 chars), no trailing period
- **Body explains why**, not what

### Examples

```
feat(android): stream mic to pc over tcp

fix(pc): skip unknown frame types in protocol parser

docs: add contributing guide and PR template

refactor(pc): extract jitter buffer into separate module

chore: update rust-toolchain to 1.80
```

### Types

| Type | Purpose |
|------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `refactor` | Code change with no behavior change |
| `perf` | Performance improvement |
| `test` | Adding/updating tests |
| `chore` | Build, CI, tooling |

---

## Pull Request Process

1. **Fill out the PR template completely** - description, type, component, proof of change, testing.
2. **Keep PRs small and focused** - one feature, one fix, or one refactor per PR.
3. **Ensure CI passes** - all checks must be green before review.
4. **Respond to review feedback** - maintainers may request changes. This is normal and constructive.
5. **Squash-merge** - we squash commits on merge for a clean history.

### PR title format

Use the same format as commit messages:

```
feat(android): add bluetooth transport toggle
fix(pc): prevent crash on malformed audio frame
docs: add troubleshooting guide for VB-Cable
```

---

## Coding Standards

### General

- **Build what is asked** - no speculative future-proofing.
- **Minimal diffs** - don't rewrite or reformat code outside the task scope.
- **No chatty comments** - only comment to explain non-obvious *why*, invariants, or OS/hardware quirks.
- **No new dependencies** without maintainer approval (see [add-a-dependency](../docs/DEVELOPER_PLAYBOOKS_AND_SKILLS.md)).

### Kotlin (Android)

- Follow the existing code style (Jetpack Compose conventions).
- UI observes `StateFlow` from `MikeyService` - UI never owns logic.
- Run `./gradlew lintDebug` and fix all warnings.

### Rust (PC)

- `cargo fmt` - all code must be formatted.
- `cargo clippy -- -D warnings` - zero warnings policy.
- No `async`/Tokio outside `pc/src/transport/bt.rs` (Linux Bluetooth only).
- Every socket must have a timeout. Every queue/channel must have a bounded capacity.
- Modules must be ≤175 lines.

### Full rules

The complete set of 18 coding rules lives in [`docs/PERFORMANCE_AND_REALTIME_BUDGETS.md`](../docs/PERFORMANCE_AND_REALTIME_BUDGETS.md). Read it before submitting code changes.

---

## Architecture Guide

Mikey has a detailed architecture documented in `docs/`. Here's what to read based on what you're working on:

| Working on | Read first |
|-----------|-----------|
| Any code change | [SYSTEM_ARCHITECTURE.md](../docs/SYSTEM_ARCHITECTURE.md) |
| Android transport/connection | [TRANSPORTS_AND_NETWORKING.md](../docs/TRANSPORTS_AND_NETWORKING.md) |
| PC audio pipeline | [AUDIO_PIPELINE.md](../docs/AUDIO_PIPELINE.md) |
| PC video pipeline | [VIDEO_PIPELINE.md](../docs/VIDEO_PIPELINE.md) |
| Handshake / pairing | [SESSIONS_AND_TRUST.md](../docs/SESSIONS_AND_TRUST.md) |
| Frame format / protocol | [WIRE_PROTOCOL.md](../docs/WIRE_PROTOCOL.md) |
| Adding a library / playbooks | [DEVELOPER_PLAYBOOKS_AND_SKILLS.md](../docs/DEVELOPER_PLAYBOOKS_AND_SKILLS.md) |
| UI / design | [UI_AND_DESIGN_LANGUAGE.md](../docs/UI_AND_DESIGN_LANGUAGE.md) |

---

## Getting Help

- **Questions?** Open a [Discussion](https://github.com/diveshpatil9104/mikey/discussions).
- **Stuck on setup?** Check [TROUBLESHOOTING.md](../docs/TROUBLESHOOTING.md) or ask in Discussions.
- **Not sure where to start?** Look for issues labeled [`good first issue`](https://github.com/diveshpatil9104/mikey/labels/good%20first%20issue).

---

*Thank you for helping make Mikey better. Every contribution - code, docs, testing, or ideas - makes a difference.*
