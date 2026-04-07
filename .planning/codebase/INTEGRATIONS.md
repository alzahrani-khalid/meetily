# External Integrations

**Analysis Date:** 2026-04-07

## Service Topology

Meetily runs three local processes that communicate over loopback HTTP plus Tauri IPC:

| Service | Default URL | Source |
|---|---|---|
| Whisper.cpp HTTP server | `http://localhost:8178` | `backend/whisper-custom/server/server.cpp`, `backend/docker-compose.yml` env `WHISPER_PORT=8178` |
| FastAPI backend (meetily) | `http://localhost:5167` | `backend/app/main.py`, `frontend/src-tauri/src/api/api.rs` const `APP_SERVER_URL = "http://localhost:5167"`, `frontend/src/components/Sidebar/SidebarProvider.tsx` (`setServerAddress('http://localhost:5167')`) |
| Next.js dev server | `http://localhost:3118` | `frontend/package.json` script `"dev": "next dev -p 3118"`, `frontend/src-tauri/tauri.conf.json` `devUrl` |
| Ollama (external local LLM) | `http://localhost:11434` | `frontend/src-tauri/config/backend_config.json` `ollamaEndpoint`, `backend/app/transcript_processor.py` env `OLLAMA_HOST` |

The Tauri CSP in `frontend/src-tauri/tauri.conf.json` whitelists exactly:
`http://localhost:11434 http://localhost:5167 http://localhost:8178 https://api.ollama.ai`.

## LLM Providers

LLM access is dual-stacked: the Rust desktop process can call providers directly via `frontend/src-tauri/src/summary/llm_client.rs`, while the Python backend uses `pydantic-ai` agents in `backend/app/transcript_processor.py`. Provider keys are stored in the local SQLite `settings` table (`backend/app/db.py`) and exchanged through the FastAPI endpoints below.

**Anthropic Claude:**
- Endpoint: `https://api.anthropic.com/v1/messages` (Rust: `frontend/src-tauri/src/summary/llm_client.rs:195`).
- Model listing: `https://api.anthropic.com/v1/models` (`frontend/src-tauri/src/anthropic/anthropic.rs:103`).
- Backend SDK: `pydantic_ai.models.AnthropicModel` via `AnthropicProvider` (`backend/app/transcript_processor.py:114-116`).
- Provider key in DB: `anthropicApiKey` column (`backend/app/db.py:590`); provider name `"claude"`.
- Env var fallback: `ANTHROPIC_API_KEY` (referenced in `transcript_processor.py:115` error message).
- Rust module: `frontend/src-tauri/src/anthropic/anthropic.rs`.

**OpenAI:**
- Endpoint: `https://api.openai.com/v1/chat/completions` (`frontend/src-tauri/src/summary/llm_client.rs:153`).
- Model listing: `https://api.openai.com/v1/models` (`frontend/src-tauri/src/openai/openai.rs:130`).
- Backend SDK: `pydantic_ai.models.OpenAIModel` via `OpenAIProvider` (`backend/app/transcript_processor.py:140-142`).
- Provider key in DB: `openaiApiKey` column (`backend/app/db.py:588`); default model seeded to `gpt-4o-2024-11-20` (`backend/app/db.py:613`).
- Env var fallback: `OPENAI_API_KEY`.
- Rust module: `frontend/src-tauri/src/openai/openai.rs`.

**Groq:**
- Endpoint: `https://api.groq.com/openai/v1/chat/completions` (`frontend/src-tauri/src/summary/llm_client.rs:157`).
- Model listing: `https://api.groq.com/openai/v1/models` (`frontend/src-tauri/src/groq/groq.rs:98`).
- Backend SDK: `pydantic_ai.models.GroqModel` via `GroqProvider` (`backend/app/transcript_processor.py:134-136`).
- Provider key in DB: `groqApiKey` column (`backend/app/db.py:592`).
- Env var fallback: `GROQ_API_KEY`.
- Rust module: `frontend/src-tauri/src/groq/groq.rs`.

**OpenRouter:**
- Endpoint: `https://openrouter.ai/api/v1/chat/completions` (`frontend/src-tauri/src/summary/llm_client.rs:161`).
- Model listing: `https://openrouter.ai/api/v1/models` (`frontend/src-tauri/src/openrouter/openrouter.rs:45`).
- Rust module: `frontend/src-tauri/src/openrouter/openrouter.rs`.
- Used as the OpenAI-compatible aggregator path from the desktop app; not currently wired into `pydantic-ai` in the backend.

