# Codebase Concerns

**Analysis Date:** 2026-04-07
**Active branch:** `fix/audio-mixing` (audio pipeline rework in progress)

## Marker Inventory (TODO/FIXME/HACK/XXX/"for now")

| Marker | Frontend Rust | Frontend TS/TSX | Backend Python | Whisper vendored |
|--------|---------------|-----------------|----------------|------------------|
| TODO/FIXME/HACK/XXX | ~22 (excl. `lib_old_complex.rs`) | 2 | 0 in `backend/app` | 13 in `backend/whisper-custom` (vendored upstream) |
| "for now" / "temporary" / "hardcoded" | ~30+ | n/a | n/a | n/a |

The bulk of stub markers cluster in two places: `frontend/src-tauri/src/audio_v2/` (an unfinished parallel rewrite) and `frontend/src-tauri/src/lib_old_complex.rs` (a 2,437-line legacy file that still ships in-tree).

---

## Critical

### 1. Dead/half-finished `audio_v2` module shipped alongside production audio
- Files: `frontend/src-tauri/src/audio_v2/lib.rs`, `audio_v2/recorder.rs`, `audio_v2/resampler.rs`, `audio_v2/limiter.rs`, `audio_v2/normalizer.rs`, `audio_v2/sync.rs`, `audio_v2/compatibility.rs`, `audio_v2/mixer.rs`
- Issue: Multiple methods are stubs (`// TODO: Implement initialization`, `// TODO: Implement recording start/stop`, `// TODO: Implement in Phase 3/4`, `// For now, return simple normalization`, `// For now, return simple clipping`, `// For now, use a simple weighted average`). The `compatibility.rs` shim picks between legacy and modern code paths but explicitly notes "use the legacy sender for now" and "Implement quality metrics (TODO)".
- Impact: Two parallel audio stacks coexist. Any change to the legacy `audio/` module risks divergence from `audio_v2/`. If `audio_v2` is ever toggled on accidentally, recording will silently produce degraded output (no real resampler, no limiter, no sync).
- Fix approach: Either (a) gate `audio_v2` behind a Cargo feature so it cannot be linked in release, (b) finish Phases 3–4, or (c) delete it entirely and document the rewrite as cancelled.

### 2. Legacy `lib_old_complex.rs` (2,437 lines) still in source tree
- File: `frontend/src-tauri/src/lib_old_complex.rs`
- Issue: 2,437 lines vs `lib.rs` 747 lines. Contains its own TODO ("Calling discover_models as workaround for updating the available_models") and "for now" markers. Confuses search results, increases compile time, and risks accidental import.
- Impact: Maintenance noise; reviewers cannot tell which `lib*.rs` is canonical. New contributors may modify the wrong file.
- Fix approach: Verify `lib.rs` is the only registered entry, then delete `lib_old_complex.rs` (preserve in git history).

### 3. CORS wildcard in production-shaped server
- File: `backend/app/main.py:43-46`
- Code: `allow_origins=["*"]` with comment "Allow all origins for testing"
- Impact: Any website the user visits while the backend is running on `localhost:5167` can issue authenticated CRUD requests against meetings, summaries, **and saved LLM API keys** (`/api-key`, `/transcript-api-key`). High-severity exfiltration risk.
- Fix approach: Restrict to `tauri://localhost`, `http://localhost:3118`, and the specific Tauri origin. Add `allow_credentials=False` if wildcard ever returns. Pull origins from env var.

### 4. LLM/Transcript API keys persisted in SQLite without encryption
- Files: `backend/app/db.py:582` (`save_api_key`), `backend/app/main.py:555-599` (`/api-key`, `/transcript-api-key`, `get_api_key`, `get_transcript_api_key`), `backend/app/transcript_processor.py:114-142` (Anthropic / Groq / OpenAI)
- Issue: Anthropic, Groq, OpenAI, OpenRouter, and transcript-provider keys are saved to a local SQLite file via plain inserts. No mention of OS keyring, no field-level encryption.
- Impact: Combined with concern #3, any malicious page can retrieve every saved third-party API key via a single fetch. Even without CORS, anyone with read access to the SQLite file (backups, sync folders, mis-configured Docker volume) gets every key.
- Fix approach: Use `tauri-plugin-stronghold` / OS keychain (macOS Keychain, Windows Credential Manager, libsecret) or at minimum encrypt-at-rest with a key derived from a user passphrase.

