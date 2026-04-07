# Testing Patterns

**Analysis Date:** 2026-04-07

## Honest Summary

**Meetily has minimal automated test coverage.** This is typical for Tauri-based desktop apps where audio hardware, OS permissions, and GPU pipelines are difficult to mock. The project relies heavily on **manual integration testing** during development (run the app, record a meeting, observe).

| Layer | Test Files Found | Test Framework |
|-------|------------------|----------------|
| Rust (Tauri backend) | 0 dedicated test files; 27 files contain `#[cfg(test)]`/`#[test]`/`#[tokio::test]` blocks (inline unit tests only) | `cargo test` (Rust built-in) |
| TypeScript (Next.js frontend) | **0** test files (no `*.test.ts`, `*.test.tsx`, `*.spec.ts`, no `__tests__/` directories) | None configured |
| Python (FastAPI backend) | **0** test files (no `test_*.py` or `*_test.py` apart from `backend/debug_cors.py` which is a manual debug script, not a test) | None configured |
| End-to-end | None | Not used |

**There is NO `jest.config.*`, `vitest.config.*`, `pytest.ini`, `conftest.py`, or `pyproject.toml` with test config in the repo.** Adding tests requires also adding configuration.

## Test Framework Status

### Rust — `cargo test` (built-in)
- **Runner:** Rust standard test harness (no external framework)
- **Async tests:** Use `#[tokio::test]` (since Tokio is already a dependency in `frontend/src-tauri/Cargo.toml`)
- **Config:** None — uses Rust defaults
- **Run command:**
  ```bash
  cd frontend/src-tauri
  cargo test                    # Run all tests
  cargo test --lib              # Library tests only
  cargo test <pattern>          # Filter by name
  cargo test -- --nocapture     # Show println! output
  ```

### TypeScript — Not configured
- **Runner:** None. `package.json` has no `test` script.
- `frontend/package.json` scripts include `dev`, `build`, `tauri:dev`, `tauri:build`, `lint` — but **no `test`**.
- To add: install `vitest` (preferred for Vite/Next compatibility) or `jest` + `@testing-library/react`.

### Python — Not configured
- **Runner:** None. No `pytest` in `backend/requirements.txt` (only `fastapi`, `pydantic`, `uvicorn`, `aiosqlite`, `ollama`, etc.).
- To add: append `pytest` and `httpx` to `backend/requirements.txt`, create `backend/tests/`.

## Test File Organization (Rust only)

### Inline test modules (Rust convention)
The codebase follows Rust idiomatic **inline tests** — `#[cfg(test)] mod tests { ... }` blocks at the bottom of each source file rather than a separate `tests/` directory. There is no `frontend/src-tauri/tests/` integration test directory.

**Files containing inline test blocks** (sample):
- `frontend/src-tauri/src/whisper_engine/system_monitor.rs`
- `frontend/src-tauri/src/summary/templates/types.rs`
- `frontend/src-tauri/src/summary/templates/loader.rs`
- `frontend/src-tauri/src/summary/templates/defaults.rs`
- `frontend/src-tauri/src/summary/template_commands.rs`
- `frontend/src-tauri/src/summary/summary_engine/client.rs`
- `frontend/src-tauri/src/ollama/metadata.rs`
- `frontend/src-tauri/src/audio/vad.rs`
- `frontend/src-tauri/src/audio/system_detector.rs`
- `frontend/src-tauri/src/audio/system_audio_stream.rs`
- `frontend/src-tauri/src/audio/system_audio_commands.rs`
- `frontend/src-tauri/src/audio/retranscription.rs`
- `frontend/src-tauri/src/audio/playback_monitor.rs`
- `frontend/src-tauri/src/audio/permissions.rs`
- `frontend/src-tauri/src/audio/incremental_saver.rs`
- `frontend/src-tauri/src/audio/import.rs`
- `frontend/src-tauri/src/audio/hardware_detector.rs`
- `frontend/src-tauri/src/audio/ffmpeg_mixer.rs`
- `frontend/src-tauri/src/audio/diagnostics.rs`
- `frontend/src-tauri/src/audio/devices/fallback.rs`
- `frontend/src-tauri/src/audio/device_monitor.rs`
- `frontend/src-tauri/src/audio/device_detection.rs`
- `frontend/src-tauri/src/audio/decoder.rs`
- `frontend/src-tauri/src/audio/capture/core_audio.rs`
- `frontend/src-tauri/src/audio/capture/backend_config.rs`
- `frontend/src-tauri/src/audio/buffer_pool.rs`

**Note:** Many of these files use `#[cfg(test)]` for test-only helpers or conditional compilation, not necessarily for assertions. Treat the inline `#[cfg(test)] mod tests` block as the canonical place to add new Rust unit tests.

### Naming
- Test functions: `#[test] fn test_<behavior>()` or `#[tokio::test] async fn test_<behavior>()`
- Module: `#[cfg(test)] mod tests { use super::*; ... }`

## Test Structure (Rust idiomatic)

When adding new Rust tests, follow this pattern:

```rust
// At the bottom of any src file, e.g. frontend/src-tauri/src/audio/buffer_pool.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_pool_returns_cleared_buffer() {
        let pool = AudioBufferPool::new(10, 1024);
        let buf = pool.get_buffer();
        assert_eq!(buf.len(), 0);
        assert!(buf.capacity() >= 1024);
    }

    #[tokio::test]
    async fn async_operation_completes() {
        // Use #[tokio::test] for async functions
        let result = some_async_fn().await;
        assert!(result.is_ok());
    }
}
```

## Mocking

### Rust
- **Framework:** None installed (`mockall` is NOT in `Cargo.toml`)
- **Approach:** Use trait abstractions and feature flags. The audio device layer is split into platform modules (`audio/devices/platform/{windows,macos,linux}.rs`) so non-target platforms are excluded at compile time rather than mocked.
- **What to mock:** Hardware (`cpal`), filesystem, network. Currently mostly skipped because there are few unit tests.
- **What NOT to mock:** Pure data transformations (mixing math, VAD threshold logic) — test them directly with sample arrays.

### TypeScript
- N/A — no test framework configured.

### Python
- N/A — no test framework configured.

## Fixtures and Test Data

- **No `tests/fixtures/` directory** in the repo.
- **Sample audio files:** None checked in for testing purposes. Whisper models are downloaded at runtime to `frontend/models/` or `~/Library/Application Support/Meetily/models/`.

## Coverage

- **No coverage tool configured** (no `tarpaulin`, `grcov`, `c8`, `nyc`, or `coverage.py` config).
- To enable Rust coverage: install `cargo-tarpaulin` and run `cargo tarpaulin --out Html`.

## Test Types

### Unit tests
- Sparse Rust inline unit tests in the files listed above (mostly for self-contained modules: VAD, buffer pools, summary template parsing, system monitor)
- Most audio pipeline code is **not unit tested** — relies on manual end-to-end testing

### Integration tests
- **None automated.** Integration is exercised by running `./clean_run.sh` and recording a real meeting.

### End-to-end tests
- **None.** No Playwright, Cypress, WebdriverIO, or `tauri-driver` setup.

### Manual smoke tests
The project relies on these manual workflows during development:
1. `./clean_run.sh debug` — full app startup with debug logging (`frontend/clean_run.sh`)
2. Record a meeting using both microphone and system audio
3. Verify transcript appears in real time
4. Verify recording is saved to `~/Library/Application Support/Meetily/`
5. Verify summary generation via Ollama/Claude/Groq

## CI Configuration

GitHub Actions workflows in `.github/workflows/`:

| Workflow | File | Purpose | Runs Tests? |
|----------|------|---------|-------------|
| Validation Check | `.github/workflows/pr-main-check.yml` | Manual trigger; verifies version format and branch | **No** — version validation only |
| Build (devtest) | `.github/workflows/build-devtest.yml` | Unsigned dev builds | No automated tests |
| Build (Linux) | `.github/workflows/build-linux.yml` | Linux build matrix | No automated tests |
| Build (macOS) | `.github/workflows/build-macos.yml` | macOS build matrix | No automated tests |
| Build (Windows) | `.github/workflows/build-windows.yml` | Windows build matrix | No automated tests |
| Build & Test | `.github/workflows/build-test.yml` | Build verification | Likely build-only — verify locally |
| Build (production) | `.github/workflows/build.yml` | Production builds | No automated tests |
| Release | `.github/workflows/release.yml` | Tagged releases | No automated tests |

**Reality:** CI is build-and-package focused. There is **no `cargo test`, `pnpm test`, or `pytest` step** in any workflow. Cross-platform builds succeeding on macOS/Linux/Windows is the primary "test" gate.

## Common Patterns (When Adding Tests)

### Async testing in Rust
```rust
#[tokio::test]
async fn async_test() {
    let result = my_async_fn().await.unwrap();
    assert_eq!(result, expected);
}
```

### Error testing in Rust
```rust
#[test]
fn returns_error_on_invalid_input() {
    let result = parse_audio_device("");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MyError::EmptyInput));
}
```

### Testing audio data
```rust
#[test]
fn vad_detects_speech() {
    let mut vad = ContinuousVadProcessor::new(48000, 200).unwrap();
    let speech_samples: Vec<f32> = generate_test_tone(48000, 1.0); // 1 sec
    let segments = vad.process(&speech_samples);
    assert!(!segments.is_empty());
}
```

## Recommendations for Test Coverage Growth

If adding tests becomes a priority:

1. **Rust pure logic first** — `audio/buffer_pool.rs`, `audio/vad.rs` (VAD threshold logic), `summary/templates/loader.rs`, mixing math in `audio/pipeline.rs`. These are pure functions, no hardware needed.
2. **Python API tests** — Add `pytest` + `httpx.AsyncClient` to test FastAPI endpoints in `backend/app/main.py` against an in-memory SQLite.
3. **Frontend services** — Add `vitest` and unit-test the service classes in `frontend/src/services/` by mocking `@tauri-apps/api/core` `invoke`.
4. **E2E** — `tauri-driver` exists but requires significant setup; defer until 2 and 3 are in place.

---

*Testing analysis: 2026-04-07*
