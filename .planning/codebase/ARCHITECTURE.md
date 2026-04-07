# Architecture

**Analysis Date:** 2026-04-07

## Pattern Overview

**Overall:** Three-tier privacy-first desktop application with local-first processing.

```
┌──────────────────────────────────────────────────────────────────────┐
│  Tier 1: Frontend (Tauri Desktop App)                                 │
│  ┌──────────────────┐   Tauri IPC    ┌────────────────────────────┐  │
│  │  Next.js / React │ ◄────────────► │  Rust Backend (src-tauri)  │  │
│  │  (TypeScript UI) │  cmds + events │  (Audio + Whisper + IPC)   │  │
│  └──────────────────┘                └────────────────────────────┘  │
└────────────────────────────────────────────────┬─────────────────────┘
                                                 │ HTTP / WebSocket
                                                 ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Tier 2: Backend API (FastAPI, Python)                                │
│  ┌─────────────┐    ┌────────────────────┐    ┌────────────────────┐ │
│  │   SQLite    │◄──►│  DatabaseManager   │◄──►│ SummaryProcessor   │ │
│  │ (meetings,  │    │ (CRUD + metadata)  │    │ (LLM orchestration)│ │
│  │ summaries)  │    └────────────────────┘    └─────────┬──────────┘ │
│  └─────────────┘                                        │            │
└─────────────────────────────────────────────────────────┼────────────┘
                                                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Tier 3: LLM Providers                                                │
│  Ollama (local) | Claude | Groq | OpenRouter                          │
└──────────────────────────────────────────────────────────────────────┘
```

**Key Characteristics:**
- Local-first: Audio capture, mixing, VAD, and Whisper transcription run entirely on-device in Rust
- Two-process desktop runtime: Next.js webview UI + native Rust audio/STT engine, bridged by Tauri IPC
- Backend is **optional for capture** but required for persistent meeting storage and LLM summarization
- Modular Rust audio system with platform-specific device backends (WASAPI / ScreenCaptureKit / ALSA)
- Dual-path audio pipeline: one path for high-fidelity recording, one for VAD-filtered transcription

## Layers

**Tier 1a — Next.js UI Layer (TypeScript / React):**
- Purpose: User-facing recording controls, transcript display, settings, meeting browser
- Location: `frontend/src/`
- Contains: React components, hooks, contexts, services calling Tauri commands and the FastAPI backend
- Depends on: Tauri `invoke` API, FastAPI HTTP endpoints, browser DOM
- Used by: End user via Tauri webview

**Tier 1b — Rust Native Layer (Tauri Backend):**
- Purpose: Audio device access, capture, mixing, VAD, Whisper inference, IPC command handlers
- Location: `frontend/src-tauri/src/`
- Contains: Tauri command handlers, audio pipeline, Whisper engine, system tray, notifications, onboarding
- Depends on: `cpal`, `whisper-rs`, OS audio APIs, `tauri` runtime
- Used by: UI layer via `invoke()` and Tauri events

**Audio Submodule (modularized from former monolithic `core.rs`):**
- Purpose: Cross-platform audio capture, mixing, VAD, recording persistence
- Location: `frontend/src-tauri/src/audio/`
- Sub-layers:
  - `audio/devices/` — discovery and configuration of input/output devices
  - `audio/devices/platform/` — OS-specific device enumeration backends
  - `audio/capture/` — live capture streams (mic + system loopback)
  - `audio/pipeline.rs` — mixing, VAD, fan-out
  - `audio/recording_manager.rs` — high-level orchestration
  - `audio/recording_saver.rs` — WAV file persistence
  - `audio/recording_commands.rs` — Tauri command interface for the audio module

**Whisper Engine Submodule:**
- Purpose: Local speech-to-text inference with GPU acceleration
- Location: `frontend/src-tauri/src/whisper_engine/`
- Contains: Model loader, transcription commands, parallel processor, system monitor
- Depends on: `whisper-rs`, Metal/CoreML (macOS), CUDA/Vulkan (Win/Linux)
- Used by: Audio pipeline (transcription path), Tauri commands

