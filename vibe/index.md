# Mikey — Brain Index

> **Plug in. Tap once. Forget it exists.**
> Turn your Android phone into a mic and webcam for your PC — over USB, Bluetooth or Wi‑Fi — with no accounts, no ads, and no setup ritual.

The `vibe/` directory is the single source of truth for the Mikey project. It contains the product philosophy, technical architecture, wire protocols, UI/UX specs, coding standards, and operational playbooks.

---

## Current Status

- **Phase:** 2 (Android) / Phases 1–4 complete (PC) — Audio, Video, DSP, Flyout UI, and Installers locked in (Checklist: [core/roadmap.md](core/roadmap.md)).
- **Android (`android/`):** Phase 1 complete (TCP mic streaming over USB debugging, `MikeyService` foreground service). Builds with `./gradlew assembleDebug`.
- **PC (`pc/`):** Complete (Phases 1–4). Multi-transport tray app (`mikey`). TCP :7653, UDP beacon :7654, BT RFCOMM, SessionManager, Opus/PCM decoding, adaptive jitter buffer, drift resampler, RNNoise noise suppression, SpeexDSP AEC loopback cancellation, camera pipeline (JPEG decode, letterboxing, vcam, pop-out preview), Win32 GDI flyout companion app, and Inno Setup / Linux packaging installers. All modules <= 175 lines.
- **Next immediate step:** Phase 2 — Android Phase 2 (full UI per [phone-ux.md](core/phone-ux.md), `TransportManager`, Opus capture, pairing tokens) and completing PC Linux support.

---

## Directory Hierarchy & Descriptions

```text
vibe/
├── index.md               # This file: master index, directory map & current status
├── core/                  # Product vision, principles, UX specs, roadmap & decisions
│   ├── vision.md          # Target user, core promise, elevator pitch
│   ├── principles.md      # The 7 product principles governing all decisions
│   ├── feasibility.md     # Feasibility validation, corrections & technical proof
│   ├── scope.md           # Explicit in-scope and out-of-scope feature boundaries
│   ├── user-journeys.md   # User flows: first-run, everyday connection, mid-call upgrade
│   ├── phone-ux.md        # Android interface: split halves, gestures, drawer, notifications
│   ├── pc-ux.md           # PC interface: tray icon, companion flyout app, join prompts, preview window
│   ├── roadmap.md         # The 5 phases with checklists and completion criteria
│   ├── risks.md           # Risk register and concrete mitigations
│   ├── open-decisions.md  # Recorded architectural choices & rationale
│   └── changelog.md       # Evolution history from PRD v0
│
├── architecture/          # Engineering architecture, protocols & pipelines
│   ├── architecture.md    # High-level topology, thread model & component boundaries
│   ├── connection-levels.md # The 4 transport levels (USB Debug, Tether, BT, Wi-Fi)
│   ├── sessions-trust.md  # Pairing tokens, trust rules & session lifecycle
│   ├── wire-protocol.md   # Framing, packet types, handshake & media header
│   ├── media-pipeline.md  # Audio (Opus/PCM), video (JPEG), jitter buffer, DSP & virtual devices
│   ├── tech-stack.md      # Approved libraries, system permissions & forbidden dependencies
│   ├── security-privacy.md # Local security, threat model & socket boundaries
│   └── repo-structure.md  # Planned codebase file tree & standards
│
├── rules/                 # Invariants, quality budgets & standards
│   ├── design-language.md # Palette, typography, icon metrics & UI rules
│   ├── performance-budgets.md # CPU, RAM, battery, binary size & latency budgets
│   ├── test-matrix.md     # Real-device test suite across phones & host OSes
│   ├── coding-rules.md    # The 18 non-negotiable coding rules
│   └── agent-files.md     # Maintenance rules for AGENTS.md and CLAUDE.md
│
└── skills/                # Step-by-step operational playbooks
    ├── add-a-dependency.md# Vetting, approving and measuring new dependencies
    ├── build-and-run-android.md # Android build, install, and ADB reverse testing
    ├── change-a-decision.md # Process for revisiting architectural decisions
    ├── change-the-protocol.md # Making wire protocol modifications safely
    ├── close-a-phase.md   # Checklist & budget sign-offs before advancing phases
    └── update-agent-files.md # Synchronizing AGENTS.md and CLAUDE.md
```

---

## How to Use

1. Read this index before writing any code.
2. Open only the specific section files your task requires.
3. Recurring task? Follow its dedicated **skill** playbook.

---

## Index of Sections

### Core — Product & UX

