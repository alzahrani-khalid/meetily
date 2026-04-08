# Roadmap: Meetily v1.0 — Arabic Bilingual Support

**Milestone:** v1.0 (Arabic Bilingual)
**Created:** 2026-04-07
**Granularity:** standard
**Source spec:** `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md`
**Requirements source:** `.planning/REQUIREMENTS.md`

## Core Value

Record a meeting, get an accurate transcript and a useful summary — in your own language, without any audio or content leaving the machine.

## Phases

- [ ] **Phase 1: Preferences Foundation** — Single-source-of-truth `user_preferences` SQLite + Rust module with atomic Parakeet-ban invariant
- [ ] **Phase 2: i18n Framework & Locale Bootstrap** — `next-intl` client provider, Tajawal font, `<html lang dir>` switch, locale detection, full-reload locale switching
- [ ] **Phase 3: RTL Layout Conversion** — BlockNote spike, ESLint guardrail, hotspot-first conversion of 286 directional hits across 65 files
- [ ] **Phase 4: Arabic Transcription Policy** — Whisper `large-v3`-only Arabic path, Parakeet ban enforcement at UI + onboarding, non-blocking large-v3 download
- [ ] **Phase 5: Templates & Prompts (Bilingual Content)** — Locale-suffix loader, `defaults.rs` 2/6 embed fix, externalized prompts, 6 AR templates + 5 AR prompts authored in MSA
- [ ] **Phase 6: Rust Strings, QA & Release Hardening** — Tray/notification hydration, automated regression tests, manual RTL pass, Arabic transcription + summary quality spot-checks

## Phase Details

### Phase 1: Preferences Foundation
**Goal**: User preferences live in exactly one place with atomic writes that make invalid states (e.g. Arabic + Parakeet) unrepresentable.
**Depends on**: Nothing (foundation phase — everything else reads from this)
**Requirements**: PREFS-01, PREFS-02, PREFS-03, PREFS-04
**Plans**: 1 plan
**Success Criteria** (what must be TRUE):
  1. User can set their UI locale and have the value survive an app restart with no `useEffect` workaround in `ConfigContext.tsx` rehydrating it
  2. User running a recording immediately after switching `transcription_language` sees the new value honored on the *next* recording (no stale Rust process-global cache)
  3. User attempting to write `provider: 'parakeet'` while `ui_locale === 'ar'` is rejected by `set_user_preferences` before SQLite is touched — invariant enforced inside the same transaction
  4. User cannot observe a window where SQLite and the in-memory `RwLock` disagree (concurrent setter test passes)
**Plans**:
- [x] 01-01-PLAN.md — Preferences module, atomic setter with REAL reject branch (A1 Option B), T1..T5 tests, 4 call-site migration, ConfigContext desync workaround deletion, .backup cleanup (17 tasks across 6 commit-aligned waves)

### Phase 2: i18n Framework & Locale Bootstrap
**Goal**: Users see the app in their language on first launch, with the correct font, document direction, and a clean reload-based language switch.
**Depends on**: Phase 1 (root layout reads `preferences::read().ui_locale` at startup)
**Requirements**: UI-01, UI-02, UI-03, UI-04
**Success Criteria** (what must be TRUE):
  1. User on a system reporting `navigator.language` starting with `ar` launches Meetily for the first time and sees Arabic UI strings (with `en` fallback if detection ambiguous)
  2. User sees Arabic text rendered in Tajawal (loaded via `next/font/google`) and English in Source Sans 3 — neither leaks into the other locale
  3. User opens Settings, switches language, confirms — the app performs a full reload and the new locale is active on the next mount, persisted across subsequent restarts
  4. User on first paint never sees a flash of LTR English chrome before Arabic loads (boot splash gates the provider tree until preferences resolve)
**Plans**: TBD
**UI hint**: yes

