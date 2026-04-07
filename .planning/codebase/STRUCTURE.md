# Codebase Structure

**Analysis Date:** 2026-04-07

## Top-Level Layout

```
Meetily/
├── frontend/                       # Tauri desktop app (Next.js + Rust)
│   ├── src/                        # Next.js / React UI (TypeScript)
│   ├── src-tauri/                  # Rust native side (audio, Whisper, IPC)
│   ├── public/                     # Static assets served by Next.js
│   ├── scripts/                    # Frontend build/dev scripts
│   ├── package.json                # JS deps + pnpm scripts
│   ├── next.config.js              # Next.js config
│   ├── tailwind.config.ts          # Tailwind config
│   ├── tsconfig.json               # TypeScript config
│   ├── eslint.config.mjs           # Lint rules
│   ├── components.json             # shadcn/ui registry
│   ├── clean_run.sh                # macOS dev runner
│   ├── clean_build.sh              # macOS production build
│   ├── clean_run_windows.bat       # Windows dev runner
│   └── clean_build_windows.bat     # Windows production build
├── backend/                        # FastAPI server (Python)
│   ├── app/                        # API source
│   ├── whisper-custom/             # Custom Whisper build artifacts
│   ├── whisper.cpp/                # Whisper C++ submodule
│   ├── docker/                     # Docker support files
│   ├── docker-compose.yml          # Multi-service compose
│   ├── Dockerfile.app              # FastAPI app image
│   ├── Dockerfile.server-cpu       # Whisper server (CPU)
│   ├── Dockerfile.server-gpu       # Whisper server (GPU)
│   ├── Dockerfile.server-macos     # Whisper server (macOS Metal)
│   ├── requirements.txt            # Python deps
│   ├── build_whisper.sh            # macOS Whisper builder
│   ├── build_whisper.cmd           # Windows Whisper builder
│   ├── clean_start_backend.sh      # macOS backend launcher (port 5167)
│   └── clean_start_backend.cmd     # Windows backend launcher
├── docs/                           # Project docs (plus superpowers/specs)
├── scripts/                        # Repo-wide helper scripts
├── llama-helper/                   # llama.cpp helper utilities
├── .planning/                      # GSD planning workspace
│   └── codebase/                   # Codebase analysis docs (this file lives here)
├── Cargo.toml                      # Rust workspace manifest
├── CLAUDE.md                       # AI-agent project instructions
├── README.md
└── LICENSE.md
```

## Frontend (Next.js / React) — `frontend/src/`