### 5. Hardcoded server URL coupling frontend to specific backend host
- Files: `frontend/src-tauri/src/api/api.rs:19,232,234`
- Code/Comments: "Hardcoded server URL", "Helper function to get server address - now hardcoded", `log_info!("Using hardcoded server URL: {}", APP_SERVER_URL);`
- Impact: Cannot deploy backend on a non-default port, cannot run frontend against a remote/staging backend, breaks Docker port-mapping flexibility.
- Fix approach: Read from env var or Tauri config, persist user override in `recording_preferences`.

---

## High

### 6. Audio pipeline panics on VAD init failure
- File: `frontend/src-tauri/src/audio/pipeline.rs:736`
- Code: `panic!("VAD processor creation failed: {}", e);`
- Impact: A transient model-load issue or missing ONNX runtime crashes the entire Tauri process mid-recording. No graceful fallback to non-VAD passthrough.
- Fix approach: Return `Result`, surface error to UI, fall back to "all audio is speech" mode so recording still saves.

### 7. Whisper one-shot model loading + recent unload regressions
- Files: `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`
- Recent fix history (red flag — three consecutive fixes in the last few weeks):
  - `544acaa fix: unload transcription engine after batch jobs to free memory`
  - `edf71c0 ... fix/model-unload-fixes`
  - `4b796ad fix: correct VAD timestamp calculations ambiguity and model key parsing`
- Issue: CLAUDE.md confirms "Whisper model loading is one-shot; switching models requires app restart." Combined with active fixes, the load/unload lifecycle is fragile.
- Impact: Memory leaks if unload misses a path, stale model after user switches models, potential double-load OOM on small machines.
- Fix approach: Centralize load/unload behind a single state machine with explicit transitions; add a leak test that loads/unloads N times and asserts RSS bound.

### 8. Backend uses a singleton-style `DatabaseManager()` per-request
- Files: `backend/app/main.py` (multiple endpoints construct `db = DatabaseManager()` ad-hoc), `backend/app/db.py`
- Impact: Each request opens a new aiosqlite connection. Under load, this thrashes file locks and can produce `database is locked` errors. Also bypasses any future connection-pool optimization.
- Fix approach: Use FastAPI dependency injection with a process-wide pool, or a single `DatabaseManager` instance attached to `app.state`.

### 9. Audio device modules shipped as stubs after modularization
- Files: `frontend/src-tauri/src/audio/capture/microphone.rs:2,4` ("TODO: Extract microphone AudioStream logic from core.rs", "Placeholder for now - will be implemented in later phase")
- Issue: The modularization documented in CLAUDE.md split `core.rs` into focused modules, but at least one capture module is still a placeholder. The real logic likely lives elsewhere or in `lib_old_complex.rs`.
- Impact: Confusing source layout; the module name implies functionality that isn't there. Future contributors will edit the wrong file.
- Fix approach: Either complete the extraction or delete the placeholder file and document where the logic actually lives.

### 10. Hardcoded Silero VAD sample-rate assumption
- File: `frontend/src-tauri/src/audio/vad.rs:33` ("Silero VAD MUST use 16kHz - this is hardcoded requirement")
- Combined with: pipeline assumes 48kHz capture (per CLAUDE.md "Pipeline expects consistent 48kHz sample rate")
- Impact: A capture device that cannot do 48kHz, or a future model needing a different SR, breaks the resample bridge silently. The recent commit `6b68bcc fix VAD sample range crash` shows this area has bitten users.
- Fix approach: Make the resample factor configurable; assert capture SR at startup with an actionable error.

### 11. Platform-specific audio capture has known quirks per CLAUDE.md
- Files: `frontend/src-tauri/src/audio/devices/platform/macos.rs`, `windows.rs`, `linux.rs`, `frontend/src-tauri/src/audio/system_audio_stream.rs:204`
- macOS: ScreenCaptureKit requires macOS 13+; needs both microphone AND screen-recording permissions; relies on BlackHole virtual device for some flows. `system_audio_stream.rs:204` notes "always use enhanced capture on macOS" with no fallback path.
- Windows: WASAPI exclusive-mode conflicts with other apps; loopback path is the only system-audio source.
- Linux: ALSA/PulseAudio coverage is the least exercised of the three.
- Impact: Per-platform regressions are easy to introduce. macOS users on <13 silently fail; Windows users with exclusive-mode apps lose system audio.
- Fix approach: Add a platform capability probe at startup, surface "system audio unavailable because X" to the UI, and add CI smoke tests on each platform.