**Ollama (local):**
- Default endpoint: `http://localhost:11434` (`frontend/src-tauri/src/ollama/ollama.rs:163,282,441`, `frontend/src-tauri/src/ollama/metadata.rs:145`, `frontend/src-tauri/src/summary/llm_client.rs:167`, `frontend/src-tauri/config/backend_config.json:3`).
- Backend client: `ollama==0.5.2` Python SDK invoked from `backend/app/transcript_processor.py:258` with env override `OLLAMA_HOST` (defaulting to `http://127.0.0.1:11434`).
- Docker: `OLLAMA_HOST=${OLLAMA_HOST:-http://host.docker.internal:11434}` set on `meetily-backend` and `meetily-backend-macos` services in `backend/docker-compose.yml` (with `extra_hosts: host.docker.internal:host-gateway`).
- Frontend hosted endpoint reference: `https://api.ollama.ai` (CSP whitelisted in `frontend/src-tauri/tauri.conf.json`).
- User-facing endpoint override UI: `frontend/src/components/ModelSettingsModal.tsx:1132,1147,1201` (placeholder `http://localhost:11434`).
- Rust modules: `frontend/src-tauri/src/ollama/ollama.rs`, `frontend/src-tauri/src/ollama/metadata.rs`.

## FastAPI Backend Endpoints (`backend/app/main.py`)

Base URL: `http://localhost:5167`. Docs at `/docs` (Swagger) and `/redoc`. CORS allows all origins for development. Frontend client lives at `frontend/src-tauri/src/api/api.rs` (Rust) and is exposed to the React UI through Tauri commands plus direct fetch in `frontend/src/components/Sidebar/SidebarProvider.tsx`.

| Method | Path | Source line |
|---|---|---|
| GET | `/get-meetings` | `backend/app/main.py:172` |
| GET | `/get-meeting/{meeting_id}` | `backend/app/main.py:182` |
| POST | `/save-meeting-title` | `backend/app/main.py:196` |
| POST | `/delete-meeting` | `backend/app/main.py:206` |
| POST | `/process-transcript` | `backend/app/main.py:329` |
| GET | `/get-summary/{meeting_id}` | `backend/app/main.py:368` |
| POST | `/save-transcript` | `backend/app/main.py:511` |
| GET | `/get-model-config` | `backend/app/main.py:550` |
| POST | `/save-model-config` | `backend/app/main.py:560` |
| GET | `/get-transcript-config` | `backend/app/main.py:568` |
| POST | `/save-transcript-config` | `backend/app/main.py:578` |
| POST | `/get-api-key` | `backend/app/main.py:589` |
| POST | `/get-transcript-api-key` | `backend/app/main.py:596` |
| POST | `/save-meeting-summary` | `backend/app/main.py:607` |
| POST | `/search-transcripts` | `backend/app/main.py:623` |

Health check used by Docker: `curl -f http://localhost:5167/get-meetings` (`backend/docker-compose.yml`).

## Whisper.cpp HTTP Server (`backend/whisper-custom/server/server.cpp`)

- Port: `8178` (env `WHISPER_PORT`, default in `backend/docker-compose.yml`).
- Bind host: `WHISPER_HOST=0.0.0.0` in containers.
- Streaming endpoint: `POST http://localhost:8178/stream` (referenced in `frontend/API.md:188` and called from the Rust audio pipeline).
- Health check: `curl -f http://localhost:8178/` (`backend/docker-compose.yml`).
- Build script: `backend/build_whisper.sh` / `backend/build_whisper.cmd` (compiles `whisper-custom/server`).
- Start scripts: `backend/start_whisper_server.cmd`, `backend/start_with_output.ps1`.
- CMake project: `backend/whisper-custom/server/CMakeLists.txt`.

**Container env vars (`backend/docker-compose.yml` whisper-server):**
- `WHISPER_HOST=0.0.0.0`
- `WHISPER_PORT=8178`
- `WHISPER_MODEL=${WHISPER_MODEL:-models/ggml-base.en.bin}` (macOS profile defaults to `ggml-large-v3.bin`)
- `WHISPER_THREADS=${WHISPER_THREADS:-0}`
- `WHISPER_USE_GPU=${WHISPER_USE_GPU:-true}` (macOS profile forces `false`)
- `WHISPER_LANGUAGE=${WHISPER_LANGUAGE:-en}`
- `WHISPER_TRANSLATE=${WHISPER_TRANSLATE:-false}`
- `WHISPER_DIARIZE=${WHISPER_DIARIZE:-false}`
- `WHISPER_PRINT_PROGRESS=${WHISPER_PRINT_PROGRESS:-true}`
- `WHISPER_PLATFORM=macos` (macOS profile only)