### Phase 3: RTL Layout Conversion
**Goal**: Every visible screen mirrors correctly in Arabic with no physical-direction Tailwind classes left in the codebase, and a decision is locked on whether the BlockNote summary editor can be made RTL.
**Depends on**: Phase 2 (`<html dir>` switch and locale hydration must exist before RTL classes can be visually verified)
**Requirements**: UI-05, UI-06, QA-07
**Success Criteria** (what must be TRUE):
  1. User in Arabic mode sees all 10 RTL hotspot files (Sidebar, ModelSettingsModal, AnalyticsDataModal, AISummary, WhisperModelManager, ChunkProgressDisplay, SettingsModal, dropdown-menu, SummaryPanel, ImportAudioDialog — 146 of 286 hits) rendering as proper right-to-left mirrors with no visual asymmetry
  2. User collapses the sidebar in Arabic mode and the animation slides toward the right edge (correct `translate-x` branch on `dir`); collapsing in English mode still slides left
  3. User running the dev build with a new `ml-*` / `mr-*` / `pl-*` / `pr-*` / `text-left` / `text-right` / `border-l-*` / `border-r-*` / `rounded-l-*` / `rounded-r-*` class anywhere under `frontend/src/**/*.tsx` sees an ESLint error at PR time
  4. The BlockNote spike (first plan in this phase) produces a written decision answering all 4 questions in spec §7 and the SUMM-04 editable-vs-readonly path is locked before any summary-rendering work begins
  5. The remaining 55 non-hotspot `.tsx` files are swept and contain zero physical-direction classes
**Plans**: TBD
**UI hint**: yes
**Phase note**: This phase opens with a 1-day BlockNote RTL spike (decision gate for SUMM-04), then ESLint rule, then hotspot batch (10 files / 146 hits), then sweep batch (55 files / 140 hits). Spike-as-first-plan rather than its own phase per spec §11 + standard granularity guidance.

### Phase 4: Arabic Transcription Policy
**Goal**: An Arabic-locale user gets accurate Arabic transcription via Whisper `large-v3` and never encounters Parakeet anywhere in the product.
**Depends on**: Phase 1 (Parakeet-ban invariant lives inside `set_user_preferences`), Phase 2 (onboarding fork keys on `uiLocale`)
**Requirements**: TRANS-01, TRANS-02, TRANS-03, TRANS-04
**Success Criteria** (what must be TRUE):
  1. User records a meeting with `ui_locale='ar'` and receives an MSA Arabic transcript produced by Whisper `large-v3` (not Parakeet, not a smaller Whisper model)
  2. User in Arabic mode opens `TranscriptSettings` and sees no Parakeet option in the model dropdown; an explanatory banner explains why
  3. User completes Arabic onboarding without being blocked on the ~3GB Whisper `large-v3` download — onboarding finishes immediately and a "ready to record" gate waits for the model in the background
  4. User switching `uiLocale` from `en` to `ar` from Settings has their `transcript_settings.provider` automatically rewritten to `localWhisper` + `large-v3` inside the same `set_user_preferences` transaction — no separate UI step
**Plans**: TBD

### Phase 5: Templates & Prompts (Bilingual Content)
**Goal**: All 6 meeting templates and all 5 LLM prompts exist in both English and Arabic, resolved through a single locale-aware loader, with the pre-existing `defaults.rs` 2/6 embed gap fixed.
**Depends on**: Phase 1 (`prefs.summary_language` is the resolution key)
**Requirements**: TPL-01, TPL-02, TPL-03, TPL-04, SUMM-01, SUMM-02, SUMM-03, SUMM-04
**Success Criteria** (what must be TRUE):
  1. User selects any of the 6 templates (daily_standup, standard_meeting, project_sync, psychatric_session, retrospective, sales_marketing_client_call) with `summary_language='ar'` and receives a rendered template using native MSA phrasing authored by a native speaker
  2. User running an offline build (no app-resources directory) still gets all 6 templates × 2 locales because every file is `include_str!`-embedded in `defaults.rs`
  3. User in English UI mode requests an Arabic summary (or vice versa) and gets the locale they asked for — summary language is independent from UI locale
  4. User receives Arabic summaries with Arabic punctuation (`،` `؛` `؟`) because prompts are loaded via `prompts::get_prompt(id, "ar")`, not inline strings in `processor.rs`
  5. User views an Arabic summary in BlockNote following the path locked by the Phase 3 spike — editable if green, read-only rendered markdown if red (SUMM-04 fallback)
**Plans**: TBD