```
frontend/src/
├── app/                            # Next.js App Router routes
│   ├── _components/                # Route-private components
│   ├── meeting-details/            # Meeting detail page
│   ├── notes/                      # Notes page
│   ├── settings/                   # Settings page
│   ├── layout.tsx                  # Root layout (providers wrap here)
│   ├── page.tsx                    # ★ Main recording UI (root route)
│   ├── globals.css                 # Global styles + Tailwind imports
│   ├── metadata.ts                 # Static metadata
│   ├── metadata.tsx                # Dynamic metadata helpers
│   └── favicon.ico
├── components/                     # Reusable React components (PascalCase)
│   ├── Sidebar/                    # ★ Global state provider lives here
│   │   ├── SidebarProvider.tsx     # React Context: meetings, recording state
│   │   └── index.tsx               # Sidebar UI
│   ├── MainContent/                # Main content area shell
│   ├── MainNav/                    # Top navigation
│   ├── MeetingDetails/             # Meeting detail subcomponents
│   ├── AISummary/                  # AI summary rendering
│   ├── BlockNoteEditor/            # Rich-text editor wrapper
│   ├── ConfirmationModel/          # Confirmation dialogs
│   ├── DatabaseImport/             # Import flow
│   ├── ImportAudio/                # Audio import flow
│   ├── TranscriptRecovery/         # Crash-recovery flow
│   ├── molecules/                  # Mid-level composite components
│   ├── shared/                     # Shared primitives
│   ├── ui/                         # shadcn/ui generated primitives
│   ├── onboarding/                 # First-run flow
│   ├── RecordingControls.tsx
│   ├── RecordingSettings.tsx
│   ├── RecordingStatusBar.tsx
│   ├── DeviceSelection.tsx
│   ├── AudioBackendSelector.tsx
│   ├── AudioLevelMeter.tsx
│   ├── AudioPlayer.tsx
│   ├── TranscriptView.tsx
│   ├── VirtualizedTranscriptView.tsx
│   ├── ConfidenceIndicator.tsx
│   ├── WhisperModelManager.tsx
│   ├── ParakeetModelManager.tsx
│   ├── BuiltInModelManager.tsx
│   ├── ModelSettingsModal.tsx
│   ├── ModelDownloadProgress.tsx
│   ├── SummaryModelSettings.tsx
│   ├── TranscriptSettings.tsx
│   ├── PreferenceSettings.tsx
│   ├── BetaSettings.tsx
│   ├── SettingTabs.tsx
│   ├── LanguageSelection.tsx
│   ├── PermissionWarning.tsx
│   ├── BluetoothPlaybackWarning.tsx
│   ├── ChunkProgressDisplay.tsx
│   ├── EditableTitle.tsx
│   ├── EmptyStateSummary.tsx
│   ├── ComplianceNotification.tsx
│   ├── MessageToast.tsx
│   ├── CustomDialog.tsx
│   ├── ConsoleToggle.tsx
│   ├── About.tsx
│   ├── Info.tsx
│   ├── Logo.tsx
│   ├── AnalyticsProvider.tsx
│   ├── AnalyticsConsentSwitch.tsx
│   ├── AnalyticsDataModal.tsx
│   ├── UpdateCheckProvider.tsx
│   ├── UpdateDialog.tsx
│   └── UpdateNotification.tsx
├── config/                         # App configuration constants
├── constants/                      # Shared constants
├── contexts/                       # Additional React contexts
├── hooks/                          # Custom React hooks
├── lib/                            # Pure utility functions
├── services/                       # API clients (Tauri invoke + HTTP)
└── types/                          # Shared TypeScript types
```

## Rust Native Layer — `frontend/src-tauri/src/`

```
frontend/src-tauri/src/
├── lib.rs                          # ★ Tauri entry: registers commands, runs app
├── main.rs                         # Process entry, calls lib::run()
├── lib_old_complex.rs              # Legacy entry (kept for reference)
├── state.rs                        # Global app state types
├── config.rs                       # App configuration
├── onboarding.rs                   # First-run setup logic
├── tray.rs                         # System tray integration
├── utils.rs                        # General utilities
│
├── audio/                          # ★ Modularized audio system (see below)
├── audio_v2/                       # Experimental next-gen audio backend
├── whisper_engine/                 # ★ Whisper STT engine (see below)
├── parakeet_engine/                # Alternative STT engine (Parakeet)
├── transcription/                  # Transcription orchestration
├── summary/                        # Local summary generation
├── database/                       # Local DB helpers
├── notifications/                  # Native notifications
├── analytics/                      # Telemetry (opt-in)
├── api/                            # HTTP client to FastAPI backend
├── console_utils/                  # Logging/console helpers
│
├── anthropic/                      # Claude provider client
├── groq/                           # Groq provider client
├── ollama/                         # Ollama provider client
├── openai/                         # OpenAI provider client
└── openrouter/                     # OpenRouter provider client
```

## Audio Subdirectory — `frontend/src-tauri/src/audio/` (verbatim)

