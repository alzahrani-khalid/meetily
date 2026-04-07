# Meetily

## What This Is

Meetily is a privacy-first AI meeting assistant that captures, transcribes, and summarizes meetings entirely on local infrastructure. It runs as a Tauri desktop app (Rust + Next.js) with an optional FastAPI + SQLite backend for storage and LLM-based summarization. Current milestone extends it from English-only to full bilingual (Arabic + English) support.

## Core Value

Record a meeting, get an accurate transcript and a useful summary — in your own language, without any audio or content leaving the machine. Everything else (UI polish, templates, editor features) is in service of this.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. Inferred from existing codebase as of 2026-04-07. -->

- ✓ Capture microphone + system audio simultaneously (cpal + ScreenCaptureKit / WASAPI / ALSA) — existing
- ✓ Professional audio mixing with RMS-based ducking, clipping prevention, 50ms alignment windows — existing
- ✓ Voice Activity Detection (VAD) filtering to reduce Whisper load by ~70% — existing
- ✓ Local Whisper transcription via whisper.cpp with GPU acceleration (Metal / CoreML / CUDA / Vulkan / CPU fallback) — existing
- ✓ Parakeet transcription provider for English (default for `uiLocale === 'en'`) — existing
- ✓ Meeting summarization via pluggable LLM providers (Ollama local, Claude, Groq, OpenRouter) — existing
- ✓ Meeting templates system with 3-tier resolution (custom → bundled → builtin) and 6 template types — existing
- ✓ SQLite persistence for meetings, transcripts, summaries, settings — existing (FastAPI backend)
- ✓ Cross-platform desktop app (macOS 13+, Windows, Linux) via Tauri 2.x — existing
- ✓ Tauri command + event architecture for Rust ↔ Next.js IPC — existing
- ✓ Onboarding flow with model download (Parakeet by default) — existing
- ✓ BlockNote rich-text editor for summary editing — existing (Inter font, no RTL config today)
- ✓ All audio/transcription processing happens locally; no cloud transcription path — existing

### Active

<!-- Milestone v1.0: Arabic bilingual support. Derived from docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md. -->