**Tier 2 — FastAPI Backend Layer (Python):**
- Purpose: Persistent meeting storage, transcript processing, LLM-based summarization
- Location: `backend/app/`
- Contains: HTTP routes (`main.py`), database manager (`db.py`), transcript processor (`transcript_processor.py`), schema validator (`schema_validator.py`)
- Depends on: `fastapi`, `aiosqlite`, LLM provider SDKs
- Used by: Frontend over HTTP at `http://localhost:5167`

**Tier 3 — LLM Provider Layer:**
- Purpose: Generate summaries from transcripts
- Location: Provider modules under `frontend/src-tauri/src/{anthropic,groq,ollama,openai,openrouter}` and backend equivalents
- Contains: Provider clients, prompt assembly, streaming handlers

## Data Flow

### Audio Dual-Path Flow (the core insight)

```
                         ┌──────────────────────────┐
                         │   Microphone (CPAL)      │
                         │ frontend/src-tauri/src/  │
                         │ audio/capture/           │
                         │ microphone.rs            │
                         └────────────┬─────────────┘
                                      │
                         ┌────────────┴─────────────┐
                         │  System Audio Loopback   │
                         │ audio/capture/system.rs  │
                         │ + core_audio.rs (macOS)  │
                         └────────────┬─────────────┘
                                      │
                                      ▼
                  ┌────────────────────────────────────────┐
                  │      AudioPipelineManager              │
                  │ frontend/src-tauri/src/audio/          │
                  │ pipeline.rs                            │
                  │  - AudioMixerRingBuffer (50ms windows) │
                  │  - ProfessionalAudioMixer (RMS duck)   │
                  │  - AudioCapture (fan-out)              │
                  │  - AudioPipeline (orchestration)       │
                  └────────────┬───────────────┬───────────┘
                               │               │
              ┌────────────────┘               └────────────────────┐
              ▼ Recording path (mixed)              Transcription   ▼ path (VAD)
   ┌─────────────────────────┐                        ┌──────────────────────────┐
   │  RecordingSaver         │                        │  VAD filter (vad.rs)     │
   │  audio/recording_saver  │                        │  → speech-only chunks    │
   │  .rs  → WAV on disk     │                        └────────────┬─────────────┘
   └─────────────────────────┘                                     │
                                                                   ▼
                                                  ┌───────────────────────────────┐
                                                  │  WhisperEngine                │
                                                  │  whisper_engine/whisper_      │
                                                  │  engine.rs                    │
                                                  │  (Metal / CUDA / Vulkan / CPU)│
                                                  └───────────────┬───────────────┘
                                                                  │ transcript chunk
                                                                  ▼
                                                  ┌───────────────────────────────┐
                                                  │  Tauri event:                 │
                                                  │  "transcript-update"          │
                                                  │  app.emit(...)                │
                                                  └───────────────┬───────────────┘
                                                                  ▼
                                                  ┌───────────────────────────────┐
                                                  │  Frontend listener            │
                                                  │  frontend/src/app/page.tsx +  │
                                                  │  Sidebar context              │
                                                  └───────────────────────────────┘
```

**Key insight:** Recording path and transcription path are **siblings**, not stages. The pipeline tees audio after mixing — one branch goes to disk for the user's archival recording (full fidelity, RMS-ducked), the other goes through VAD before being fed to Whisper (drops ~70% of silent audio to reduce STT load).

### Tauri IPC Flow

**Command pattern (Frontend → Rust):**

```
[React component / hook]
   │  invoke('start_recording_with_devices_and_meeting', { ... })
   ▼
[Tauri runtime]
   │
   ▼
frontend/src-tauri/src/lib.rs
   #[tauri::command]
   async fn start_recording_with_devices_and_meeting<R: Runtime>(...)
   │
   ▼
frontend/src-tauri/src/audio/recording_commands.rs
   │
   ▼
frontend/src-tauri/src/audio/recording_manager.rs
   │
   ▼
AudioPipelineManager  +  RecordingState (Arc<RwLock<...>>)
```

**Event pattern (Rust → Frontend):**

```
RecordingManager / Pipeline / WhisperEngine
   │  app.emit("transcript-update", payload)
   │  app.emit("audio-level-update", payload)
   │  app.emit("recording-status", payload)
   ▼
[Tauri runtime broadcasts]
   │
   ▼
frontend/src/app/page.tsx
   await listen<TranscriptUpdate>('transcript-update', ...)
   │
   ▼
React state update → Sidebar context propagation → UI re-render
```