| File | What's in it | Read when |
|---|---|---|
| [vision.md](core/vision.md) | Motto, vision, who we build for, the promise | Any product or UX question |
| [principles.md](core/principles.md) | The 7 product principles | Deciding if a feature belongs, and where |
| [feasibility.md](core/feasibility.md) | What's possible, viability, corrections to PRD v0 | Questioning an architecture choice |
| [scope.md](core/scope.md) | In and out of scope for v1.0 | Before adding anything new |
| [user-journeys.md](core/user-journeys.md) | First setup, first connection per level, daily use, mid-call upgrade | Designing a flow |
| [phone-ux.md](core/phone-ux.md) | Phone layout, states, settings drawer, rotation, notification & lifecycle, saved settings | Any Android UI or service work |
| [pc-ux.md](core/pc-ux.md) | Tray icon & anchored flyout companion app, ask-before-join, preview, PC notifications, config file | Any PC UI work |
| [roadmap.md](core/roadmap.md) | Phases 1–5 with checklists | Picking the next task |
| [risks.md](core/risks.md) | Risk register | Planning a phase |
| [open-decisions.md](core/open-decisions.md) | Decisions the owner may revisit | Before changing a decision |
| [changelog.md](core/changelog.md) | What changed from PRD v0 | History |

### Architecture — System Design

| File | What's in it | Read when |
|---|---|---|
| [architecture.md](architecture/architecture.md) | System diagram, key decisions, PC threads | Starting any module |
| [connection-levels.md](architecture/connection-levels.md) | L1 USB debugging, L2 USB tethering, L3 Bluetooth, L4 Wi‑Fi, probing, handover, discovery beacon | Transport work |
| [sessions-trust.md](architecture/sessions-trust.md) | Identity, sessions, trust rules | Handshake and pairing |
| [wire-protocol.md](architecture/wire-protocol.md) | Framing, frame types, media header, handshake, liveness | Anything on the wire — phone and PC must match exactly |
| [media-pipeline.md](architecture/media-pipeline.md) | Audio and video capture/processing, virtual devices | Audio or video work |
| [tech-stack.md](architecture/tech-stack.md) | Libraries per platform, Android permissions, what's excluded | Before adding any dependency |
| [security-privacy.md](architecture/security-privacy.md) | Local only, trust, input hardening, encryption | Any security-relevant change |
| [repo-structure.md](architecture/repo-structure.md) | Full planned file tree, `.gitignore` | Creating files or folders |

### Rules — Invariants & Quality

| File | What's in it | Read when |
|---|---|---|
| [design-language.md](rules/design-language.md) | Colors, typography, icon sizes | Drawing anything |
| [performance-budgets.md](rules/performance-budgets.md) | Size, CPU, RAM, battery, latency budgets | Adding weight; closing a phase |
| [test-matrix.md](rules/test-matrix.md) | Real-device test cases | Closing a phase |
| [coding-rules.md](rules/coding-rules.md) | The 18 coding rules | Always |
| [agent-files.md](rules/agent-files.md) | What AGENTS.md / CLAUDE.md are and how they're kept | Editing either file |

### Skills — Workflows & Playbooks

| Skill | Use when |
|---|---|
| [build-and-run-android](skills/build-and-run-android.md) | Building, installing or debugging the phone app |
| [add-a-dependency](skills/add-a-dependency.md) | About to add any library, plugin or crate |
| [change-the-protocol](skills/change-the-protocol.md) | Touching frames, handshake, beacon or ports |
| [change-a-decision](skills/change-a-decision.md) | Part of the plan turns out wrong |
| [update-agent-files](skills/update-agent-files.md) | Editing AGENTS.md or CLAUDE.md |
| [close-a-phase](skills/close-a-phase.md) | The current phase looks done |

---

## By Side

- **Android** (`android/`): [phone-ux.md](core/phone-ux.md), [design-language.md](rules/design-language.md), [connection-levels.md](architecture/connection-levels.md) (phone side), [sessions-trust.md](architecture/sessions-trust.md), [wire-protocol.md](architecture/wire-protocol.md), [media-pipeline.md](architecture/media-pipeline.md) (phone parts), [tech-stack.md](architecture/tech-stack.md), [roadmap.md](core/roadmap.md) (Android checklists); skill *build-and-run-android*
- **PC** (`pc/`): [pc-ux.md](core/pc-ux.md), [architecture.md](architecture/architecture.md), [connection-levels.md](architecture/connection-levels.md) (PC side), [sessions-trust.md](architecture/sessions-trust.md), [wire-protocol.md](architecture/wire-protocol.md), [media-pipeline.md](architecture/media-pipeline.md) (PC parts), [tech-stack.md](architecture/tech-stack.md), [roadmap.md](core/roadmap.md) (PC checklists)

---

## Key Facts

- Android: Kotlin + Jetpack Compose, package `com.mikey`, min SDK 26.
- PC: Rust tray app, binary `mikey`, Windows 10/11 + Linux. TCP `7653`, UDP beacon `7654`.
- Connection levels, best first: 1 USB debugging › 2 USB tethering › 3 Bluetooth (audio only) › 4 Wi‑Fi.
- The phone is always the client; the PC is always the server.
- Mic and camera always start off. All audio processing runs on the PC.