```
audio/
├── mod.rs                          # Module exports
├── pipeline.rs                     # ★ AudioPipelineManager, mixer, ring buffer, VAD fan-out
├── recording_manager.rs            # High-level recording orchestration
├── recording_commands.rs           # Tauri command interface for audio
├── recording_commands.rs.backup    # Backup from refactor (cleanup candidate)
├── recording_state.rs              # RecordingState, AudioChunk, AudioError, RecordingStats
├── recording_saver.rs              # WAV file writing
├── recording_saver_old.rs          # Legacy saver (cleanup candidate)
├── recording_preferences.rs        # User recording prefs
├── core-old.rs                     # Pre-modularization monolith (cleanup candidate)
│
├── devices/                        # Device discovery & configuration
│   ├── mod.rs
│   ├── discovery.rs                # list_audio_devices, trigger_audio_permission
│   ├── microphone.rs               # default_input_device
│   ├── speakers.rs                 # default_output_device
│   ├── configuration.rs            # AudioDevice types, parsing
│   ├── fallback.rs                 # Fallback device selection
│   └── platform/                   # OS-specific device backends
│       ├── mod.rs
│       ├── windows.rs              # WASAPI logic
│       ├── macos.rs                # ScreenCaptureKit logic
│       └── linux.rs                # ALSA / PulseAudio logic
│
├── capture/                        # Audio stream capture
│   ├── mod.rs
│   ├── microphone.rs               # Microphone capture stream
│   ├── system.rs                   # System audio capture stream
│   ├── core_audio.rs               # macOS ScreenCaptureKit integration
│   └── backend_config.rs           # Capture backend selection
│
├── transcription/                  # Transcription glue between audio and STT
│
├── async_logger.rs                 # Non-blocking hot-path logger
├── audio_processing.rs             # Sample format conversion / resampling
├── batch_processor.rs              # Batched audio processing
├── buffer_pool.rs                  # AudioBufferPool (pre-allocated buffers)
├── common.rs                       # Shared audio types
├── constants.rs                    # Audio constants (sample rates, sizes)
├── decoder.rs                      # Audio decoding
├── device_detection.rs             # Device capability probing
├── device_monitor.rs               # Hot-plug device monitoring
├── diagnostics.rs                  # Diagnostic dump utilities
├── encode.rs                       # Encoding helpers
├── ffmpeg_mixer.rs                 # ffmpeg-based mixer
├── ffmpeg.rs                       # ffmpeg bindings
├── hardware_detector.rs            # CPU/GPU detection
├── import.rs                       # Audio file import
├── incremental_saver.rs            # Streaming WAV writer
├── level_monitor.rs                # Audio level metering
├── simple_level_monitor.rs         # Lightweight level meter
├── permissions.rs                  # OS audio/screen permission handling
├── playback_monitor.rs             # Playback monitoring (for testing)
├── post_processor.rs               # Post-recording processing
├── retranscription.rs              # Re-run STT on saved audio
├── stream.rs                       # Stream abstraction
├── stt.rs                          # STT integration glue
├── system_audio_commands.rs        # Tauri commands for system-audio path
├── system_audio_stream.rs          # System audio stream impl
├── system_audio_types.ts           # ★ TS type stubs co-located in Rust dir
├── system_detector.rs              # OS detection
└── vad.rs                          # Voice Activity Detection filter
```

## Whisper Engine Subdirectory — `frontend/src-tauri/src/whisper_engine/`

```
whisper_engine/
├── mod.rs                          # Module exports
├── whisper_engine.rs               # ★ WhisperEngine: load_model, transcribe
├── commands.rs                     # Tauri commands for model management
├── parallel_commands.rs            # Tauri commands for parallel transcription
├── parallel_processor.rs           # Batch / parallel transcription
├── system_monitor.rs               # GPU detection, hardware capabilities
└── _stderr_suppressor.rs           # Suppresses noisy whisper.cpp stderr
```

## Backend (FastAPI) — `backend/`