### Meeting Persistence Flow (Frontend ↔ Backend)

```
User stops recording
   │
   ▼
Frontend collects: title, transcripts[], audio file path
   │
   ▼  HTTP POST
http://localhost:5167/save-transcript
   │
   ▼
backend/app/main.py  (save_transcript handler)
   │
   ▼
backend/app/db.py  → DatabaseManager → SQLite (aiosqlite)
   │
   ▼  HTTP POST
http://localhost:5167/process-transcript
   │
   ▼
SummaryProcessor  →  LLM provider (Ollama / Claude / Groq / OpenRouter)
   │
   ▼
SQLite (summary persisted)  →  GET /get-summary/{meeting_id}
```

## Key Abstractions

**`AudioPipelineManager`:**
- Purpose: Top-level orchestrator for the entire audio capture + processing pipeline
- File: `frontend/src-tauri/src/audio/pipeline.rs` (line ~944)
- Pattern: Owns `AudioCapture`, ring buffer, mixer, VAD, and the fan-out to recording + transcription paths
- Lifecycle: Created on `start_recording`, destroyed on `stop_recording`

**`AudioMixerRingBuffer`:**
- Purpose: Synchronizes asynchronously-arriving mic and system audio into aligned 50ms windows
- File: `frontend/src-tauri/src/audio/pipeline.rs` (line ~25)
- Pattern: `VecDeque`-backed ring buffer with windowed dequeue

**`ProfessionalAudioMixer`:**
- Purpose: RMS-based ducking and clipping prevention so system audio doesn't drown out speech
- File: `frontend/src-tauri/src/audio/pipeline.rs` (line ~149)

**`AudioCapture`:**
- Purpose: Manages live capture streams from mic + system devices
- File: `frontend/src-tauri/src/audio/pipeline.rs` (line ~194)

**`RecordingState`:**
- Purpose: Thread-safe shared state across async tasks for the active recording
- File: `frontend/src-tauri/src/audio/recording_state.rs` (line ~95)
- Pattern: `Arc<AtomicBool>` for flags, `Arc<RwLock<Option<mpsc::UnboundedSender<AudioChunk>>>>` for channels
- Companions: `AudioChunk`, `ProcessedAudioChunk`, `AudioError`, `RecordingStats`

**`RecordingManager` (high-level coordinator):**
- File: `frontend/src-tauri/src/audio/recording_manager.rs`
- Purpose: Bridges Tauri commands and the audio pipeline; owns the `RecordingState`

**`WhisperEngine`:**
- Purpose: Local Whisper model loading and transcription with GPU acceleration
- File: `frontend/src-tauri/src/whisper_engine/whisper_engine.rs` (line ~35)
- Companions: `ModelInfo` (line ~25), `system_monitor.rs` (GPU detection), `parallel_processor.rs` (batch workloads)
- Pattern: Loads model once, caches it; auto-detects Metal / CoreML / CUDA / Vulkan / CPU

**`DatabaseManager`:**
- Purpose: All SQLite CRUD operations for meetings, transcripts, summaries
- File: `backend/app/db.py` (line ~20)
- Pattern: Async via `aiosqlite`; single class with methods consumed by `main.py` route handlers

**`SummaryProcessor`:**
- Purpose: Orchestrates LLM calls to generate meeting summaries
- File: `backend/app/main.py` (line ~110)
- Pattern: Background-task-driven (`process_transcript_background`) so HTTP requests return immediately

**`SidebarProvider` (frontend global context):**
- Purpose: Global React state for meetings list, current meeting, recording status, WebSocket connections
- File: `frontend/src/components/Sidebar/SidebarProvider.tsx`
- Pattern: React Context provider wrapping the app; consumed via `useSidebar()` hook
- Communicates with: backend at `http://localhost:5167`, Tauri events from Rust layer

## Entry Points