### Phase 6: Rust Strings, QA & Release Hardening
**Goal**: Every Rust-owned UI string is localized, every regression-prone surface has automated coverage, and the Arabic experience is verified end-to-end against real audio and real LLM providers.
**Depends on**: Phase 1 (PREFS for QA-01), Phase 3 (UI-05 + UI-06 for QA-04), Phase 4 (TRANS-* for QA-02 / QA-05), Phase 5 (TPL/SUMM for QA-03 / QA-06)
**Requirements**: UI-07, QA-01, QA-02, QA-03, QA-04, QA-05, QA-06
**Success Criteria** (what must be TRUE):
  1. User in Arabic mode opens the system tray and sees all ~13 menu items in Arabic; user receives a system notification and sees Arabic text — both hydrated from `preferences::read().ui_locale` at startup and re-hydrated on preference change via Tauri event
  2. User performing the manual RTL regression pass walks Sidebar, Settings, Transcript, Summary, Onboarding, Tray, and Meeting Details with Arabic strings and finds no overflow, clipping, or visual asymmetry (Arabic averages ~1.2x English width)
  3. Automated regression suite covers: preference desync (startup + runtime + concurrent), Parakeet-ban enforcement (onboarding never downloads, settings never renders, API rejects), and template/prompt fallback (AR present → AR, AR missing → EN, both missing → error)
  4. User records an MSA Arabic audio sample and sees ~85-88% transcription accuracy with Whisper `large-v3`
  5. User generates an Arabic summary against both Claude and Ollama providers using an Arabic template + Arabic transcript + Arabic prompt and receives fully Arabic, RTL-formatted output from both
**Plans**: TBD
**UI hint**: yes

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Preferences Foundation | 0/1 | Planned | — |
| 2. i18n Framework & Locale Bootstrap | 0/0 | Not started | — |
| 3. RTL Layout Conversion | 0/0 | Not started | — |
| 4. Arabic Transcription Policy | 0/0 | Not started | — |
| 5. Templates & Prompts (Bilingual Content) | 0/0 | Not started | — |
| 6. Rust Strings, QA & Release Hardening | 0/0 | Not started | — |

## Coverage

- **v1 requirements:** 30
- **Mapped:** 30
- **Unmapped:** 0
- **Coverage:** 100% ✓

## Phase Structure Rationale (vs spec §11)

Spec §11 proposes 6 phases (P0–P5). This roadmap also lands at 6 phases with the same shape, with the following intentional adjustments:

1. **Spec P2 (BlockNote spike + RTL conversion)** is preserved as a single phase here (Phase 3), with the spike as its first plan acting as a decision gate for SUMM-04, rather than a separate phase. Rationale: standard granularity discourages a one-day-spike-as-its-own-phase.
2. **Spec P5 (Rust strings + testing & polish)** is preserved as a single phase here (Phase 6). All QA requirements (QA-01..06) sit downstream of Phases 1, 3, 4, and 5, so they naturally land last.
3. **QA-07 (ESLint rule)** is pulled forward into Phase 3 instead of Phase 6, because §4.6 prescribes "introduce ESLint rule **first**, then convert hotspots" — pushing it to QA at the end would let regressions accumulate.
4. **TRANS-04 (atomic provider repoint on locale switch)** is mapped to Phase 4 even though the *enforcement* lives in `set_user_preferences` (Phase 1). Phase 1 builds the invariant hook; Phase 4 wires the user-visible behavior, including the `transcript_settings::force_provider` call and the UI banner.

## Risks & Watch-outs

1. **BlockNote spike outcome gates SUMM-04 scope.** If the spike (first plan in Phase 3) finds blockers, SUMM-04 ships as read-only rendered markdown per §12.3. This is acceptable per the spec's explicit fallback decision, but Phase 5 plans must branch on the spike outcome.
2. **Phase 1 is the highest-risk single phase.** It touches 6+ recording-path call sites in the audio pipeline (`whisper_engine/commands.rs:396`, `parallel_processor.rs:344`, `audio/transcription/worker.rs:449,526`). The spec's CONCERNS doc notes the audio pipeline is a hot zone. QA-01 desync regression tests are in Phase 6, but Phase 1 should land its own targeted tests as part of execution.
3. **Phase 3 RTL conversion is the largest pure-edit phase** (286 hits / 65 files). The hotspot-first ordering (146 hits / 10 files = 51% coverage early) plus the ESLint rule landing before any conversion are the main de-risking levers.
4. **`translate-x` branching for sidebar collapse (UI-06)** is the only place where logical properties don't apply. It must be tested in both directions explicitly during the manual QA-04 pass.
5. **Arabic transcription quality (~85-88% MSA)** is a known ceiling, not a defect. QA-05 success criterion calibrates user expectation, not perfection.
6. **`audio/recording_commands.rs.backup`** is referenced in PREFS-03 — confirm during Phase 1 plan-check whether the `.backup` file is live code or vestigial; the spec mentions both potential call sites in it.

---
*Roadmap created: 2026-04-07*
