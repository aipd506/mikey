## Summary

<!--
Provide a concise explanation of what this pull request does.
If fixing an issue, link it here: Fixes #123
-->

## Type of Change

<!-- Mark the appropriate item with an [x] -->

- [ ] Bug fix (non-breaking change resolving an issue)
- [ ] New feature (non-breaking addition of functionality)
- [ ] Breaking change (alters wire protocol, CLI flags, or public interfaces)
- [ ] Performance improvement (reduces latency, CPU, RAM, or bandwidth)
- [ ] Refactor (code structure or cleanup without behavioral changes)
- [ ] Documentation (corrections or new guides)
- [ ] Build / Tooling (Gradle, Cargo, CI, or installer scripts)

## Affected Components

<!-- Which parts of Mikey does this change touch? -->

- [ ] Android app (`android/`)
- [ ] PC tray app (`pc/`)
- [ ] Wire protocol contract (`docs/WIRE_PROTOCOL.md`)
- [ ] Documentation (`docs/`)
- [ ] Build / CI workflows

## Proof of Change

<!--
REQUIRED: Provide concrete evidence that your change works and introduces no regressions.
Select the evidence type matching your changes:
-->

- [ ] **UI / Visual changes**: Attached screenshot(s) or screen recording (GIF/MP4/WebM) demonstrating the interface, layout, and visual fidelity.
- [ ] **Audio / Video streaming**: Attached screen recording or device log excerpt verifying connection establishment and stable streaming.
- [ ] **Protocol / Logic / Bug fix**: Attached terminal output of test runs or log trace confirming the bug fix.

<details>
<summary>Visual Evidence (Screenshots / Screen Recordings)</summary>

<!--
Paste, attach, or drag & drop screenshots or screen recordings here.
For UI modifications, please include before/after comparisons if applicable.
-->

</details>

<details>
<summary>Terminal Output & Verification Logs</summary>

```text
<!-- Paste terminal command output, unit test runs, or application log traces here -->
```

</details>

## Verification & Environment

<!-- Detail how you verified your changes -->

### Automated Quality Checks
- [ ] **Android**: `./gradlew testDebugUnitTest lintDebug` passes with zero errors
- [ ] **PC**: `cargo fmt --check` and `cargo clippy -- -D warnings` pass with zero warnings

### Hardware & Manual Test Environment
<!-- Fill in if you tested on real hardware or an emulator -->
- **Phone model**: <!-- e.g., Google Pixel 7, Samsung S23 -->
- **Android version**: <!-- e.g., Android 14 (API 34) -->
- **PC operating system**: <!-- e.g., Windows 11 23H2 / Ubuntu 24.04 -->
- **Connection level tested**: <!-- Level 1 (USB ADB) | Level 2 (USB Tethering) | Level 3 (Wi-Fi) | Level 4 (Bluetooth) -->
- **Observed latency & stability**: <!-- e.g., Audio stable over 30 min, 1080p video at 30 fps without frame drops -->

## Contributor Checklist

- [ ] My code strictly adheres to the project coding standards and performance budgets in [`docs/PERFORMANCE_AND_REALTIME_BUDGETS.md`](../docs/PERFORMANCE_AND_REALTIME_BUDGETS.md).
- [ ] I have read the relevant documentation in [`docs/`](../docs/README.md).
- [ ] Comments are included only where the non-obvious *why* or hardware/OS quirks need explanation.
- [ ] No new dependencies or third-party crates/libraries have been introduced without maintainer approval.
- [ ] All linters, formatters, and unit tests pass locally with zero warnings.
- [ ] Proof of change (screenshots, recordings, or test output) is included above.