### 12. Audio mixing synchronization is fragile by design
- File: `frontend/src-tauri/src/audio/pipeline.rs` (also `audio/recording_manager.rs`)
- Issue: The ring buffer must align mic and system streams that arrive at different rates. Branch name `fix/audio-mixing` confirms this is the active firefight.
- Impact: Clock drift, dropped chunks, RMS-ducking miscalibration → audible artifacts in saved recordings.
- Fix approach: Add explicit drift telemetry (already partial via `AudioMetricsBatcher`), unit-test the ring buffer with synthetic out-of-phase streams, document the invariant ("both streams must converge within N ms").

---

## Medium

### 13. Fake/test code paths in shipping binary
- Files:
  - `frontend/src-tauri/src/audio/simple_level_monitor.rs:41` ("create fake level data to test the UI")
  - `frontend/src-tauri/src/summary/template_commands.rs:135,155` ("just verify the function compiles and runs", "test the validation logic directly")
- Impact: Synthetic data may leak into production UI; users see fake meter levels.
- Fix approach: Gate behind `#[cfg(test)]` or feature flag.

### 14. Noise estimation comment admits broken algorithm
- File: `frontend/src-tauri/src/audio/audio_processing.rs:464`
- Comment: "for now, we will just assume the noise is constant (kinda defeats the purpose)"
- Impact: Noise suppression is effectively a no-op; loud transients still hit the encoder.
- Fix approach: Implement minimum-statistics noise estimation or remove the misleading API.

### 15. STT loop swallows iteration errors
- File: `frontend/src-tauri/src/audio/stt.rs:291` ("we'll continue to the next iteration"), `audio/stt.rs:128` ("TODO --optimize")
- Impact: Errors are silently dropped, making transcription failures invisible to the user.
- Fix approach: Emit a Tauri event on each error so the UI can show "transcription chunk failed" toasts; add a counter for the diagnostics panel.

### 16. Recording preferences cannot prompt for save location
- File: `frontend/src-tauri/src/audio/recording_preferences.rs:251` ("would need to be implemented with tauri-plugin-dialog")
- Impact: Users cannot choose where recordings are saved; everything goes to the default Downloads dir.
- Fix approach: Add `tauri-plugin-dialog`, wire `pickFolder()` into the preferences UI.

### 17. System audio command leaks the stream handle
- File: `frontend/src-tauri/src/audio/system_audio_commands.rs:17` ("TODO: Store the stream in global state if needed for management")
- Impact: Once started, the stream cannot be stopped or restarted from the command layer; relies on Drop. A second start call could leak the first stream.
- Fix approach: Store stream in app state, expose `stop_system_audio` command.

### 18. Whisper system monitor temperature reading disabled
- File: `frontend/src-tauri/src/whisper_engine/system_monitor.rs:111,113` ("disable temperature monitoring to avoid API compatibility issues", "TODO: Implement platform-specific temperature reading if needed")
- Impact: Cannot detect thermal throttling on long meetings; users may see degraded transcription speed without explanation.
- Fix approach: Use `sysinfo::Components` or platform-specific calls (IOKit on macOS, OpenHardwareMonitor on Windows).

### 19. Confidence scoring not surfaced
- File: `frontend/src-tauri/src/whisper_engine/parallel_processor.rs:363` ("TODO: Add confidence scoring if available")
- Impact: UI cannot highlight low-confidence segments; users must manually review the entire transcript.
- Fix approach: Pass through whisper.cpp `t_token_logprobs` and surface as a 0–1 score on each segment.

### 20. Notification clearing is a stub
- File: `frontend/src-tauri/src/notifications/system.rs:95` ("we'll just log that we attempted to clear")
- Impact: Stale meeting notifications accumulate in OS notification center.
- Fix approach: Use `tauri-plugin-notification` clear API.

### 21. UI find-in-summary missing
- File: `frontend/src/components/MeetingDetails/SummaryPanel.tsx:132` ("TODO: Implement find in summary functionality")
- Impact: Long summaries (1+ hour meetings) have no search affordance.
- Fix approach: Add a basic `<input>` + highlight pass over the rendered markdown.

