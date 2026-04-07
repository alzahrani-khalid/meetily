# Coding Conventions

**Analysis Date:** 2026-04-07

Meetily is a polyglot project with three primary languages: **Rust** (Tauri backend, audio pipeline, Whisper integration), **TypeScript/React** (Next.js frontend), and **Python** (FastAPI backend). Conventions differ per language; the rules below are prescriptive for each.

## Language Style Tooling

| Language | Tool | Config File | Status |
|----------|------|-------------|--------|
| Rust | `rustfmt` (default profile) | None checked in | Use defaults from `cargo fmt` |
| Rust | `clippy` | None checked in | Run via `cargo clippy` |
| TypeScript | ESLint via Next.js | `frontend/eslint.config.mjs` | Extends `next/core-web-vitals`, `next/typescript` |
| TypeScript | Prettier | Not present | No project-wide formatter; rely on editor defaults + ESLint |
| Python | None configured | No `pyproject.toml`, `.flake8`, `ruff.toml`, or `black` config | Code is pinned-version `requirements.txt` only |

**Key takeaway:** Only the frontend has enforced linting (ESLint via `next lint`). Rust and Python rely on developer discipline. There is no pre-commit hook enforcing format.

### Frontend ESLint config (`frontend/eslint.config.mjs`)
```js
const eslintConfig = [
  ...compat.extends("next/core-web-vitals", "next/typescript"),
];
```

### TypeScript compiler (`frontend/tsconfig.json`)
- `"strict": true` — strict mode is on
- `"target": "ES2017"`, `"moduleResolution": "bundler"`
- Path alias: `"@/*": ["./src/*"]`
- `src-tauri` is excluded from TS compilation

## Naming Patterns

### Rust (`frontend/src-tauri/src/`)
- **Files / modules:** `snake_case` — e.g. `recording_manager.rs`, `buffer_pool.rs`, `system_audio_stream.rs`
- **Functions:** `snake_case` — `start_recording`, `list_audio_devices`, `get_buffer`
- **Types / structs / enums:** `PascalCase` — `AudioBufferPool`, `RecordingState`, `TranscriptionStatus`, `SpeechSegment`
- **Constants:** `SCREAMING_SNAKE_CASE` — `RECORDING_FLAG`, `LANGUAGE_PREFERENCE`, `VAD_SAMPLE_RATE`
- **Modules:** `snake_case` directory or file names — `audio/devices/microphone.rs`
- **Domain naming convention:** Audio devices are always `microphone` and `system` (NEVER `input`/`output`). Enforced consistently across `audio/devices/microphone.rs`, `audio/devices/speakers.rs`, `audio/capture/microphone.rs`, `audio/capture/system.rs`.

### TypeScript / React (`frontend/src/`)
- **Component files:** `PascalCase.tsx` — `SidebarProvider.tsx`
- **Service / utility files:** `camelCase.ts` — `recordingService.ts`, `transcriptService.ts`, `indexedDBService.ts`
- **Components:** `PascalCase` — `SidebarProvider`, `RecordingService` (class)
- **Functions / methods / variables:** `camelCase` — `isRecording`, `getRecordingState`, `startRecording`
- **Interfaces / types:** `PascalCase` — `RecordingState`, `RecordingStoppedPayload`
- **Hooks:** `use` prefix — `useSidebar` (in `frontend/src/components/Sidebar/SidebarProvider.tsx`)

### Python (`backend/app/`)
- **Files:** `snake_case.py` — `main.py`, `db.py`, `transcript_processor.py`
- **Functions:** `snake_case`
- **Pydantic models / classes:** `PascalCase` — `Transcript`, `MeetingResponse`, `MeetingDetailsResponse`, `SaveTranscriptRequest`, `DatabaseManager`
- **Module-level constants:** `SCREAMING_SNAKE_CASE`

## Code Style

### Rust formatting
- Default `rustfmt` style (4-space indent, 100-column soft limit)
- Imports grouped by std → external crate → local
- Use `///` doc comments for public APIs (see `frontend/src-tauri/src/audio/buffer_pool.rs` and `frontend/src-tauri/src/audio/vad.rs`)

### TypeScript formatting
- 2-space indentation throughout (`frontend/src/services/recordingService.ts`)
- Single quotes for imports/strings inside services; double quotes acceptable in JSX
- Trailing semicolons required
- JSDoc-style comments on exported service methods:
  ```ts
  /**
   * Recording Service
   * Singleton service for managing recording lifecycle operations
   */
  export class RecordingService { ... }
  ```

### Python formatting
- 4-space indentation
- Line length: not enforced (no formatter config)
- Type hints required on Pydantic models and most function signatures (see `backend/app/main.py`)

## Import Organization