**Model downloader** (`backend/docker-compose.yml` `model-downloader` service): pulls `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-${MODEL_NAME}.bin` into the `whisper_models` named volume. Driven by `MODEL_NAME` env (default `base.en`).

**Local model paths:**
- Development: `frontend/models/` and `backend/whisper-server-package/models/`.
- Production macOS: `~/Library/Application Support/Meetily/models/`.
- Production Windows: `%APPDATA%\Meetily\models\`.

## In-Process Whisper Engine (Rust)

The desktop app does **not** require the HTTP whisper-server when running locally; it links Whisper.cpp directly via `whisper-rs`.

- Provider implementation: `frontend/src-tauri/src/audio/transcription/whisper_provider.rs`.
- Worker pool: `frontend/src-tauri/src/audio/transcription/worker.rs`.
- Engine entry: `frontend/src-tauri/src/audio/transcription/engine.rs`.
- Crate: `whisper-rs = 0.13.2` with target-specific GPU features (see STACK.md).
- Parallel processor commands: `frontend/src-tauri/src/whisper_engine/parallel_commands.rs`.

**Alternate transcription provider:**
- Parakeet (ONNX): `frontend/src-tauri/src/audio/transcription/parakeet_provider.rs`, JS wrapper `frontend/src/lib/parakeet.ts`. Powered by `ort = 2.0.0-rc.10`.
- Built-in browser AI fallback: `frontend/src/lib/builtin-ai.ts`.

## Tauri IPC Bridge (Next.js ↔ Rust)

Tauri 2.6.2 provides the only safe channel between the Next.js renderer (port 3118 in dev, bundled assets in production) and the Rust backend.

- Frontend wrappers: `@tauri-apps/api ^2.6.0` (`invoke`, `listen`, `emit`) used throughout `frontend/src/lib/*.ts` and `frontend/src/components/Sidebar/SidebarProvider.tsx`.
- Rust command registration: `frontend/src-tauri/src/lib.rs` (14 `#[tauri::command]` declarations) plus 178 commands across 19 modules including:
  - `frontend/src-tauri/src/api/api.rs` - Backend HTTP proxy commands.
  - `frontend/src-tauri/src/audio/recording_commands.rs` - `start_recording`, recording lifecycle.
  - `frontend/src-tauri/src/audio/permissions.rs`, `frontend/src-tauri/src/audio/import.rs`, `frontend/src-tauri/src/audio/retranscription.rs`, `frontend/src-tauri/src/audio/recording_preferences.rs`, `frontend/src-tauri/src/audio/incremental_saver.rs`.
  - `frontend/src-tauri/src/database/commands.rs` (sqlx-backed local DB).
  - `frontend/src-tauri/src/summary/commands.rs`, `frontend/src-tauri/src/summary/summary_engine/commands.rs`, `frontend/src-tauri/src/summary/template_commands.rs`.
  - `frontend/src-tauri/src/notifications/commands.rs`.
  - `frontend/src-tauri/src/console_utils/console_utils.rs`.
  - `frontend/src-tauri/src/whisper_engine/parallel_commands.rs`.
  - `frontend/src-tauri/src/onboarding.rs`.
  - `frontend/src-tauri/src/analytics/commands.rs`.
- Event channel: Rust emits `transcript-update` and similar events consumed by React via `listen<T>()`.

**Tauri capabilities (`frontend/src-tauri/tauri.conf.json` capabilities.main.permissions):**
`fs:default`, `fs:allow-read-file`, `fs:read-all`, `fs:write-all`, `fs:allow-app-read`, `fs:allow-app-write`, `fs:allow-download-write`, `fs:allow-download-read`, `fs:scope-download`, `core:path:default`, `core:event:default`, `core:window:default`, `core:app:default`, `core:resources:default`, `core:menu:default`, `core:tray:default`, `core:window:allow-set-title`, `store:default`, `notification:default`, `notification:allow-is-permission-granted`, `updater:default`, `process:default`, plus a custom `fs:scope` allowlist for `$APPDATA/*`.

**Asset protocol:** Enabled with scope `$APPDATA/**` for serving recorded audio and model files into the WebView.

## Data Storage

**Backend SQLite:**
- Driver: `aiosqlite==0.21.0`.
- Manager: `backend/app/db.py` (`DatabaseManager` class).
- Container path: `/app/data/meeting_minutes.db` (env `DATABASE_PATH=/app/data/meeting_minutes.db` in `backend/docker-compose.yml`).
- Local setup helpers: `backend/setup-db.sh`, `backend/setup-db.ps1`.
- Tables include `settings` with columns: `id`, `provider`, `model`, `whisperModel`, `openaiApiKey`, `anthropicApiKey`, `groqApiKey`, `ollamaApiKey` (`backend/app/db.py:582-637`).
- Persisted to host volume `./data` mounted into the `meetily-backend` containers.

**Tauri-side SQLite (sqlx):**
- Crate: `sqlx = 0.8` with `sqlite + chrono` features.
- Modules: `frontend/src-tauri/src/database/manager.rs`, `frontend/src-tauri/src/database/setup.rs`, `frontend/src-tauri/src/database/models.rs`.
- Repositories: `frontend/src-tauri/src/database/repositories/{meeting,setting,summary,transcript,transcript_chunk}.rs`.

**File Storage:**
- Audio recordings written via `frontend/src-tauri/src/audio/recording_saver.rs` and `frontend/src-tauri/src/audio/incremental_saver.rs` to platform `$APPDATA` paths managed through `dirs = 5.0.1`.
- Whisper model storage: see paths above.
- Container volumes: `whisper_models`, `whisper_uploads`, `meeting_app_logs` (named volumes in `backend/docker-compose.yml`).

**Caching:** None — audio buffers held in-memory ring buffers (`frontend/src-tauri/src/audio/buffer_pool.rs`, `frontend/src-tauri/src/audio/pipeline.rs`).

## Authentication & Identity

- No multi-user auth. Single local user per machine.
- LLM provider API keys stored in local SQLite (`settings` table) and protected only by OS file permissions on `meeting_minutes.db`.
- macOS: code-signed with ad-hoc identity `-` (`frontend/src-tauri/tauri.conf.json` `bundle.macOS.signingIdentity`).
- Windows: signed via `frontend/src-tauri/scripts/sign-windows.ps1` (referenced from `tauri.conf.json` `bundle.windows.signCommand`).

## Audio Platform Integrations

**macOS - Apple ScreenCaptureKit (system audio) + CoreAudio (microphone):**
- Bridge crate: `cidre` (git rev `a9587fa`, features `["av"]`) - exposes Apple AVFoundation/ScreenCaptureKit Objective-C APIs to Rust.
- Native frameworks: `core-graphics = 0.23`, `objc = 0.2.7`.
- Implementation files:
  - `frontend/src-tauri/src/audio/capture/core_audio.rs`
  - `frontend/src-tauri/src/audio/devices/platform/macos.rs`
  - `frontend/src-tauri/src/audio/system_audio_stream.rs`
  - `frontend/src-tauri/src/audio/system_detector.rs`
- Requires macOS 13+ and both microphone + screen recording permissions.
- Recommended virtual loopback device: BlackHole 2ch.

**Windows - WASAPI (Windows Audio Session API):**
- Loopback capture for system audio handled through `cpal`'s WASAPI backend.
- Implementation: `frontend/src-tauri/src/audio/devices/platform/windows.rs`, `frontend/src-tauri/src/audio/system_audio_stream.rs`, `frontend/src-tauri/src/audio/devices/speakers.rs`.

**Linux - ALSA / PulseAudio:**
- Backend selected automatically by `cpal`.
- Implementation: `frontend/src-tauri/src/audio/devices/platform/linux.rs`.
- Requires PulseAudio or ALSA system packages on the host.

**Cross-platform pipeline:**
- Capture: `frontend/src-tauri/src/audio/capture/microphone.rs`, `frontend/src-tauri/src/audio/capture/system.rs`.
- Mixing + VAD: `frontend/src-tauri/src/audio/pipeline.rs` (RMS ducking), `frontend/src-tauri/src/audio/vad.rs` (silero), noise suppression via `nnnoiseless`.
- Loudness normalization: `ebur128 = 0.1`.
- Resampling: `rubato = 0.15.0`.
- Decoding: `symphonia = 0.5.4` (multi-format).
- ffmpeg sidecar: `ffmpeg-sidecar` crate plus bundled binary `frontend/src-tauri/binaries/ffmpeg` (declared in `tauri.conf.json` `bundle.externalBin`).

## Telemetry & Analytics

**PostHog (desktop app):**
- Crate: `posthog-rs = 0.3.7`.
- Implementation: `frontend/src-tauri/src/analytics/analytics.rs`, `frontend/src-tauri/src/analytics/commands.rs`.
- Frontend wrapper: `frontend/src/lib/analytics.ts`.
- API key configuration: not present in checked-in source — initialized at runtime from build constants or environment (existence of `.env` files noted, contents not read).

**Logging:**
- Rust: `log = 0.4`, `env_logger = 0.11`, `tracing = 0.1.40`, `tauri-plugin-log = 2.6.0` (macOS only).
- Performance logging macros `perf_debug!` / `perf_trace!` in `frontend/src-tauri/src/lib.rs` (zero-cost in release builds).
- Backend: `uvicorn` formatted logs `YYYY-MM-DD HH:MM:SS - LEVEL - [file:line - func()] - message`.
- Container log driver: `json-file` with rotation `max-size: 10m`, `max-file: 3` (`backend/docker-compose.yml`).

## Updates & Distribution

**Tauri Updater (`frontend/src-tauri/tauri.conf.json` plugins.updater):**
- Endpoint: `https://github.com/Zackriya-Solutions/meeting-minutes/releases/latest/download/latest.json`.
- Public key: minisign key embedded as base64 (`pubkey` field).
- Crate: `tauri-plugin-updater = 2.3.0`.

**External binaries bundled (`tauri.conf.json` bundle.externalBin):**
- `binaries/llama-helper` - Built from workspace member `llama-helper/`.
- `binaries/ffmpeg` - Bundled FFmpeg sidecar.

## CI / CD

- GitHub Actions workflows under `.github/` (existence noted).
- Local distribution builds invoked via `frontend/clean_build.sh` / `frontend/clean_build_windows.bat` and `frontend/package-app.sh`.
- Docker images: `Dockerfile.app`, `Dockerfile.server-cpu`, `Dockerfile.server-gpu`, `Dockerfile.server-macos` in `backend/`. Build helpers `backend/build-docker.sh`, `backend/build-docker.ps1`.

## Environment Variables

Confirmed via source. The presence of `backend/temp.env`, `backend/app/.env`, and similar files is noted; contents are not read.

**Backend (`backend/app/main.py`, `backend/app/transcript_processor.py`, `backend/docker-compose.yml`):**
- `DATABASE_PATH` - SQLite path (default `/app/data/meeting_minutes.db` in containers).
- `OLLAMA_HOST` - Ollama base URL (default `http://localhost:11434` locally, `http://host.docker.internal:11434` in Docker).
- `PYTHONUNBUFFERED=1`, `PYTHONPATH=/app` - Container runtime hints.
- `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `GROQ_API_KEY` - Fallback keys when DB-stored keys missing.
- `APP_PORT` - Override for FastAPI port (default `5167`).

**Whisper server container (`backend/docker-compose.yml`):**
- `WHISPER_HOST`, `WHISPER_PORT`, `WHISPER_MODEL`, `WHISPER_THREADS`, `WHISPER_USE_GPU`, `WHISPER_LANGUAGE`, `WHISPER_TRANSLATE`, `WHISPER_DIARIZE`, `WHISPER_PRINT_PROGRESS`, `WHISPER_PLATFORM`, `MODEL_NAME`.
- Build args: `CUDA_VERSION`, `UBUNTU_VERSION`, `DOCKERFILE`, `TAG`.
- Volume / path overrides: `LOCAL_MODELS_DIR`, `CONFIG_DIR`, `LOCAL_ENV_FILE`, `WEB_PORT`.

**Frontend / Tauri build:**
- `RUST_LOG` - Logging filter (e.g. `RUST_LOG=app_lib::audio=debug`).
- `BLAS_INCLUDE_DIRS` - Required when building with `--features openblas` on Windows/Linux.

## Webhooks & Callbacks

- **Incoming:** None. The FastAPI backend exposes only client-driven REST endpoints listed above.
- **Outgoing:** None. All LLM and Whisper traffic is request/response over HTTPS or local HTTP. The Tauri updater periodically pulls the GitHub releases JSON URL above.

---

*Integration audit: 2026-04-07*