- [ ] Single-source-of-truth user preferences model (SQLite `user_preferences` table, Rust module, two Tauri commands) replacing the 4-way split between React state / `localStorage` / Rust process-global / SQLite `settings` table
- [ ] Migrate 6+ recording-path call sites from `get_language_preference_internal()` to `preferences::read()` and delete the startup-desync `useEffect` workaround at `ConfigContext.tsx:215`
- [ ] `next-intl` client-provider i18n framework with `en` + `ar` message catalogues and `<html lang dir>` switch driven by preferences
- [ ] Arabic-capable font loading (Tajawal via `next/font/google`) alongside existing `Source_Sans_3`
- [ ] RTL layout conversion across 286 directional Tailwind hits in 65 `.tsx` files using logical properties (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`, `border-s-*`, `border-e-*`, `rounded-s-*`, `rounded-e-*`, `rtl:space-x-reverse`)
- [ ] ESLint `no-restricted-syntax` rule preventing new physical-direction Tailwind classes
- [ ] Sidebar collapse animation branched on `dir` (logical properties don't exist for `translate-x`)
- [ ] BlockNote RTL spike answering: does 0.36.0 support `dir="rtl"`, can it be forced via CSS, does Inter break Arabic glyphs, does `@blocknote/shadcn` expose a `dictionary` prop
- [ ] Arabic transcription via Whisper large-v3 only (smaller models hallucinate in Arabic)
- [ ] Parakeet ban for Arabic enforced as a policy invariant inside `set_user_preferences` — invalid state unrepresentable
- [ ] Onboarding fork: `ui_locale === 'ar'` downloads Whisper large-v3 instead of Parakeet, non-blocking with a "ready to record" gate
- [ ] `TranscriptSettings` filters Parakeet out when `uiLocale === 'ar'` and shows an explanatory banner
- [ ] Extend `load_bundled_template` / `load_custom_template` with locale-suffix resolution: try `{id}.{locale}.json` first, fall back to `{id}.json`, at every tier
- [ ] Fix pre-existing bug: `defaults.rs` only embeds 2 of 6 templates; embed all 12 (6 EN + 6 AR)
- [ ] Author 6 Arabic meeting templates (`daily_standup.ar.json` etc.) as native MSA, not machine-translated
- [ ] Externalize 5 hardcoded LLM prompts (`processor.rs:215, 216, 281, 282, 316`) to `frontend/src-tauri/prompts/*.txt` with locale-suffix resolution mirroring the template loader
- [ ] Author 5 Arabic LLM prompts (MSA output, Arabic punctuation `،` `؛` `؟`, same chunk → combine → report structure)
- [ ] Rust-side tray menu and notification strings (~23 total) hydrated from `preferences::read().ui_locale` at startup; re-hydrated on preference change via Tauri event
- [ ] Preference desync regression tests (startup ordering, runtime switch, Parakeet-ban atomicity, concurrent setters)
- [ ] Parakeet ban enforcement tests (onboarding never downloads Parakeet for `ar`, settings modal doesn't render it, invalid configs rejected)
- [ ] Template/prompt locale fallback tests (AR present → AR, AR missing → EN fallback, both missing → error)
- [ ] Manual RTL regression pass on Sidebar / Settings / Transcript / Summary / Onboarding / Tray / Meeting Details with Arabic strings
- [ ] Transcription + summary quality spot-check with MSA audio samples and 2+ LLM providers (Claude + Ollama)

### Out of Scope

<!-- Explicit exclusions. From spec §9 and §12 decisions. -->

- Dialectal Arabic — MSA only; dialects vary too much for Whisper and we ship one quality bar
- Mixed-language / code-switching transcription — Whisper handles poorly, scope creep risk, deferred indefinitely
- Arabic-specific Whisper fine-tuning — out-of-scope custom ML work; use large-v3 as-is
- Custom Arabic keyboard input handling — the OS already does this correctly
- Per-component language switching — whole app switches at once; avoids a combinatorial testing matrix
- Hot-swap mid-session locale switching — triggers full app reload instead (§12.4); simpler invariants for `NextIntlClientProvider` + `dir` + recording-path state
- Dedicated Arabic onboarding flow for users who don't speak English — detected via `navigator.language` with an English fallback (§12.1)
- BlockNote Arabic editing if P2 spike finds blockers — fallback is read-only rendered Arabic summary; editing closed in a follow-up milestone (§12.3)
- Backend Python FastAPI i18n — backend is developer-facing (Swagger, logs); not user-visible
- Log and error message translation — developer-facing, remain in English

## Context

**Codebase state as of 2026-04-07 (verified in spec):**

- Root layout is already `'use client'` at `frontend/src/app/layout.tsx:1`, so `next-intl` must use client-provider mode, not server components
- User preferences currently live in **four** unreconciled places: React state + `localStorage` (`ConfigContext.tsx:140`), Rust process-global (`lib.rs:376, 386`), SQLite `settings` table (provider/model only, no language column), and onboarding status JSON. The `useEffect` at `ConfigContext.tsx:215` exists specifically as a "fixes startup desync bug" workaround — this milestone deletes the cause, which deletes the workaround
- 286 directional Tailwind occurrences across 65 `.tsx` files; top 10 hotspots account for 146 / 286 (51%) and include `Sidebar/index.tsx` (27), `ModelSettingsModal.tsx` (21), `AnalyticsDataModal.tsx` (16), `AISummary/index.tsx` (16), `WhisperModelManager.tsx` (12), `ChunkProgressDisplay.tsx` (12), `SettingsModal.tsx` (11), `dropdown-menu.tsx` (11), `MeetingDetails/SummaryPanel.tsx` (11), `ImportAudio/ImportAudioDialog.tsx` (9)
- Parakeet is defaulted at `ConfigContext.tsx:110` and hardcoded at `OnboardingContext.tsx:8, 424`; `TranscriptSettings.tsx:124` labels it "Recommended". Parakeet TDT variants don't support Arabic, so Arabic forces `localWhisper` with `large-v3`
- Meeting templates: 6 JSON files exist at `frontend/src-tauri/templates/` but `summary/templates/defaults.rs:7` embeds only 2 (`DAILY_STANDUP`, `STANDARD_MEETING`). Pre-existing bug fixed in this milestone
- LLM prompts: 5 prompt strings hardcoded at `frontend/src-tauri/src/summary/processor.rs:215, 216, 281, 282, 316` (chunk system, chunk user template, combine system, combine user template, final report system)
- BlockNote 0.36.0 imports `@blocknote/core/fonts/inter.css` at `Editor.tsx:6` — Inter has no Arabic glyphs. `Editor.tsx:25` calls `useCreateBlockNote({ initialContent })` with no `dictionary`, no `dir`, no custom CSS. Spec §7 explicitly demotes RTL BlockNote to a spike rather than an assumption
- `.planning/codebase/` was mapped on 2026-04-07 (`STACK.md`, `ARCHITECTURE.md`, `STRUCTURE.md`, `CONVENTIONS.md`, `INTEGRATIONS.md`, `TESTING.md`, `CONCERNS.md`)

**Design evolution:**

- v1 spec (`2026-04-06-arabic-bilingual-support-design.md`) was directionally correct but had 4 verified mismatches against main: unreconciled preference storage, Parakeet-as-tooltip framing, template folder duplication (stale against existing loader), and underestimated RTL effort (~110 files vs actual 286 hits / 65 files)
- v2 spec (current, dated 2026-04-07) fixes each with file-grounded decisions

**Tech debt intersecting with this work:**

- Preferences desync is currently patched with a `useEffect` workaround — this milestone eliminates the cause
- Audio pipeline v2 is half-built (unrelated but in same repo; do not touch in this milestone)
- `CORS + plaintext API keys` concern noted separately; not in scope here

## Constraints

- **Tech stack**: Tauri 2.x + Next.js 14 + React 18 on the desktop app, Rust for audio + whisper.cpp + Tauri backend, Python FastAPI + aiosqlite on the server backend — Established architecture, no framework swaps this milestone
- **i18n library**: `next-intl` in client-provider mode only (`NextIntlClientProvider`) — Root layout is already `'use client'`; server-component flavor is incompatible
- **Font loading**: Tajawal via `next/font/google` — Matches existing `Source_Sans_3` loading mechanism at `layout.tsx:30`; keeps build predictable
- **Preferences store**: SQLite `user_preferences` table is the single source of truth — Kills the 4-way split described in Context; enforces atomicity for the Parakeet-ban invariant
- **Arabic transcription model**: `localWhisper` provider with `large-v3` only, GPU-accelerated where available — Smaller Whisper models hallucinate on Arabic; Parakeet TDT has no Arabic support at all
- **Privacy**: All transcription, audio processing, and LLM inference paths must remain capable of running fully locally (Ollama) — Core value; cloud providers (Claude, Groq) are optional opt-ins, not required
- **Platform audio**: ScreenCaptureKit (macOS 13+), WASAPI (Windows), ALSA/PulseAudio (Linux) — Existing capture stack, unchanged
- **RTL tooling**: Tailwind logical properties (`ms-*`, `pe-*`, `start-*`, `text-end`, `border-s-*`, `rounded-e-*`, `rtl:space-x-reverse`) are the ONLY accepted RTL primitive — Matches `~/.claude/CLAUDE.md` Web/Tailwind rules; ESLint rule prevents regressions
- **BlockNote version**: 0.36.0 locked for this milestone — Editor RTL is a spike, not an assumption; upgrade is out of scope
- **Arabic translation authoring**: MSA only, written by native speakers, not machine-translated — Translation quality directly affects trust in the summary
- **Desync workaround deletion**: `ConfigContext.tsx:215` `useEffect` is removed in the same commit as the preferences migration — Don't leave dead workarounds; the comment lies once the cause is gone

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Full bilingual (AR + EN), not AR-only | Preserve existing English user base | — Pending |
| `next-intl` client-only mode (not server components) | Root layout is already `'use client'`; `NextIntlClientProvider` at `layout.tsx` wraps the existing tree | — Pending |
| Single SQLite `user_preferences` row + Rust `preferences` module as source of truth | Kills 4-way desync; enforces atomicity for Parakeet-ban invariant; deletes `ConfigContext.tsx:215` workaround | — Pending |
| Tailwind logical properties + `dir={locale==='ar'?'rtl':'ltr'}` on `<html>` | Matches project-wide RTL rule; no manual flexDirection flipping; no `.reverse()` | — Pending |
| Tajawal font via `next/font/google` | Same loading mechanism as `Source_Sans_3`; stable build | — Pending |
| Whisper `large-v3` is the only Arabic transcription model | Smaller models hallucinate; Parakeet has no Arabic support | — Pending |
| Parakeet ban for Arabic enforced in `set_user_preferences`, not in UI | Invalid state unrepresentable; no UI code needs to "remember" the rule | — Pending |
| Summary language independent from UI locale | An English-speaking team might want Arabic summaries for an Arabic meeting (or vice versa); already correct in v1 | — Pending |
| LLM prompts externalized to `frontend/src-tauri/prompts/*.txt` with locale suffix | Currently hardcoded in `processor.rs`; externalize once instead of translating inline every tuning pass | — Pending |
| Template locale resolution via filename suffix inside existing loader, not folder duplication | Reuses `custom → bundled → builtin` fallback chain; no loader rewrite | — Pending |
| Fix pre-existing `defaults.rs` 2/6 embed gap in this milestone | Discovered while adding Arabic templates; leaving it half-fixed is worse than fixing both | — Pending |
| BlockNote RTL demoted to a P2 spike | v1 assumed "Radix is RTL-aware" but this is not verified; Inter font has no Arabic glyphs | — Pending |
| BlockNote fallback if spike fails: read-only rendered Arabic summary (§12.3) | Ships Arabic without blocking on editor work; editor gap closed in a follow-up | — Pending |
| Initial locale from `navigator.language` with `en` fallback (§12.1) | No extra onboarding step; matches user expectation on first launch | — Pending |
| Arabic onboarding non-blocking with a "ready to record" gate (§12.2) | Whisper large-v3 is ~3GB vs Parakeet ~600MB; blocking onboarding is bad UX on slow networks | — Pending |
| Locale switching mid-session triggers a full reload, not hot-swap (§12.4) | `NextIntlClientProvider` + `dir` + recording-path invariants are all easier to reason about on a fresh mount | — Pending |
| ESLint `no-restricted-syntax` rule banning physical Tailwind direction classes in new code | RTL regressions are easy to introduce; a lint error catches them at PR time | — Pending |
| v1 spec (2026-04-06) superseded by v2 (2026-04-07) | v2 verified against main, fixes 4 concrete mismatches; v1 no longer the reference | ✓ Good |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-07 after initialization*
