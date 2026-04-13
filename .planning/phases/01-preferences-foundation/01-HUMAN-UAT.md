---
status: complete
phase: 01-preferences-foundation
source: [01-VERIFICATION.md]
started: 2026-04-08T00:00:00Z
updated: 2026-04-13T04:22:00Z
---

## Current Test

[testing complete]

## Tests

### 1. T1..T5 Nyquist tests pass under default parallelism
expected: `cargo test --lib preferences::` → `test result: ok. 5 passed; 0 failed`
result: pass
note: Verified by orchestrator during phase execution. All 5 tests green after the test-isolation fix (`215e02a test(preferences): serialize PREFS_CACHE tests to fix parallel-test pollution`). The verifier was unable to reproduce because the worktree is missing the `llama-helper-aarch64-apple-darwin` binary required for full `cargo test`, but the `--lib` subset does not need that binary and completed successfully.

### 2. M1 — Restart persistence
expected: `pnpm run tauri:dev` → switch UI locale to Arabic via settings → kill app fully → relaunch → UI renders in Arabic before any IPC call, with no flash of English
result: pass
note: "Human tested 2026-04-13. Switched to Arabic, quit (Cmd+Q), relaunched via ./clean_run.sh. Rust logs confirmed `preferences hydrated from db: ui_locale=ar`. UI loaded in Arabic immediately with no flash of English. Required a bugfix in setup.rs — first-launch branch was skipping app.manage(AppState), causing a panic before hydrate_from_db could run."

### 3. M3 — Next-recording language honoring
expected: Start recording → stop → switch `transcription_language` via settings (no app restart) → start second recording → `RUST_LOG=app_lib::audio=debug` shows the new language code reaching Whisper
result: pass
note: "Human tested 2026-04-13. Recording 1 with transcription_language=ar: Whisper prompt showed [_LANG_ar], output was Arabic script ('السلام عليكم ورحمة الله وبركاته'). User switched transcription language to English via home page. Recording 2: Whisper prompt showed [_LANG_en], output was English transliteration ('Assalamu alaikum wa rahmatullahi wa barakatuhu'). Language change honored without restart."

## Summary

total: 3
passed: 3
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
