---
status: partial
phase: 01-preferences-foundation
source: [01-VERIFICATION.md]
started: 2026-04-08T00:00:00Z
updated: 2026-04-08T00:00:00Z
---

## Current Test

[awaiting human testing — items 2 and 3]

## Tests

### 1. T1..T5 Nyquist tests pass under default parallelism
expected: `cargo test --lib preferences::` → `test result: ok. 5 passed; 0 failed`
result: passed
note: Verified by orchestrator during phase execution. All 5 tests green after the test-isolation fix (`215e02a test(preferences): serialize PREFS_CACHE tests to fix parallel-test pollution`). The verifier was unable to reproduce because the worktree is missing the `llama-helper-aarch64-apple-darwin` binary required for full `cargo test`, but the `--lib` subset does not need that binary and completed successfully.

### 2. M1 — Restart persistence
expected: `pnpm run tauri:dev` → switch UI locale to Arabic via settings → kill app fully → relaunch → UI renders in Arabic before any IPC call, with no flash of English
result: pending
why_human: Requires Tauri app lifecycle. `lib.rs::run` hydration path cannot be exercised from `cargo test`. This is the end-to-end proof that `hydrate_from_db` + SQLite persistence work together across a cold process start.

### 3. M3 — Next-recording language honoring
expected: Start recording → stop → switch `transcription_language` via settings (no app restart) → start second recording → `RUST_LOG=app_lib::audio=debug` shows the new language code reaching Whisper
result: pending
why_human: Requires a live recording flow end-to-end. Validates that the four migrated call sites at `preferences::read().transcription_language` return the updated value after a `set_user_preferences` call.

## Summary

total: 3
passed: 1
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
