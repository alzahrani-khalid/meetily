# Technology Stack

**Analysis Date:** 2026-04-07

## Languages

**Primary:**
- Rust 2021 edition (rust-version 1.77) - Tauri desktop backend, audio pipeline, Whisper integration. Configured in `Cargo.toml` and `frontend/src-tauri/Cargo.toml`.
- TypeScript ^5.7.2 - Next.js frontend (React 18). Configured in `frontend/tsconfig.json`.
- Python (3.x via FastAPI 0.115.9) - Backend API and LLM orchestration in `backend/app/main.py`, `backend/app/transcript_processor.py`.

**Secondary:**
- C / C++ - Whisper.cpp custom server vendored in `backend/whisper-custom/server/server.cpp` and `backend/whisper.cpp/`.
- PowerShell / Bash - Build, packaging, and run scripts (`frontend/clean_run.sh`, `frontend/clean_run_windows.bat`, `backend/run-docker.sh`, `backend/run-docker.ps1`, `backend/start_with_output.ps1`).
- JavaScript - Helper scripts (`frontend/scripts/tauri-auto.js`).

## Runtime

**Environment:**
- Node.js (consumer of `next` ^14.2.25). Dev server runs on port 3118 (`frontend/package.json` script `"dev": "next dev -p 3118"`).
- Rust toolchain (workspace defined in `Cargo.toml` with members `frontend/src-tauri` and `llama-helper`).
- Python 3.x with `uvicorn==0.34.0` runtime serving `backend/app/main.py`.
- Tauri 2.6.2 runtime for the desktop shell (`frontend/src-tauri/tauri.conf.json`, `frontend/src-tauri/Cargo.toml`).

**Package Managers:**
- pnpm - Frontend JavaScript dependencies (`frontend/package.json`, `frontend/pnpm-lock.yaml` referenced by `pnpm dev` and `pnpm build` in `tauri.conf.json`).
- Cargo - Rust workspace at root `Cargo.toml`; member crate `frontend/src-tauri/Cargo.toml`.
- pip - Python dependencies pinned in `backend/requirements.txt`.
- Lockfiles: `frontend/pnpm-lock.yaml` and `Cargo.lock` present at workspace root; pip uses pinned versions in `requirements.txt` (no separate lockfile).

## Frameworks

**Core (Frontend Desktop):**
- Tauri 2.6.2 (`tauri = { version = "2.6.2", features = ["macos-private-api", "protocol-asset", "tray-icon"] }`) - Desktop shell, IPC bridge, system tray.
- Next.js ^14.2.25 - React framework, App Router. Configured in `frontend/next.config.js`.
- React ^18.2.0 + react-dom ^18.2.0.
- Tailwind CSS ^3.4.1 with `@tailwindcss/typography` ^0.5.15 (`frontend/tailwind.config.ts`, `frontend/postcss.config.js`).

**Core (Backend):**
- FastAPI 0.115.9 - Async REST API in `backend/app/main.py`.
- Uvicorn 0.34.0 - ASGI server.
- Pydantic 2.11.5 + pydantic-ai 0.2.15 - Schema validation and LLM agent abstraction.
- aiosqlite 0.21.0 - Async SQLite driver used by `backend/app/db.py`.

**UI Component Libraries:**
- Radix UI primitives (`@radix-ui/react-*`: accordion, dialog, dropdown-menu, label, popover, progress, scroll-area, select, separator, slot, switch, tabs, tooltip). Listed in `frontend/package.json`.
- shadcn/ui pattern via `radix-ui` ^1.4.3, `class-variance-authority` ^0.7.1, `clsx` ^2.1.1, `tailwind-merge` ^3.3.1, `tailwindcss-animate` ^1.0.7.
- BlockNote 0.36.0 (`@blocknote/core`, `@blocknote/react`, `@blocknote/shadcn`) - Block editor.
- Remirror ^3.0.1 (`@remirror/core`, extension-bold, extension-italic, extension-list, extension-markdown, extension-mention, extension-underline, `@remirror/pm`, `@remirror/react`) - Rich-text editor.
- Tiptap ^2.10.4 (`@tiptap/pm`, `@tiptap/react`, `@tiptap/starter-kit`) - Alternate rich-text editor.
- `@tanstack/react-virtual` ^3.13.13 - Transcript virtualization.
- `framer-motion` ^11.15.0, `lucide-react` ^0.469.0, `@heroicons/react` ^2.2.0, `cmdk` ^1.1.1, `sonner` ^2.0.7 (toasts).