### Rust (example from `frontend/src-tauri/src/lib.rs`)
1. `std::*` first
2. External crates (`serde`, `tokio`, `anyhow`, `tauri`, `log`)
3. Local crate modules (`use audio::...`, `use notifications::...`)

```rust
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex as StdMutex;
// ... external crates ...
use audio::{list_audio_devices, AudioDevice, trigger_audio_permission};
use log::{error as log_error, info as log_info};
use tauri::{AppHandle, Manager, Runtime};
use tokio::sync::RwLock;
```

### TypeScript
1. External packages first (`@tauri-apps/api/core`, `react`, `next`)
2. Internal absolute imports via `@/` alias
3. Relative imports last

```ts
// frontend/src/services/recordingService.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
```

### Python (`backend/app/main.py`)
1. Stdlib (`logging`, `json`, `time`, `typing`)
2. Third-party (`fastapi`, `pydantic`, `uvicorn`, `dotenv`)
3. Local modules (`from db import DatabaseManager`, `from transcript_processor import TranscriptProcessor`)

## Error Handling Patterns

### Rust — `anyhow::Result` for internal, `Result<T, String>` for Tauri commands
- Library/internal code uses `anyhow::Result<T>` (declared in workspace at `Cargo.toml`)
- Tauri commands convert errors to `String` so they cross the FFI boundary into JS:

```rust
// frontend/src-tauri/src/lib.rs
#[tauri::command]
async fn start_recording<R: Runtime>(
    app: AppHandle<R>,
    mic_device_name: Option<String>,
    system_device_name: Option<String>,
    meeting_name: Option<String>,
) -> Result<(), String> {
    if is_recording().await {
        return Err("Recording already in progress".to_string());
    }
    match audio::recording_commands::start_recording_with_devices_and_meeting(...).await {
        Ok(_) => { /* ... */ Ok(()) },
        Err(e) => Err(e.to_string()),
    }
}
```

- `frontend/src-tauri/src/audio/vad.rs` shows `anyhow::{anyhow, Result}` pattern for internal modules
- Use `?` operator for propagation in non-command code

### TypeScript — `try/catch` with user-friendly messages
- Service classes (`frontend/src/services/recordingService.ts`) are thin pass-throughs that let errors bubble up to React components
- React components wrap calls in `try/catch` and show toasts via `sonner`

### Python — FastAPI `HTTPException`
- Endpoints throw `HTTPException(status_code=..., detail=...)` for client errors
- Pydantic models perform request validation automatically (see `backend/app/main.py`)

## Logging Patterns

### Rust — performance-aware macros
**Critical pattern** (`frontend/src-tauri/src/lib.rs`): Use `perf_debug!()` and `perf_trace!()` macros for hot-path logging — they are zero-cost in release builds.

```rust
// Defined in lib.rs and re-exported to all modules:
#[cfg(debug_assertions)]
macro_rules! perf_debug { ($($arg:tt)*) => { log::debug!($($arg)*) }; }

#[cfg(not(debug_assertions))]
macro_rules! perf_debug { ($($arg:tt)*) => {}; }

pub(crate) use perf_debug;
pub(crate) use perf_trace;
```

**Rules:**
- Use `log::info!`, `log::warn!`, `log::error!` for normal-frequency logging (`use log::{error as log_error, info as log_info}` in `lib.rs`)
- Use `perf_debug!()`/`perf_trace!()` ONLY for code on audio hot paths (per-sample, per-frame loops in `pipeline.rs`, `vad.rs`)
- Emoji prefixes used in critical info logs for visual scanning: `🔥 CALLED`, `📋 Backend received`, etc. (see `lib.rs:90-96`)

### Python — detailed formatter with file:line:function
`backend/app/main.py` configures the standard logger:

```python
logger = logging.getLogger(__name__)
logger.setLevel(logging.DEBUG)
console_handler = logging.StreamHandler()
formatter = logging.Formatter(
    '%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d - %(funcName)s()] - %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
console_handler.setFormatter(formatter)
```

**Rule:** Always include file/line/function in log output for backend modules. Use module-level `logger = logging.getLogger(__name__)`.

### TypeScript — `console.log/warn/error`
No structured logger; React components and services use plain `console.*`. Toggle visibility from in-app console UI (`frontend/src-tauri/src/console_utils/`).

## Async Patterns

### Rust shared state
- **Mutable shared state across async tasks:** `Arc<RwLock<T>>` from `tokio::sync` (NOT `std::sync::RwLock`)
- **Simple boolean flags:** `Arc<AtomicBool>` with `Ordering::SeqCst`
- **Static globals:** `std::sync::LazyLock<StdMutex<T>>` for app-wide preferences