**Rust Native Entry Point:**
- File: `frontend/src-tauri/src/lib.rs`
- Function: `pub fn run()` (line ~390)
- Triggers: Tauri runtime startup
- Responsibilities:
  - Registers all `#[tauri::command]` handlers (`start_recording`, `stop_recording`, `get_audio_devices`, `start_recording_with_devices_and_meeting`, `set_language_preference`, `read_audio_file`, `save_transcript`, audio-level monitoring commands, etc.)
  - Initializes app state, system tray, notifications, onboarding
  - Spawns the webview hosting the Next.js UI

**Rust Process Entry:**
- File: `frontend/src-tauri/src/main.rs`
- Calls into `lib::run()`

**Frontend UI Entry Point:**
- File: `frontend/src/app/layout.tsx`
- Wraps the app in providers (Sidebar, Analytics, UpdateCheck)
- File: `frontend/src/app/page.tsx`
- Triggers: User loads the desktop app (root route)
- Responsibilities:
  - Recording controls UI
  - Subscribes to Tauri events (`transcript-update`, `audio-level-update`, `recording-status`)
  - Invokes Tauri commands for start/stop/device-selection
  - Renders the live transcript view

**Backend HTTP Entry Point:**
- File: `backend/app/main.py`
- Triggers: `clean_start_backend.sh` / `clean_start_backend.cmd` launches uvicorn on port 5167
- Responsibilities:
  - Defines FastAPI `app`
  - Registers all REST endpoints (`/get-meetings`, `/get-meeting/{id}`, `/save-meeting-title`, `/delete-meeting`, `/process-transcript`, `/get-summary/{id}`, `/save-transcript`, `/get-model-config`, `/save-model-config`, `/get-transcript-config`, `/save-transcript-config`, `/get-api-key`, `/save-meeting-summary`, `/search-transcripts`)
  - Configures CORS (open `*` for development)
  - Hosts background task pool for summary generation

## Error Handling

**Strategy:** Layered — Rust uses `anyhow::Result` and `Result<T, String>` at the Tauri command boundary; Python uses FastAPI exception handlers; frontend uses try/catch with user-facing toast notifications.

**Patterns:**
- Tauri commands return `Result<T, String>` so the JS side gets a parseable error message
- `AudioError` enum (`audio/recording_state.rs` line ~51) for typed audio failures
- FastAPI endpoints raise `HTTPException` for client errors; background tasks catch all exceptions and persist failure state to SQLite
- Frontend `MessageToast.tsx` and `CustomDialog.tsx` surface errors to the user
- Permission failures handled by `audio/permissions.rs` and `PermissionWarning.tsx`

## Cross-Cutting Concerns

**Logging:**
- Rust: `log` crate + custom `perf_debug!` / `perf_trace!` macros (zero-cost in release builds)
- Audio metrics: batched via `AudioMetricsBatcher` in `audio/pipeline.rs`
- Async logger: `audio/async_logger.rs` for non-blocking hot-path logs
- Python: standard `logging` with detailed format `filename:line:function`
- Frontend: `console_utils` Rust module + in-app `ConsoleToggle.tsx`

**Validation:**
- Backend: Pydantic models in `backend/app/main.py` (`Transcript`, `MeetingResponse`, `TranscriptRequest`, etc.)
- Schema migrations validated via `backend/app/schema_validator.py`
- Frontend: TypeScript types in `frontend/src/types/`

**Authentication:**
- Local-first design — no user auth on the desktop app
- LLM provider API keys stored locally and retrieved via `/get-api-key` and `/get-transcript-api-key` endpoints

**State Management:**
- Rust: `Arc<RwLock<T>>` for shared async state, `Arc<AtomicBool>` for flags
- Frontend: React Context (`SidebarProvider`) for global state; local `useState` for component state
- Persistence: SQLite via backend `DatabaseManager`; local preferences via Tauri filesystem APIs

**Performance:**
- VAD reduces Whisper load by ~70% (transcribe only speech)
- `AudioBufferPool` (`audio/buffer_pool.rs`) pre-allocates buffers to avoid GC pressure
- Whisper GPU acceleration: Metal/CoreML on macOS, CUDA/Vulkan on Win/Linux, CPU fallback
- Frontend uses `VirtualizedTranscriptView.tsx` for large transcripts
- Audio level UI throttled to 60fps

---

*Architecture analysis: 2026-04-07*