```
backend/
├── app/
│   ├── main.py                     # ★ FastAPI app, all REST routes, SummaryProcessor
│   ├── db.py                       # ★ DatabaseManager (aiosqlite)
│   ├── transcript_processor.py     # Transcript chunking and prep
│   └── schema_validator.py         # SQLite schema migration validator
├── whisper.cpp/                    # Whisper C++ source (git submodule)
├── whisper-custom/                 # Custom-built Whisper artifacts
├── docker/                         # Docker support assets
├── examples/                       # Example payloads / scripts
├── docker-compose.yml
├── Dockerfile.app
├── Dockerfile.server-cpu
├── Dockerfile.server-gpu
├── Dockerfile.server-macos
├── requirements.txt
├── build_whisper.sh                # macOS Whisper builder
├── build_whisper.cmd               # Windows Whisper builder
├── download-ggml-model.sh          # Model downloader (macOS)
├── download-ggml-model.cmd         # Model downloader (Windows)
├── clean_start_backend.sh          # Start FastAPI on port 5167 (macOS)
├── clean_start_backend.cmd         # Start FastAPI (Windows)
├── start_python_backend.cmd        # Alt Windows starter
├── start_whisper_server.cmd        # Whisper server starter
├── start_with_output.ps1           # Interactive PS starter
├── set_env.sh                      # Env var setup
├── setup-db.sh                     # SQLite init (macOS)
├── setup-db.ps1                    # SQLite init (Windows)
├── debug_cors.py                   # CORS debug helper
├── temp.env                        # Env template (do NOT commit secrets here)
├── API_DOCUMENTATION.md
├── SCRIPTS_DOCUMENTATION.md
└── README.md
```

## Directory Purposes

**`frontend/src/app/`:**
- Purpose: Next.js App Router pages and route-private components
- Key files: `frontend/src/app/page.tsx` (main recording UI), `frontend/src/app/layout.tsx` (provider wrapper)

**`frontend/src/components/`:**
- Purpose: All reusable React components, organized by feature folder + flat single-file components
- Convention: Folders for related groups (`Sidebar/`, `MeetingDetails/`, `MainNav/`); flat files for standalone components
- Key files: `frontend/src/components/Sidebar/SidebarProvider.tsx`, `frontend/src/components/RecordingControls.tsx`, `frontend/src/components/TranscriptView.tsx`

**`frontend/src/services/`:**
- Purpose: Wrappers around Tauri `invoke()` calls and HTTP clients to the FastAPI backend
- Where API call code lives — keeps components clean

**`frontend/src/contexts/`, `frontend/src/hooks/`:**
- Purpose: React contexts and custom hooks beyond the Sidebar context

**`frontend/src/types/`, `frontend/src/lib/`, `frontend/src/constants/`, `frontend/src/config/`:**
- Purpose: Shared TypeScript types, pure utilities, constants, configuration

**`frontend/src-tauri/src/`:**
- Purpose: Rust native side. Audio capture, Whisper inference, Tauri command handlers
- Key file: `frontend/src-tauri/src/lib.rs` (registers all Tauri commands and runs the app)

**`frontend/src-tauri/src/audio/`:**
- Purpose: Cross-platform audio capture, mixing, VAD, recording. Recently modularized from monolithic `core.rs`
- Key file: `frontend/src-tauri/src/audio/pipeline.rs` (contains `AudioPipelineManager`, `AudioMixerRingBuffer`, `ProfessionalAudioMixer`, `AudioCapture`, `AudioPipeline`)

**`frontend/src-tauri/src/audio/devices/`:**
- Purpose: Device discovery and configuration
- Pattern: Cross-platform interface (`discovery.rs`, `microphone.rs`, `speakers.rs`) delegates to platform-specific code in `platform/`

**`frontend/src-tauri/src/audio/devices/platform/`:**
- Purpose: OS-specific device enumeration and capability detection
- Files: `windows.rs` (WASAPI), `macos.rs` (ScreenCaptureKit), `linux.rs` (ALSA/PulseAudio)