**Forms & Validation:**
- `react-hook-form` ^7.59.0, `@hookform/resolvers` ^5.1.1, `zod` ^3.25.71.

**Markdown / Formatting:**
- `react-markdown` ^9.0.1, `remark-gfm` ^4.0.1, `date-fns` ^4.1.0.

**Testing:**
- Frontend JS: No dedicated JS test runner registered in `frontend/package.json` scripts (only `lint`).
- Rust: `criterion = { version = "0.5.1", features = ["async_tokio"] }` and `tempfile`, `infer`, `memory-stats`, `strsim`, `futures`, `tracing-subscriber` listed under `[dev-dependencies]` in `frontend/src-tauri/Cargo.toml`.
- Python: No test framework declared in `backend/requirements.txt`.

**Build / Dev:**
- `@tauri-apps/cli` ^2.1.0 - Tauri build orchestrator.
- `tauri-build = { version = "2.3.0" }` - Build script in `frontend/src-tauri/Cargo.toml [build-dependencies]`.
- TypeScript ^5.7.2, `autoprefixer` ^10.0.1, `postcss` ^8.
- `concurrently` ^8.2.2, `wait-on` ^7.2.0 - Dev orchestration.
- `eslint` via `frontend/eslint.config.mjs` (Next lint).

## Tauri Plugins (Rust crates + JS wrappers)

Both halves declared in `frontend/src-tauri/Cargo.toml` and `frontend/package.json`:

- `tauri-plugin-fs = 2.4.0` / `@tauri-apps/plugin-fs ^2.4.0`
- `tauri-plugin-dialog = 2.3.0`
- `tauri-plugin-store = 2.4.0` / `@tauri-apps/plugin-store ^2.4.0`
- `tauri-plugin-notification = 2.3.1` / `@tauri-apps/plugin-notification ~2.3.1`
- `tauri-plugin-updater = 2.3.0` / `@tauri-apps/plugin-updater ^2.3.0`
- `tauri-plugin-process = 2.3.0` / `@tauri-apps/plugin-process ^2.3.0`
- `tauri-plugin-log = 2.6.0` (macOS only target dependency)
- `@tauri-apps/api ^2.6.0`, `@tauri-apps/plugin-os ^2.3.2`

## Rust Crate Dependencies (frontend/src-tauri/Cargo.toml)

**Audio / DSP:**
- `cpal = 0.15.3` (patched from `https://github.com/RustAudio/cpal` rev `51c3b43`) - Cross-platform audio capture.
- `whisper-rs = 0.13.2` - Whisper.cpp bindings. Features differ per target:
  - macOS: `["raw-api", "metal", "coreml"]`
  - Windows: `["raw-api", "vulkan"]`
  - Linux: `["raw-api"]`
- `silero_rs` (git: `https://github.com/emotechlab/silero-rs` rev `26a6460`, package `silero`) - Voice Activity Detection.
- `nnnoiseless = 0.5` - RNNoise neural noise suppression.
- `ebur128 = 0.1` - EBU R128 loudness normalization.
- `rubato = 0.15.0` - Sample rate conversion.
- `realfft = 3.4.0` - FFT for audio analysis.
- `symphonia = 0.5.4` (features `aac`, `isomp4`, `mp3`, `flac`, `ogg`, `vorbis`, `pcm`, `wav`, `opt-simd`) - Audio decoding.
- `ringbuf = 0.4.8`, `dasp = 0.11.0` (macOS), `bytemuck = 1.16.1`.
- `ffmpeg-sidecar` (git: `https://github.com/nathanbabcock/ffmpeg-sidecar` branch `main`).

**Speech / ML:**
- `ort = 2.0.0-rc.10` - ONNX Runtime for Parakeet transcription provider (`frontend/src-tauri/src/audio/transcription/parakeet_provider.rs`).
- `ndarray = 0.16` - Tensor operations for ML inputs.