Examples (`frontend/src-tauri/src/lib.rs`):
```rust
static RECORDING_FLAG: AtomicBool = AtomicBool::new(false);

static LANGUAGE_PREFERENCE: std::sync::LazyLock<StdMutex<String>> =
    std::sync::LazyLock::new(|| StdMutex::new("auto-translate".to_string()));
```

`frontend/src-tauri/src/audio/recording_state.rs` shows the pattern:
```rust
pub struct RecordingState {
    is_recording: Arc<AtomicBool>,
    audio_sender: Arc<RwLock<Option<mpsc::UnboundedSender<AudioChunk>>>>,
}
```

- **Channels for streaming data:** `tokio::sync::mpsc::UnboundedSender` for audio chunk pipelines
- **Async runtime:** Tauri provides Tokio; mark commands `async fn`

### TypeScript async
- All Tauri service methods are `async` returning `Promise<T>`
- Event listeners return `UnlistenFn` for cleanup in `useEffect`

```ts
// frontend/src/services/recordingService.ts
async startRecording(): Promise<void> {
  return invoke('start_recording');
}
```

### Python async
- FastAPI endpoints are `async def`
- Database access uses `aiosqlite` with `await db.some_operation()`

## Tauri Command & Event Conventions

### Commands (Frontend → Rust)
- Snake_case names: `start_recording`, `is_recording`, `get_recording_state`
- All commands return `Result<T, String>` (errors must be `String`)
- Generic over `Runtime`: `<R: Runtime>` and accept `AppHandle<R>` for emitting events
- Optional parameters use `Option<String>` (becomes `string | null` in JS)
- Registered in `tauri::Builder::invoke_handler(tauri::generate_handler![...])` in `frontend/src-tauri/src/lib.rs`

```rust
#[tauri::command]
async fn start_recording<R: Runtime>(
    app: AppHandle<R>,
    mic_device_name: Option<String>,
    system_device_name: Option<String>,
    meeting_name: Option<String>,
) -> Result<(), String> { ... }
```

### Events (Rust → Frontend)
- Kebab-case event names: `transcript-update`, `recording-stopped`, `audio-level`
- Payloads are `#[derive(Serialize, Clone)]` structs (e.g. `TranscriptionStatus`)
- Frontend listens via `import { listen } from '@tauri-apps/api/event'`

### Service wrapping pattern (frontend)
- Each Tauri command/event group has a corresponding service class in `frontend/src/services/`
- Services are 1:1 wrappers — no error transformation, just typed `invoke` calls
- See `frontend/src/services/recordingService.ts` (RecordingService), `transcriptService.ts`, `configService.ts`, `storageService.ts`, `indexedDBService.ts`, `updateService.ts`

## Comments

### When to comment
- Public Rust APIs use `///` doc comments (rustdoc)
- TypeScript services use JSDoc on exported methods
- Inline comments explain WHY, not WHAT — e.g. `// CONTINUOUS SPEECH FIX: Tuned for capturing complete 5+ second utterances` in `audio/vad.rs`
- Section headers in long files use `// ===== Section Name =====` style

### Architectural comments
Long-lived files (e.g. `frontend/src-tauri/src/audio/pipeline.rs`, `frontend/src-tauri/src/lib.rs`) carry block-comment summaries explaining purpose, threading model, and gotchas at the top.

## Function & Module Design

### Function size
- Rust: prefer focused functions; the monolithic `audio/core.rs` (1028 lines) was deliberately split into `audio/devices/`, `audio/capture/`, `audio/pipeline.rs`, etc. (see `AUDIO_MODULARIZATION_PLAN.md`)
- TypeScript services: one method = one Tauri command, kept short

### Module organization (Rust)
- `mod.rs` files re-export public surface
- Platform-specific code lives under `platform/{windows,macos,linux}.rs` subdirs (`audio/devices/platform/`)
- Submodules: `audio/devices/`, `audio/capture/`, `audio/devices/platform/`

### Component organization (React)
- Provider pattern for global state: `frontend/src/components/Sidebar/SidebarProvider.tsx`
- Context wraps app at top level, consumed via custom hook `useSidebar`
- Components colocated with their providers: `frontend/src/components/Sidebar/{index.tsx, SidebarProvider.tsx}`

## Domain-Specific Conventions

1. **Audio device naming:** Always `microphone` and `system`. Never `input`/`output`. Enforced across Rust, TS, and Python layers.
2. **Whisper model names:** Lowercase with hyphens: `tiny`, `base`, `small.en`, `large-v3-turbo`
3. **Branch naming:** `fix/*`, `enhance/*`, `feature/*`; `main` is stable
4. **Sample rate:** Pipeline assumes 48 kHz. Whisper VAD requires 16 kHz (resampled in `audio/vad.rs`)
5. **Storage paths:** Use Tauri path APIs (`downloadDir`, etc.) — never hardcode

---

*Convention analysis: 2026-04-07*