**`frontend/src-tauri/src/audio/capture/`:**
- Purpose: Live audio stream capture from devices
- Files: `microphone.rs`, `system.rs`, `core_audio.rs` (macOS-specific ScreenCaptureKit)

**`frontend/src-tauri/src/whisper_engine/`:**
- Purpose: Local Whisper STT model loading and transcription
- Key file: `frontend/src-tauri/src/whisper_engine/whisper_engine.rs`

**`frontend/src-tauri/src/{anthropic,groq,ollama,openai,openrouter}/`:**
- Purpose: LLM provider clients (one folder per provider)

**`backend/app/`:**
- Purpose: FastAPI application source
- Key files: `backend/app/main.py` (routes), `backend/app/db.py` (DatabaseManager)

**`backend/whisper.cpp/`:**
- Purpose: Whisper C++ source as a git submodule (do not edit)

**`backend/whisper-custom/`:**
- Purpose: Custom-built Whisper binaries used by the backend's whisper server

## Key File Locations

**Entry Points:**
- `frontend/src-tauri/src/lib.rs` — Tauri command registration and app runner (`run()` at line ~390)
- `frontend/src-tauri/src/main.rs` — Rust process main
- `frontend/src/app/layout.tsx` — Next.js root layout (provider wrapping)
- `frontend/src/app/page.tsx` — Main recording UI route
- `backend/app/main.py` — FastAPI app and all HTTP endpoints

**Configuration:**
- `frontend/package.json` — JS deps + pnpm scripts (`tauri:dev`, `tauri:build`, GPU variants)
- `frontend/src-tauri/Cargo.toml` — Rust deps + Cargo features (`metal`, `cuda`, `vulkan`)
- `frontend/src-tauri/tauri.conf.json` — Tauri runtime config
- `frontend/next.config.js` — Next.js build config
- `frontend/tailwind.config.ts` — Tailwind theme
- `frontend/tsconfig.json` — TypeScript compiler options
- `frontend/eslint.config.mjs` — ESLint flat config
- `frontend/components.json` — shadcn/ui registry config
- `backend/requirements.txt` — Python deps
- `backend/docker-compose.yml` — Multi-service orchestration
- `Cargo.toml` (repo root) — Rust workspace manifest

**Core Audio Logic:**
- `frontend/src-tauri/src/audio/pipeline.rs` — Pipeline manager, mixer, ring buffer
- `frontend/src-tauri/src/audio/recording_manager.rs` — Recording orchestration
- `frontend/src-tauri/src/audio/recording_state.rs` — Shared async state
- `frontend/src-tauri/src/audio/recording_saver.rs` — WAV persistence
- `frontend/src-tauri/src/audio/vad.rs` — Voice Activity Detection
- `frontend/src-tauri/src/audio/recording_commands.rs` — Tauri command surface

**Core Whisper Logic:**
- `frontend/src-tauri/src/whisper_engine/whisper_engine.rs` — Model load + transcribe
- `frontend/src-tauri/src/whisper_engine/parallel_processor.rs` — Batch transcription
- `frontend/src-tauri/src/whisper_engine/system_monitor.rs` — GPU/hardware detection
- `frontend/src-tauri/src/whisper_engine/commands.rs` — Tauri commands

**Core Backend Logic:**
- `backend/app/main.py` — All HTTP routes + `SummaryProcessor`
- `backend/app/db.py` — `DatabaseManager` (SQLite via `aiosqlite`)
- `backend/app/transcript_processor.py` — Transcript prep
- `backend/app/schema_validator.py` — Migration validator

**Frontend State:**
- `frontend/src/components/Sidebar/SidebarProvider.tsx` — Global React context
- `frontend/src/components/Sidebar/index.tsx` — Sidebar UI