**Async / Concurrency:**
- `tokio = 1.32.0` (features `full`, `tracing`) + `tokio-util = 0.7`.
- `async-trait = 0.1`, `futures-util = 0.3`, `futures-channel = 0.3.31`.
- `crossbeam = 0.8.4`, `dashmap = 6.1.0`, `rayon = 1.10`.
- `once_cell = 1.17.1`, `lazy_static = 1.4.0`.

**HTTP / Serialization:**
- `reqwest = 0.11` (features `blocking`, `multipart`, `json`, `stream`) - Used to call LLM and Whisper HTTP APIs from Rust.
- `serde = 1.0` (derive), `serde_json = 1.0`, `bytes = 1.9.0`.

**Database / Storage:**
- `sqlx = 0.8` (features `runtime-tokio`, `sqlite`, `chrono`) - Local Tauri-side database (`frontend/src-tauri/src/database/`).
- `dirs = 5.0.1`, `tempfile = 3.3.0`.

**Platform Integration (macOS):**
- `cidre` (git: `https://github.com/yury/cidre` rev `a9587fa`, features `av`) - Apple frameworks bridge for ScreenCaptureKit.
- `core-graphics = 0.23`, `objc = 0.2.7`, `time = 0.3`.

**Telemetry / Logging:**
- `posthog-rs = 0.3.7` - Analytics (used in `frontend/src-tauri/src/analytics/analytics.rs` and `analytics/commands.rs`).
- `log = 0.4`, `env_logger = 0.11`, `tracing = 0.1.40`.

**Error / Utility:**
- `anyhow = 1.0`, `thiserror = 2.0.16`.
- `chrono = 0.4.31` (serde), `uuid = 1.0` (v4, serde).
- `clap = 4.3` (derive), `regex = 1.11.0`, `url = 2.5.0`, `rand = 0.8.5`.
- `sysinfo = 0.32` - System resource monitoring.
- `which = 6.0.1`.
- `esaxx-rs = 0.1.10` (patched git fork `thewh1teagle/esaxx-rs` branch `feat/dynamic-msvc-link`).

**Build dependencies (`frontend/src-tauri/Cargo.toml [build-dependencies]`):**
- `tauri-build = 2.3.0`, `reqwest = 0.11`, `which = 6.0.1`, `zip = 2.2`, `tar = 0.4`, `xz2 = 0.1` (used by build script to fetch/extract Whisper assets).

## Python Dependencies (`backend/requirements.txt`)

- `pydantic-ai==0.2.15` - LLM agent abstraction (Anthropic, OpenAI, Groq, Ollama providers).
- `pydantic==2.11.5` - Schema validation.
- `pandas==2.2.3`, `devtools==0.12.2` - Data utilities and debugging.
- `python-dotenv==1.1.0` - `.env` loading.
- `fastapi==0.115.9`, `uvicorn==0.34.0`, `python-multipart==0.0.20` - API server.
- `aiosqlite==0.21.0` - Async SQLite driver.
- `ollama==0.5.2` - Ollama Python client.

## Configuration Files

**Frontend (Next.js / TS):**
- `frontend/package.json` - Scripts and JS deps.
- `frontend/next.config.js` - Next.js configuration.
- `frontend/tsconfig.json` - TypeScript config.
- `frontend/eslint.config.mjs` - ESLint flat config.
- `frontend/tailwind.config.ts`, `frontend/tailwind.config.js` - Tailwind themes.
- `frontend/postcss.config.js`, `frontend/postcss.config.mjs` - PostCSS pipeline.
- `frontend/components.json` - shadcn/ui registry config.

**Tauri / Rust:**
- `Cargo.toml` (workspace root) - Workspace members and shared deps.
- `frontend/src-tauri/Cargo.toml` - Member crate manifest.
- `frontend/src-tauri/tauri.conf.json` - Tauri window/security/bundle config.
- `frontend/src-tauri/entitlements.plist` - macOS hardened runtime entitlements.
- `frontend/src-tauri/config/backend_config.json` - Default Ollama and FastAPI endpoints (`http://localhost:11434`, `http://localhost:5167`).
- `frontend/src-tauri/build.rs` (referenced by `[build-dependencies]`) - Whisper asset bootstrap.

