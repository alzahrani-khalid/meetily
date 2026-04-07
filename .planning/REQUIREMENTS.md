# Requirements: Meetily v1.0 — Arabic Bilingual Support

**Defined:** 2026-04-07
**Core Value:** Record a meeting, get an accurate transcript and a useful summary — in your own language, without any audio or content leaving the machine.
**Source spec:** `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md`

## v1 Requirements

Requirements for the Arabic bilingual milestone. Each maps to exactly one roadmap phase.

### UI (Bilingual UI Surface)

- [ ] **UI-01**: User can launch Meetily on a system with `navigator.language` starting with `ar` and see the interface in Arabic on first run (detection with `en` fallback, per §12.1)
- [ ] **UI-02**: User can switch UI language between Arabic and English from Settings
- [ ] **UI-03**: User switches language and the app triggers a full reload; the selection persists across restarts (§12.4 — reload, not hot-swap)
- [ ] **UI-04**: User sees Arabic text rendered right-to-left in Tajawal (loaded via `next/font/google` alongside existing `Source_Sans_3`)
- [ ] **UI-05**: User sees all layout elements mirror correctly in RTL across the 65 `.tsx` files / 286 directional hit surfaces: sidebar, top bar, dialogs, buttons, dropdowns, menus, icons
- [ ] **UI-06**: User sees the sidebar collapse animation work in both directions (`translate-x-full` vs `-translate-x-full` branched on `dir`, since logical properties don't exist for `translate`)
- [ ] **UI-07**: User sees Rust-owned UI elements (tray menu ~13 strings, system notifications ~10 strings) in the selected language, hydrated from `preferences::read().ui_locale` at startup and re-hydrated on preference change via a Tauri event

### TRANS (Arabic Transcription)

- [ ] **TRANS-01**: User can record a meeting in Arabic and receive an accurate MSA transcription via Whisper `large-v3`
- [ ] **TRANS-02**: User in Arabic UI mode cannot express `provider: 'parakeet'` anywhere in the app — Parakeet option hidden in Settings, ban enforced inside `set_user_preferences`, invalid API calls rejected (§5.2 invariant)
- [ ] **TRANS-03**: User in Arabic onboarding is not blocked on the Whisper `large-v3` (~3GB) download; onboarding completes and a "ready to record" gate waits for the model (§12.2 — non-blocking with gate)
- [ ] **TRANS-04**: User switching UI locale from `en` → `ar` has their transcript provider automatically repointed to `localWhisper` + `large-v3` atomically within the same preferences write

### SUMM (Arabic Summaries)

- [ ] **SUMM-01**: User can request a meeting summary in Arabic regardless of UI locale (summary language independent from UI language, per §2)
- [ ] **SUMM-02**: User can select from all 6 meeting templates (daily_standup, standard_meeting, project_sync, psychatric_session, retrospective, sales_marketing_client_call) available in both English and Arabic
- [ ] **SUMM-03**: User receives Arabic summaries with native MSA phrasing and proper Arabic punctuation (`،` `؛` `؟`) — prompts authored by a native speaker, not machine-translated
- [ ] **SUMM-04**: User can view an Arabic summary in the BlockNote editor — editable if the §7 spike confirms RTL support, read-only rendered markdown fallback if the spike finds blockers (§12.3 decision)

### PREFS (Preferences Infrastructure)

- [ ] **PREFS-01**: User preferences (`ui_locale`, `summary_language`, `transcription_language`) live in a single SQLite `user_preferences` row, readable through one Rust `preferences` module with process-global `Lazy<RwLock<UserPreferences>>` hydrated at startup
- [ ] **PREFS-02**: Setting a preference updates SQLite + in-memory `RwLock` atomically in a single transaction; no window where callers can observe partial state, and the Parakeet-ban invariant (§5.2) is enforced as part of the same write
- [ ] **PREFS-03**: All 6+ recording-path call sites (`whisper_engine/commands.rs:396`, `whisper_engine/parallel_processor.rs:344`, `audio/transcription/worker.rs:449`, `audio/transcription/worker.rs:526`, plus any in `audio/recording_commands.rs.backup`) read preferences from the new module; `get_language_preference_internal()` is deleted, not deprecated
- [ ] **PREFS-04**: The `ConfigContext.tsx:215` startup-desync `useEffect` workaround is removed in the same commit that eliminates its cause; `primaryLanguage` no longer touches `localStorage`

### TPL (Templates & Prompts Infrastructure)

- [ ] **TPL-01**: Meeting template loader resolves `{id}.{locale}.json` first, falls back to `{id}.json`, at every tier of the existing custom → bundled → builtin chain in `frontend/src-tauri/src/summary/templates/loader.rs`
- [ ] **TPL-02**: All 6 meeting templates are embedded in `defaults.rs`, fixing the pre-existing 2/6 embed gap (currently only `DAILY_STANDUP` and `STANDARD_MEETING` are embedded)
- [ ] **TPL-03**: 5 LLM prompts externalized from `frontend/src-tauri/src/summary/processor.rs:215, 216, 281, 282, 316` to `frontend/src-tauri/prompts/*.txt` with a new `summary/prompts/` loader module mirroring the template resolution signature
- [ ] **TPL-04**: `processor.rs` call sites read prompts via `prompts::get_prompt(id, locale)` instead of inline strings; all 10 prompt files (5 EN + 5 AR) embedded via `include_str!` in a `prompts/defaults.rs` so offline builds work

### QA (Quality Assurance)

- [ ] **QA-01**: Automated regression tests cover preference desync scenarios: startup with `ui_locale='ar'` renders RTL on first paint (no LTR flash), runtime switch is visible to `whisper_engine::commands` on the next recording, concurrent `set_user_preferences` calls don't leave partial state (§10.1)
- [ ] **QA-02**: Automated tests enforce the Parakeet-ban invariant: Arabic onboarding never calls `parakeet_download_model`, settings modal doesn't render the Parakeet option, `api_save_transcript_config({provider:'parakeet'})` rejected with a clear error when `ui_locale === 'ar'` (§10.2)
- [ ] **QA-03**: Automated tests cover template/prompt locale fallback: `get_template("daily_standup", "ar")` with both files present → AR, with only EN present → EN fallback, with neither present → error (§10.3)
- [ ] **QA-04**: Manual RTL regression pass completed on Sidebar (including collapse animation §4.4), Settings modal, Transcript panel, Summary panel, Onboarding flow, Tray menu, Meeting Details — no text overflow or clipping with Arabic strings (§10.4, Arabic averages ~1.2x English width)
- [ ] **QA-05**: Arabic transcription quality spot-check with MSA audio samples against Whisper `large-v3` — expected ~85-88% accuracy for MSA (§5.4, §10.5)
- [ ] **QA-06**: Arabic summary quality check with Claude + Ollama providers: Arabic template + Arabic transcript + Arabic prompt → fully Arabic RTL-formatted output (§10.5)
- [ ] **QA-07**: ESLint `no-restricted-syntax` rule prevents new physical-direction Tailwind classes (`ml-*`, `mr-*`, `pl-*`, `pr-*`, `text-left`, `text-right`, `border-l-*`, `border-r-*`, `rounded-l-*`, `rounded-r-*`) from entering the codebase (§4.6)

## v2 Requirements

Acknowledged but deferred to a future release.

### Editor (BlockNote Arabic Editing)

- **EDIT-01**: User can edit an Arabic summary in the BlockNote editor with full rich-text controls (deferred if §7 spike finds blockers; fallback for v1.0 is read-only rendered markdown per SUMM-04)
- **EDIT-02**: User sees the BlockNote slash menu and block labels in Arabic (requires `dictionary` prop or equivalent in `@blocknote/shadcn`; to be confirmed in spike)

### Additional Languages

- **LANG-01**: User can select a third UI language beyond English and Arabic (infrastructure supports it via `next-intl`, but content authoring deferred)

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Dialectal Arabic (Egyptian, Levantine, Gulf, Maghrebi) | MSA only; dialects vary too much for Whisper and we ship one quality bar (§9) |
| Mixed-language / code-switching transcription | Whisper handles poorly, scope creep risk, deferred indefinitely (§9) |
| Arabic-specific Whisper fine-tuning | Custom ML work out of scope; use `large-v3` as-is (§9) |
| Custom Arabic keyboard input handling | The OS already handles this correctly (§9) |
| Per-component language switching | Whole app switches at once; avoids a combinatorial testing matrix (§9) |
| Hot-swap mid-session locale switching | Triggers full reload instead (§12.4); simpler invariants for `NextIntlClientProvider` + `dir` + recording-path state |
| Backend Python FastAPI i18n | Backend is developer-facing (Swagger, logs); not user-visible (§13) |
| Log and error message translation | Developer-facing; remain in English (§8) |
| Dedicated "pick your language" onboarding step | First-run uses `navigator.language` detection; user switches later from Settings if needed (§12.1) |
| BlockNote editor upgrade beyond 0.36.0 | Version locked for this milestone; upgrade is a separate concern |

## Traceability

Which phases cover which requirements. Populated by the roadmapper during Step 8.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PREFS-01 | — | Pending |
| PREFS-02 | — | Pending |
| PREFS-03 | — | Pending |
| PREFS-04 | — | Pending |
| UI-01 | — | Pending |
| UI-02 | — | Pending |
| UI-03 | — | Pending |
| UI-04 | — | Pending |
| UI-05 | — | Pending |
| UI-06 | — | Pending |
| UI-07 | — | Pending |
| TRANS-01 | — | Pending |
| TRANS-02 | — | Pending |
| TRANS-03 | — | Pending |
| TRANS-04 | — | Pending |
| TPL-01 | — | Pending |
| TPL-02 | — | Pending |
| TPL-03 | — | Pending |
| TPL-04 | — | Pending |
| SUMM-01 | — | Pending |
| SUMM-02 | — | Pending |
| SUMM-03 | — | Pending |
| SUMM-04 | — | Pending |
| QA-01 | — | Pending |
| QA-02 | — | Pending |
| QA-03 | — | Pending |
| QA-04 | — | Pending |
| QA-05 | — | Pending |
| QA-06 | — | Pending |
| QA-07 | — | Pending |

**Coverage:**
- v1 requirements: 30 total
- Mapped to phases: 0 (populated by roadmapper)
- Unmapped: 30 ⚠️ (will resolve after Step 8)

---
*Requirements defined: 2026-04-07*
*Last updated: 2026-04-07 after initial definition*