**Build & Run Scripts:**
- `frontend/clean_run.sh` — macOS dev (Tauri)
- `frontend/clean_build.sh` — macOS prod build
- `frontend/clean_run_windows.bat` — Windows dev
- `frontend/clean_build_windows.bat` — Windows prod build
- `frontend/dev-gpu.sh` / `dev-gpu.bat` / `dev-gpu.ps1` — GPU-feature dev runners
- `frontend/build-gpu.sh` / `build-gpu.bat` / `build-gpu.ps1` — GPU-feature builds
- `backend/build_whisper.sh` / `.cmd` — Build Whisper.cpp
- `backend/clean_start_backend.sh` / `.cmd` — Start FastAPI on port 5167
- `backend/run-docker.sh` / `.ps1` — Docker orchestration

**Documentation:**
- `CLAUDE.md` — Project instructions for AI agents
- `README.md`, `CONTRIBUTING.md`, `PRIVACY_POLICY.md`, `BLUETOOTH_PLAYBACK_NOTICE.md`
- `frontend/API.md`, `frontend/README.md`
- `backend/API_DOCUMENTATION.md`, `backend/SCRIPTS_DOCUMENTATION.md`, `backend/README.md`

## Naming Conventions

**Rust files (`frontend/src-tauri/src/`):**
- `snake_case.rs` for all module files
- Example: `recording_manager.rs`, `system_audio_stream.rs`, `whisper_engine.rs`
- Submodule directories use `snake_case/` with `mod.rs` inside

**Python files (`backend/app/`):**
- `snake_case.py`
- Example: `main.py`, `db.py`, `transcript_processor.py`, `schema_validator.py`
- Classes inside use `PascalCase` (`DatabaseManager`, `SummaryProcessor`, `MeetingResponse`)

**TypeScript components (`frontend/src/components/`):**
- `PascalCase.tsx` for component files
- Example: `RecordingControls.tsx`, `TranscriptView.tsx`, `SidebarProvider.tsx`
- Folder names also `PascalCase` when they group a feature: `Sidebar/`, `MeetingDetails/`, `MainNav/`
- Default-export the component matching the filename

**TypeScript non-component files (`frontend/src/lib`, `services/`, `hooks/`):**
- `camelCase.ts` for utilities, services, hooks
- React hooks prefixed with `use` (e.g., `useSidebar`)

**Next.js App Router (`frontend/src/app/`):**
- `kebab-case/` for route folders (`meeting-details/`, `notes/`, `settings/`)
- Special files use Next.js conventions: `page.tsx`, `layout.tsx`, `loading.tsx`, `error.tsx`, `not-found.tsx`
- Private/co-located components in `_components/` (underscore prefix excludes from routing)

**Audio device terminology (project-wide):**
- Use **`microphone`** and **`system`** consistently — never `input`/`output`
- Examples: `microphone.rs`, `system.rs`, `default_input_device` (function name allowed), `mic_device_name` / `system_device_name` (Tauri command args)
- Frontend mirrors this: `mic_device_name`, `system_device_name`

**Tauri commands:**
- `snake_case` function names exposed to JS (e.g., `start_recording`, `start_recording_with_devices_and_meeting`, `get_audio_devices`, `trigger_microphone_permission`)
- JS calls them with `invoke('snake_case_name', { ...args })` — args are also `snake_case`

**Tauri events:**
- `kebab-case` event names (e.g., `transcript-update`, `audio-level-update`, `recording-status`)

**Backend API endpoints:**
- `kebab-case` paths (e.g., `/get-meetings`, `/save-meeting-title`, `/process-transcript`, `/get-summary/{meeting_id}`)

## Where to Add New Code

**New audio device platform:**
- Create `frontend/src-tauri/src/audio/devices/platform/{platform_name}.rs`
- Wire it up in `frontend/src-tauri/src/audio/devices/platform/mod.rs`
- Add platform-specific config in `frontend/src-tauri/src/audio/devices/configuration.rs`

**New audio capture backend:**
- Add to `frontend/src-tauri/src/audio/capture/`
- Register backend in `frontend/src-tauri/src/audio/capture/backend_config.rs`