**Backend (Python):**
- `backend/requirements.txt` - Python pinned deps.
- `backend/docker-compose.yml` - Multi-service orchestration (whisper-server, meetily-backend, model-downloader, web-ui).
- `backend/Dockerfile.app`, `backend/Dockerfile.server-cpu`, `backend/Dockerfile.server-gpu`, `backend/Dockerfile.server-macos` - Container builds.
- `backend/temp.env` - Environment variable scratch file (existence noted, contents not read).
- `backend/app/.env` - Mounted into containers per docker-compose `${LOCAL_ENV_FILE:-./app/.env}` (existence noted, contents not read).
- `backend/set_env.sh` - Environment bootstrap script.

**Build / Run Scripts:**
- `frontend/clean_run.sh`, `frontend/clean_run_windows.bat`, `frontend/clean_build.sh`, `frontend/clean_build_windows.bat`.
- `frontend/build-gpu.sh`, `frontend/build-gpu.ps1`, `frontend/build-gpu.bat`, `frontend/dev-gpu.sh`, `frontend/dev-gpu.ps1`, `frontend/dev-gpu.bat`.
- `frontend/package-app.sh`, `frontend/scripts/tauri-auto.js`.
- `backend/build_whisper.sh`, `backend/build_whisper.cmd`, `backend/clean_start_backend.sh`, `backend/clean_start_backend.cmd`.
- `backend/run-docker.sh`, `backend/run-docker.ps1`, `backend/build-docker.sh`, `backend/build-docker.ps1`.
- `backend/download-ggml-model.sh`, `backend/download-ggml-model.cmd`.
- `backend/install_dependancies_for_windows.ps1`.

## Whisper.cpp Submodules / Vendored Binaries

- `backend/whisper.cpp/` - Vendored upstream whisper.cpp (referenced by `.gitmodules`).
- `backend/whisper-custom/server/` - Custom HTTP server fork (`server.cpp`, `CMakeLists.txt`, `README.md`).
- `frontend/src-tauri/binaries/llama-helper` - External bundled binary defined in `frontend/src-tauri/tauri.conf.json` (`bundle.externalBin`).
- `frontend/src-tauri/binaries/ffmpeg` - Bundled ffmpeg sidecar binary.
- `llama-helper/` - Workspace member crate (root `Cargo.toml`) producing the `llama-helper` binary.

## Cargo Features (GPU acceleration matrix)

Defined in `frontend/src-tauri/Cargo.toml [features]`:

- `default = ["platform-default"]` - Auto-selects best backend by target OS.
- `metal = ["whisper-rs/metal"]`, `coreml = ["whisper-rs/coreml"]` - macOS GPU.
- `cuda = ["whisper-rs/cuda"]` - NVIDIA on Windows/Linux.
- `vulkan = ["whisper-rs/vulkan"]` - AMD/Intel on Windows/Linux.
- `hipblas = ["whisper-rs/hipblas"]` - AMD ROCm on Linux.
- `openblas = ["whisper-rs/openblas"]`, `openmp = ["whisper-rs/openmp"]` - CPU optimization.

Per-target Whisper features (resolved automatically): macOS uses `metal + coreml`, Windows uses `vulkan`, Linux uses base.

## Platform Requirements

**Development:**
- Rust 1.77+, Cargo, Node.js with pnpm.
- Python 3.x with pip.
- macOS: Xcode command-line tools, microphone + screen recording permissions, virtual audio device (BlackHole) for system capture.
- Windows: Visual Studio Build Tools (C++ workload), `vs_buildtools.exe` checked into `frontend/`.
- Linux: cmake, llvm, libomp, ALSA/PulseAudio dev headers.

**Production / Bundling:**
- Tauri bundle targets in `frontend/src-tauri/tauri.conf.json`: `deb`, `appimage`, `msi`, `nsis`, `app`, `dmg`.
- Updater configured with public minisign key and endpoint `https://github.com/Zackriya-Solutions/meeting-minutes/releases/latest/download/latest.json`.
- macOS bundle uses `entitlements.plist`, signing identity `-`, hardened runtime enabled.
- Windows uses `scripts/sign-windows.ps1` for code signing.
- Backend ships via Docker (`backend/Dockerfile.app`) or local Python venv started by `backend/clean_start_backend.sh`.

---

*Stack analysis: 2026-04-07*
