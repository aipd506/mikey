# Coding Rules

These apply from Phase 1 onwards. They are not suggestions.

1. **Build only what the current phase needs.** No abstractions for later phases.
2. **One responsibility per file.** `tcp.rs` does TCP. `beacon.rs` does discovery. Never two concerns in one file.
3. **No dependency without a reason.** Every crate/library must appear in [tech-stack.md](../architecture/tech-stack.md). If it isn't there, propose updating [tech-stack.md](../architecture/tech-stack.md) first.
4. **UI never owns logic.** Compose observes `StateFlow` from the service; the tray & flyout reflect `SessionManager` state.
5. **Real-time paths never block on I/O or locks held by other threads.** Audio capture and audio output use bounded, lock-free or try-lock handoffs; when in doubt, drop old data.
6. **Every socket has a timeout; every queue has a bound.** No unbounded growth anywhere.
7. **Platform code behind `cfg`/interfaces.** Windows and Linux differences live only in `sink.rs`, `vcam.rs`, `bt.rs`, `tray.rs`, `flyout.rs`, `notify.rs`, `autostart.rs`.
8. **Formatting:** `rustfmt` + `clippy -D warnings` (PC), `ktlint` (Android). Run before committing.
9. **English everywhere.** Names, comments, commits.
10. **Comments only where non-obvious.** Explain *why*, not *what*.
11. **Phase 1 can be rough. Phase 2 cleans it. Later phases build on clean.**
12. **`vibe/scratchpad.md` is the trash can.** Never put code in `vibe/`. Useful notes become code comments or edits to the brain (`vibe/`).
13. **Keep it small.** `cargo bloat` before adding a crate; check APK size diff on every dependency change; verify [performance-budgets.md](performance-budgets.md) before closing a phase.
14. **Test on real devices** ([test-matrix.md](test-matrix.md)).
15. **User-facing text is short, calm and actionable.** "Tap Allow on your phone", not "ADB authorization failed (error 3)".
16. **No AI slop.** Write lean, intentional code. No speculative abstractions, boilerplate wrappers, hypothetical utilities, or narrative comments explaining syntax.
17. **Zero code churn.** Keep diffs surgical and minimal. Never rewrite, reformat, or reorder working code or files outside the task scope. Never rewrite whole files when a small edit suffices.
18. **Strict Git safety.** Never run destructive commands (`git push --force`, `git reset --hard`, `git clean -fd`, `git checkout .`, `git restore .`). Never overwrite remote branches or clobber local uncommitted changes. Never commit without asking the user for confirmation first. Inspect `git status` and diff before staging; never blind mass-stage.