**New Tauri command:**
- Define `#[tauri::command] async fn ...` in `frontend/src-tauri/src/lib.rs` (or in a feature-specific file like `audio/recording_commands.rs`)
- Register it in `tauri::Builder::invoke_handler(tauri::generate_handler![...])` inside `lib.rs`
- Call from frontend via `invoke('your_command', { ... })` in `frontend/src/services/` or directly in components

**New React component:**
- Single-file component → `frontend/src/components/YourComponent.tsx` (PascalCase)
- Feature group → `frontend/src/components/YourFeature/` with `index.tsx`
- Route-private → `frontend/src/app/your-route/_components/`

**New page/route:**
- `frontend/src/app/your-route/page.tsx`
- Use `kebab-case` for the folder name

**New Tauri-invoked service:**
- Wrap the `invoke()` call in `frontend/src/services/yourService.ts`
- Add types to `frontend/src/types/`

**New backend endpoint:**
- Add route handler to `backend/app/main.py` with appropriate Pydantic models
- Add DB methods to `backend/app/db.py` `DatabaseManager` class
- Update `backend/app/schema_validator.py` if schema changes

**New LLM provider:**
- Create `frontend/src-tauri/src/your_provider/` mirroring the structure of `frontend/src-tauri/src/anthropic/` or `ollama/`
- Add corresponding integration in `backend/app/main.py` `SummaryProcessor`

**New whisper engine feature:**
- Add to `frontend/src-tauri/src/whisper_engine/`
- Expose Tauri commands via `frontend/src-tauri/src/whisper_engine/commands.rs`

**Shared TypeScript utility:**
- `frontend/src/lib/yourUtil.ts` (camelCase)

**Shared TypeScript type:**
- `frontend/src/types/yourType.ts`

## Special Directories

**`frontend/src-tauri/target/`:**
- Purpose: Rust build artifacts
- Generated: Yes (by `cargo build`)
- Committed: No (in `.gitignore`)

**`frontend/.next/`:**
- Purpose: Next.js build cache
- Generated: Yes (by `next dev` / `next build`)
- Committed: No

**`frontend/node_modules/`:**
- Purpose: pnpm-installed JS deps
- Generated: Yes (by `pnpm install`)
- Committed: No

**`frontend/models/`:**
- Purpose: Whisper model files (development)
- Generated: Downloaded via `download-ggml-model.sh`
- Committed: No

**`backend/whisper.cpp/`:**
- Purpose: Whisper C++ source as git submodule
- Generated: No (managed by `.gitmodules`)
- Committed: As submodule reference

**`backend/whisper-server-package/models/`:**
- Purpose: Whisper model storage (backend-side)
- Generated: Downloaded by `build_whisper.sh` / `download-ggml-model.sh`
- Committed: No

**Production model storage (runtime):**
- macOS: `~/Library/Application Support/Meetily/models/`
- Windows: `%APPDATA%\Meetily\models\`

**`.planning/`:**
- Purpose: GSD planning workspace, including this codebase analysis
- Committed: Conventionally yes (planning artifacts)

**`docs/superpowers/specs/`:**
- Purpose: Design specifications (e.g., `2026-04-07-arabic-bilingual-support-design-v2.md`)
- Committed: Yes

**`.remember/`:**
- Purpose: Local agent memory store
- Committed: Convention varies

**Files marked as cleanup candidates** (legacy files left from refactors):
- `frontend/src-tauri/src/audio/core-old.rs`
- `frontend/src-tauri/src/audio/recording_saver_old.rs`
- `frontend/src-tauri/src/audio/recording_commands.rs.backup`
- `frontend/src-tauri/src/lib_old_complex.rs`
- `frontend/build_backup.bat`
- `frontend/vs_buildtools.exe` (binary in repo — review)

---

*Structure analysis: 2026-04-07*