### 22. Device monitoring placeholder in UI
- File: `frontend/src/components/DeviceSelection.tsx:233` ("{/* TODO: Monitoring */}")
- Impact: Users cannot see real-time device level meters in the device picker; harder to verify the right mic is selected.
- Fix approach: Wire `simple_level_monitor` (after concern #13 is fixed) into this slot.

### 23. Frontend OpenAI/Anthropic/Groq fallback model lists are hardcoded
- Files: `frontend/src-tauri/src/openai/openai.rs:40`, `anthropic/anthropic.rs:40`, `groq/groq.rs:40`
- Impact: When the provider's `/models` endpoint fails, users see a stale list missing recent models (e.g., new Claude / GPT releases).
- Fix approach: Cache last successful response in app data dir; fall back to cache before falling back to hardcoded list.

### 24. Recording manager comment hints at unfinished refactor
- File: `frontend/src-tauri/src/audio/recording_manager.rs:55` ("Remove app handle storage for now - will be passed directly when saving")
- Impact: Threading the AppHandle through every save call increases coupling and makes refactoring harder.
- Fix approach: Re-introduce `Arc<AppHandle>` storage or accept the trade-off and document why.

---

## Low

### 25. Backend logs may emit secrets at INFO level
- Files: `backend/app/main.py` (the model-config endpoints log on every call; concern #4 puts api keys in `model_config`)
- Impact: API keys could end up in stdout / log files via verbose logging.
- Fix approach: Add a redaction filter on `apiKey`/`api_key` keys before logging.

### 26. Env files present in repo working tree
- Files (existence only, contents not read): `.env*` patterns may exist in `backend/` or `frontend/` based on CLAUDE.md docker workflow references.
- Impact: Secrets potentially staged accidentally if `.gitignore` is incomplete.
- Fix approach: Audit `.gitignore` to ensure `.env*` is excluded at every level; add a pre-commit secret scanner.

### 27. Vendored `whisper-custom/server/server.cpp` has multiple TODOs
- File: `backend/whisper-custom/server/server.cpp:90,716,973,982,1092` (TODOs incl. "this is a hack, remove when the mutex is removed")
- Impact: Local fork of upstream whisper-server diverges from upstream; missed bugfixes upstream.
- Fix approach: Track upstream whisper.cpp version, plan periodic rebases; OR justify and document the fork divergence.

### 28. Vendored `httplib.h` has Brotli + FD_SETSIZE TODOs
- File: `backend/whisper-custom/server/httplib.h` (multiple lines)
- Impact: Vendored dependency, low risk but should be tracked.
- Fix approach: Pin a release version of cpp-httplib and document the version.

### 29. Symphonia decoder relies on temp WAV trampoline for some formats
- File: `frontend/src-tauri/src/audio/decoder.rs:274,297,302,405`
- Impact: Each import writes a temp WAV via ffmpeg, doubling disk I/O for large files. Temp file is auto-deleted but consumes disk space mid-import.
- Fix approach: Stream-decode with Symphonia where possible; reserve ffmpeg fallback only for codecs Symphonia doesn't support.

### 30. Unit tests inside command modules pollute production binary
- Files: `frontend/src-tauri/src/summary/summary_engine/client.rs:326,339` (`panic!("Wrong response type")`)
- Impact: Test panics share namespace with production code. If `cfg(test)` gating is missed, panics could trip in release.
- Fix approach: Move tests into `#[cfg(test)] mod tests {}` blocks or `tests/` integration directory.

---

## Recently Fixed (Watch For Regressions)

From `git log` on `main`/`fix/*` branches in the last month:
- `544acaa fix: unload transcription engine after batch jobs to free memory` — memory leak class
- `edf71c0 fix/model-unload-fixes` — same area
- `4b796ad fix: correct VAD timestamp calculations ambiguity and model key parsing` — VAD math
- `6b68bcc fix VAD sample range crash` — VAD bounds
- `c724319 fix: address PR #358 review — fix memory leaks, race conditions, and data safety` — broad audio import safety
- `ca12d11 fix: ... build breakers, data safety, race conditions` — audio import
- `0fbeb6e fix: replace unstable floor_char_boundary with stable equivalent` — Rust nightly drift
- `0bcd2fa fix(language): desync of language preference on startup` — preferences

**Pattern:** Memory lifecycle (unload), VAD math, and audio import dominate recent fixes. Any new work in these areas should ship with regression tests.

---

## Test Coverage Gaps (high-value targets)

- **Audio mixing ring buffer**: No unit tests cover out-of-phase mic/system input. File: `frontend/src-tauri/src/audio/pipeline.rs`.
- **Whisper load/unload lifecycle**: No leak test asserts RSS bound after N cycles. File: `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`.
- **API key endpoints**: No auth test, no CORS rejection test. File: `backend/app/main.py`.
- **Audio device platform code**: Each `devices/platform/{macos,windows,linux}.rs` lacks a stub test that at least exercises enumeration on the host CI runner.
- **Symphonia + ffmpeg decoder fallback**: `frontend/src-tauri/src/audio/decoder.rs` round-trip tests for MKV/WebM/WMA additions (recent feature `6b68bcc`) should be added.

---

*Concerns audit: 2026-04-07 — focus area `concerns`, branch `fix/audio-mixing`*
