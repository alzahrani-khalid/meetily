# Meetily Arabic Bilingual Support — Design Spec v2

**Date:** 2026-04-07
**Status:** Draft (supersedes `2026-04-06-arabic-bilingual-support-design.md`)
**Scope:** Full bilingual (Arabic + English) support for Meetily

---

## 0. Why v2 exists

v1 was directionally correct but had four concrete mismatches with the codebase
(verified 2026-04-07 against `main`). v2 fixes each one with a file-grounded design
decision, not a new folder layout.

| v1 gap | v2 resolution |
|---|---|
| `next-intl` + `SettingsStore` + `localStorage` + Rust in-memory, all unreconciled | **§3** — single source of truth: `user_preferences` in SQLite, read through a new `preferences` Rust module that extends the existing `get_language_preference_internal()` pattern |
| Parakeet described as a "tooltip" problem | **§5** — Parakeet is treated as an **app-wide policy invariant** keyed on `locale`, enforced at onboarding, settings, and the transcription call sites |
| Templates described as folder duplication; stale on the existing loader | **§6** — extend `load_bundled_template` / `load_custom_template` to try `{id}.{locale}.json` first and fall back to `{id}.json`. No folder duplication. Also fixes `defaults.rs` 2/6 embed gap. |
| RTL effort described as "~110 files" | **§4** — verified: **286 directional Tailwind hits across 65 `.tsx` files**. Batched by file, hotspots listed. |
| BlockNote declared "works with RTL" | **§7** — demoted to a spike/validation item, not an assumption |

---

## 1. Overview

Add full Arabic language support: UI locale, RTL layout, Arabic speech transcription,
Arabic AI summaries. Preserve all existing English behavior.

## 2. Key Decisions (revised from v1)

| Decision | Choice | Rationale |
|---|---|---|
| Scope | Full bilingual (AR + EN) | Preserve existing English user base |
| i18n library | `next-intl` (client-only mode) | Root layout is `'use client'` (`frontend/src/app/layout.tsx:1`); use `NextIntlClientProvider`, not the server-component flavor |
| Locale storage | **SQLite `user_preferences` row**, loaded once on startup via Tauri command | Single source of truth; kills the React-state-vs-`localStorage`-vs-Rust-in-memory desync (the `useEffect` at `ConfigContext.tsx:215` already exists as a workaround for exactly this class of bug) |
| RTL strategy | Tailwind logical properties + `dir={locale === 'ar' ? 'rtl' : 'ltr'}` on `<html>` | Matches the project-wide rule in `~/.claude/CLAUDE.md` (Web/Tailwind section) |
| Arabic font | Tajawal via `next/font/google` | Same loading mechanism used for `Source_Sans_3` (`layout.tsx:30`) |
| Arabic transcription | **localWhisper large-v3 only** — Parakeet forbidden for `ar` | Parakeet TDT variants don't support Arabic; enforced as a policy, not a tooltip |
| Summary language | Independent from UI locale | Unchanged from v1 — correct design |
| LLM prompts | Externalize the 5 hardcoded prompts in `processor.rs:215,216,281,282,316` to locale-aware resource files | Currently inline; touched every time prompts need tuning |
| Meeting templates | **Locale-suffix resolution inside existing loader** | No folder duplication; reuses `custom → bundled → builtin` fallback chain in `frontend/src-tauri/src/summary/templates/loader.rs:95` |

---

## 3. The Preferences Model (the one Codex said was missing)

### 3.1 Current state (verified 2026-04-07)

User preferences currently live in **four** places:

1. **React state + `localStorage`** — `ConfigContext.tsx:140` reads `primaryLanguage`
   from `localStorage`, mirrors it in component state.
2. **Rust process-global** — `lib.rs:376` (`set_language_preference`) +
   `lib.rs:386` (`get_language_preference_internal`). Read from 6+ call sites in
   the transcription path: `whisper_engine/commands.rs:396`,
   `whisper_engine/parallel_processor.rs:344`,
   `audio/transcription/worker.rs:449`, `audio/transcription/worker.rs:526`,
   plus two in `audio/recording_commands.rs.backup`.
3. **SQLite `settings` table** — `database/repositories/setting.rs`, but only for
   provider/model/API keys. No language column.
4. **Onboarding status JSON** — `OnboardingContext.tsx` checks model download state.

This is why `ConfigContext.tsx:215` exists as a "fixes startup desync bug"
workaround: the React state and Rust state drift at startup.

### 3.2 v2 design: one Rust module, one SQLite row, two Tauri commands

**New SQLite table** (migration, applied at startup):

```sql
CREATE TABLE IF NOT EXISTS user_preferences (
  id TEXT PRIMARY KEY DEFAULT '1',
  ui_locale TEXT NOT NULL DEFAULT 'en',         -- 'en' | 'ar'
  summary_language TEXT NOT NULL DEFAULT 'en',  -- 'en' | 'ar' (can diverge from ui_locale)
  transcription_language TEXT NOT NULL DEFAULT 'auto', -- existing primaryLanguage
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);
INSERT OR IGNORE INTO user_preferences (id) VALUES ('1');
```

**New Rust module** `frontend/src-tauri/src/preferences/`:

- `mod.rs` — `UserPreferences` struct, process-global `Lazy<RwLock<UserPreferences>>`
  hydrated from SQLite on app start (via `lib.rs::run` before command registration).
- `commands.rs` — two Tauri commands:
  - `#[tauri::command] async fn get_user_preferences() -> UserPreferences`
  - `#[tauri::command] async fn set_user_preferences(prefs: UserPreferences) -> Result<UserPreferences, String>`
    — updates SQLite + the RwLock in a single transaction so there is no
    window where in-memory and disk disagree.
- `repository.rs` — SQL adapters (mirrors `settings/setting.rs` style).

**Migrate existing callers**:

- Replace `get_language_preference_internal()` with
  `preferences::read().transcription_language`. All 6 call sites updated in one
  commit; the old function is deleted, not deprecated (no external consumers).
- Delete `primaryLanguage` from `localStorage`. Replace
  `ConfigContext.tsx:140` with a single Tauri call at mount, hydrated into
  context. The startup-desync `useEffect` at `ConfigContext.tsx:215` can then
  be deleted — there is nothing left to desync.

**Frontend contract**:

```ts
// frontend/src/services/preferencesService.ts  (new)
export type UiLocale = 'en' | 'ar';
export type SummaryLanguage = 'en' | 'ar';

export interface UserPreferences {
  uiLocale: UiLocale;
  summaryLanguage: SummaryLanguage;
  transcriptionLanguage: string; // 'auto' | 'en' | 'ar' | ...
}

export async function getUserPreferences(): Promise<UserPreferences>;
export async function setUserPreferences(p: Partial<UserPreferences>): Promise<UserPreferences>;
```

Context wiring: `ConfigContext` adds `uiLocale`, `summaryLanguage`, and a single
setter that calls `setUserPreferences` and updates React state from the
returned `UserPreferences` payload. No direct `localStorage` access for these
fields.

### 3.3 Root layout integration

`layout.tsx:234` currently hardcodes `<html lang="en">`. v2:

```tsx
// layout.tsx (unchanged 'use client' at top)
const [initialPreferences, setInitialPreferences] = useState<UserPreferences | null>(null);

useEffect(() => {
  invoke<UserPreferences>('get_user_preferences').then(setInitialPreferences);
}, []);

const uiLocale = initialPreferences?.uiLocale ?? 'en';

return (
  <html lang={uiLocale} dir={uiLocale === 'ar' ? 'rtl' : 'ltr'} suppressHydrationWarning>
    <body className={bodyFontClasses(uiLocale)}>
      {initialPreferences ? (
        <ConfigProvider initialPreferences={initialPreferences}>
          <NextIntlClientProvider locale={uiLocale} messages={messages[uiLocale]}>
            {/* existing provider tree unchanged */}
          </NextIntlClientProvider>
        </ConfigProvider>
      ) : (
        <BootSplash />
      )}
    </body>
  </html>
);
```

Because the layout is already a client component, bootstrap happens in
`RootLayout` itself. Until `get_user_preferences` resolves, the app renders only
the blocking splash shell and does **not** mount `Sidebar`, `MainContent`, or
other RTL-sensitive UI. This avoids a visible flash of LTR app chrome without
inventing a separate pre-React bootstrap path.

---

## 4. RTL Layout Conversion — with real numbers

### 4.1 Verified scope

As of 2026-04-07, a `.tsx` sweep for
`ml-|mr-|pl-|pr-|left-|right-|text-left|text-right|border-l|border-r|rounded-l|rounded-r|space-x-`
across `frontend/src/**/*.tsx` returns **286 occurrences across 65 files**.

### 4.2 Hotspots (top 10 by occurrence count)

| File | Occurrences |
|---|---|
| `components/Sidebar/index.tsx` | 27 |
| `components/ModelSettingsModal.tsx` | 21 |
| `components/AnalyticsDataModal.tsx` | 16 |
| `components/AISummary/index.tsx` | 16 |
| `components/WhisperModelManager.tsx` | 12 |
| `components/ChunkProgressDisplay.tsx` | 12 |
| `app/_components/SettingsModal.tsx` | 11 |
| `components/ui/dropdown-menu.tsx` | 11 |
| `components/MeetingDetails/SummaryPanel.tsx` | 11 |
| `components/ImportAudio/ImportAudioDialog.tsx` | 9 |

Remaining 55 files have ≤8 occurrences each.

### 4.3 Conversion rules (unchanged from v1, correct)

| Before | After |
|---|---|
| `ml-*` | `ms-*` |
| `mr-*` | `me-*` |
| `pl-*` | `ps-*` |
| `pr-*` | `pe-*` |
| `left-0` | `start-0` |
| `right-0` | `end-0` |
| `text-left` | `text-start` |
| `text-right` | `text-end` |
| `border-l-*` | `border-s-*` |
| `border-r-*` | `border-e-*` |
| `rounded-l-*` | `rounded-s-*` |
| `rounded-r-*` | `rounded-e-*` |
| `space-x-*` | `space-x-*` + `rtl:space-x-reverse` |

### 4.4 Sidebar (the hardest one)

`components/Sidebar/index.tsx:664` uses `translate-x` for collapse animation.
Logical properties do not exist for `translate`. v2 approach: branch the
animation class on `dir`:

```tsx
const collapseClass = uiLocale === 'ar'
  ? (collapsed ? 'translate-x-full' : 'translate-x-0')
  : (collapsed ? '-translate-x-full' : 'translate-x-0');
```

`fixed top-0 left-0` → `fixed top-0 start-0` (works fine).

### 4.5 Meeting details panel

`app/meeting-details/page-content.tsx:173` has a `flex-row` container. Setting
`dir="rtl"` on the document root automatically flips visual order — no JSX
reordering, no `flex-row-reverse`.

### 4.6 Conversion methodology

1. Introduce ESLint rule `no-restricted-syntax` forbidding `ml-*`/`mr-*`/etc.
   in new code (fast fail for regressions).
2. Convert hotspot files first (10 files above = 146 of 286 hits = 51%).
3. Sweep remaining 55 files.
4. Manual RTL regression pass on: Sidebar, Settings, Transcript, Summary,
   Onboarding, Meeting Details, all dialogs.

---

## 5. Transcription Policy — Parakeet ban for Arabic

### 5.1 Current Parakeet entanglement (verified)

| Location | What it does | v2 change |
|---|---|---|
| `ConfigContext.tsx:110` | Defaults `transcriptModelConfig.provider` to `'parakeet'` | Default stays `'parakeet'` for `uiLocale === 'en'`, switches to `'localWhisper'` when hydrated prefs have `uiLocale === 'ar'` |
| `OnboardingContext.tsx:8` | `PARAKEET_MODEL` constant hardcoded | Add `DEFAULT_TRANSCRIPTION_MODEL` that branches on `uiLocale` |
| `OnboardingContext.tsx:424` | `startBackgroundDownloads` always downloads Parakeet first | Fork: if onboarding locale is `ar`, download Whisper large-v3 instead; skip Parakeet entirely |
| `TranscriptSettings.tsx:124` | Renders Parakeet as "Recommended" | Hide the Parakeet `<SelectItem>` when `uiLocale === 'ar'`; show an info banner instead |
| `SettingsModal.tsx:229` | Passes `provider` into `LanguageSelection` | Unchanged (already correct); `LanguageSelection` already has `'ar'` at `LanguageSelection.tsx:29` |
| `whisper_engine/commands.rs:396`, `parallel_processor.rs:344`, `audio/transcription/worker.rs:449,526` | Read `get_language_preference_internal()` at recording time | After §3.2 migration, read `preferences::read().transcription_language`; no behavior change for English; Arabic already routes through Whisper because Parakeet was never wired into these call sites |

### 5.2 The invariant

**When `uiLocale === 'ar'`, `transcriptModelConfig.provider === 'parakeet'` must
be impossible to express.** The enforcement layer is the preferences setter:

```rust
// preferences/commands.rs
async fn set_user_preferences(...) -> Result<UserPreferences, String> {
    if prefs.ui_locale == "ar" {
        // Force transcript provider to localWhisper on Arabic switch
        transcript_settings::force_provider(pool, "localWhisper", "large-v3").await?;
    }
    // ... write prefs
}
```

This way, no UI code needs to "remember" the Parakeet ban — the state can't
enter an invalid configuration.

### 5.3 Arabic Whisper model requirement

`large-v3` only (smaller models hallucinate in Arabic). Enforced in
`TranscriptSettings` by filtering the model dropdown when `uiLocale === 'ar'`.
Whisper model download in onboarding triggered if not present.

### 5.4 Quality expectations (unchanged from v1)

- MSA: ~85-88% accuracy with large-v3
- Dialectal: lower, out of scope
- Code-switching: out of scope

---

## 6. Prompts & Templates — Locale-aware resolution in the existing loader

### 6.1 Current state (verified)

**Templates** — already externalized with a 3-tier loader:
- `frontend/src-tauri/src/summary/templates/loader.rs:95` implements
  `custom (appdata) → bundled (app resources) → builtin (embedded)`.
- 6 JSON files exist at `frontend/src-tauri/templates/`:
  `daily_standup.json`, `standard_meeting.json`, `project_sync.json`,
  `psychatric_session.json` (sic), `retrospective.json`,
  `sales_marketing_client_call.json`.
- **Only 2 are embedded** at `defaults.rs:7` (`DAILY_STANDUP`, `STANDARD_MEETING`).
  This is a pre-existing bug: if bundled resources are missing, 4 templates
  vanish. v2 fixes this by including all 6.

**Prompts** — still hardcoded, confirmed at `processor.rs:215, 216, 281, 282, 316`.
5 prompt strings total: chunk system, chunk user template, combine system,
combine user template, final report system.

### 6.2 Template resolution: filename suffix, not folder duplication

Extend the loader to try locale-suffixed filenames first:

```rust
// loader.rs
pub fn get_template(template_id: &str, locale: &str) -> Result<Template, String> {
    // Try: {id}.{locale}.json → {id}.json, each at custom → bundled → builtin
    let suffixed = format!("{}.{}", template_id, locale);
    let json = load_custom_template(&suffixed)
        .or_else(|| load_custom_template(template_id))
        .or_else(|| load_bundled_template(&suffixed))
        .or_else(|| load_bundled_template(template_id))
        .or_else(|| defaults::get_builtin_template(&suffixed).map(str::to_string))
        .or_else(|| defaults::get_builtin_template(template_id).map(str::to_string))
        .ok_or_else(|| format!("template '{}' not found", template_id))?;
    validate_and_parse_template(&json)
}
```

Arabic templates live alongside English in the same directory:

```
frontend/src-tauri/templates/
├── daily_standup.json          # English (default)
├── daily_standup.ar.json       # Arabic
├── standard_meeting.json
├── standard_meeting.ar.json
├── project_sync.json
├── project_sync.ar.json
├── psychatric_session.json
├── psychatric_session.ar.json
├── retrospective.json
├── retrospective.ar.json
├── sales_marketing_client_call.json
└── sales_marketing_client_call.ar.json
```

The caller in `processor.rs:309` updates to pass the current `summaryLanguage`:

```rust
let template = templates::get_template(template_id, &prefs.summary_language)?;
```

### 6.3 Fix the `defaults.rs` 2/6 embed gap

`defaults.rs` currently embeds only 2 of the 6 templates. v2 embeds all 12
(6 English + 6 Arabic) so the builtin fallback is complete:

```rust
pub const DAILY_STANDUP:       &str = include_str!("../../../templates/daily_standup.json");
pub const DAILY_STANDUP_AR:    &str = include_str!("../../../templates/daily_standup.ar.json");
pub const STANDARD_MEETING:    &str = include_str!("../../../templates/standard_meeting.json");
pub const STANDARD_MEETING_AR: &str = include_str!("../../../templates/standard_meeting.ar.json");
// ... 4 more pairs
```

### 6.4 Prompts: externalize to resource files with same locale resolution

New directory `frontend/src-tauri/prompts/`:

```
prompts/
├── chunk_system.txt
├── chunk_system.ar.txt
├── chunk_user.txt
├── chunk_user.ar.txt
├── combine_system.txt
├── combine_system.ar.txt
├── combine_user.txt
├── combine_user.ar.txt
├── report_system.txt
└── report_system.ar.txt
```

New module `frontend/src-tauri/src/summary/prompts/`:
- `loader.rs` — mirrors `templates/loader.rs` signature:
  `pub fn get_prompt(id: &str, locale: &str) -> Result<String, String>`
- `defaults.rs` — embeds all 10 files via `include_str!` so offline builds work.

`processor.rs:215, 281, 316` replaced with `prompts::get_prompt(id, locale)?`.

### 6.5 Arabic prompt design

- Instructs LLM to generate MSA Arabic output
- Requests proper RTL formatting (Arabic punctuation: `،` `؛` `؟`)
- Maintains same structure as English (chunk → combine → report)
- Written by a native speaker, not machine-translated from English

---

## 7. BlockNote Editor — demoted to a spike

v1 claimed BlockNote "works with RTL — Radix primitives are RTL-aware." This is
**not verified**. Known facts:

- `frontend/src/components/BlockNoteEditor/Editor.tsx:6` imports
  `@blocknote/core/fonts/inter.css` — Inter has no Arabic glyphs.
- `Editor.tsx:25` calls `useCreateBlockNote({ initialContent })` — no `dictionary`,
  no direction config, no custom CSS classes.
- BlockNote 0.36.0 is the installed version (`package.json:31`); its RTL
  support story must be checked against its release notes.

**v2 treatment: Phase P2 opens with a 1-day spike** to answer:

1. Does BlockNote 0.36.0 support RTL out of the box via `dir="rtl"` on the parent?
2. If not, can it be forced via CSS (`[dir="rtl"] .bn-editor { direction: rtl; }`)?
3. Does the Inter font import break Arabic rendering, and is there a drop-in
   replacement that covers both scripts?
4. Does `@blocknote/shadcn` surface any `dictionary` prop for UI strings?

Only after the spike answers these do we commit to the editor implementation.
The spec does **not** promise RTL BlockNote in v1 of Arabic support — if the
spike finds blockers, we ship Arabic with a read-only summary fallback and
close the editor gap in a follow-up.

---

## 8. Rust Backend Strings

- Tray menu items (~13 strings): small locale map in Rust, hydrated at startup
  from `preferences::read().ui_locale`. Re-hydrated on preference change via a
  Tauri event.
- Notification messages (~10 strings): same approach.
- Log/error messages: remain in English (developer-facing).

---

## 9. Out of Scope (YAGNI)

- Dialect-specific Arabic (MSA only for v1)
- Mixed-language / code-switching transcription
- Arabic-specific Whisper fine-tuning
- Custom Arabic keyboard input handling (OS handles this)
- Per-component language switching (whole app switches at once)
- Dynamic locale switching without a full re-render (a reload is fine)

---

## 10. Testing Strategy

### 10.1 Preference desync regression tests

The §3 migration is the highest-risk change because it touches recording-path
code. Tests must cover:

- Starting the app with `ui_locale = 'ar'` in SQLite → layout renders RTL on first
  paint, no flash of LTR
- Switching locale at runtime → new preference is visible to
  `whisper_engine::commands` on the **next** recording (no stale cache)
- `set_user_preferences({ ui_locale: 'ar' })` → `transcript_settings.provider`
  is rewritten to `localWhisper` atomically (§5.2 invariant)
- Concurrent `set_user_preferences` calls don't leave partial state

### 10.2 Parakeet ban enforcement tests

- Onboarding flow with `ui_locale = 'ar'` never calls `parakeet_download_model`
- Settings modal with `ui_locale = 'ar'` does not render Parakeet option
- Attempting to invoke `api_save_transcript_config` with `provider: 'parakeet'`
  while `ui_locale = 'ar'` → rejected with a clear error

### 10.3 Template/prompt resolution tests

- `get_template("daily_standup", "ar")` with both files present → returns Arabic
- `get_template("daily_standup", "ar")` with only English present → returns English (fallback)
- `get_template("nonexistent", "ar")` → error

### 10.4 Visual RTL regression

- Manual pass through all screens in Arabic mode (automated visual diff nice-to-have)
- Key surfaces: Sidebar (collapse/expand animation §4.4), Settings modal,
  Transcript panel, Summary panel, Onboarding flow, Tray menu, Meeting details
- Verify no text overflow or clipping with Arabic strings (Arabic averages ~1.2x
  English width)

### 10.5 Transcription & summary quality

- Whisper Arabic with MSA audio samples → accuracy spot-check
- LLM summaries in Arabic with 2+ providers (Claude + Ollama)
- Template rendering: Arabic template with Arabic transcript with Arabic prompt
  → output is fully Arabic, RTL-formatted

---

## 11. Implementation Phases

| Phase | Scope |
|---|---|
| P0 | **Preferences module (§3)** — new SQLite table, Rust module, Tauri commands, migration of 6+ recording-path call sites, deletion of desync workaround. Ship behind a feature flag; no user-visible UI yet. |
| P1 | i18n framework setup — `next-intl` client provider, English message extraction, Arabic translation stubs, root layout `dir` switch |
| P2 | **BlockNote spike (§7)**, then RTL CSS conversion — hotspot files first (§4.2), then sweep. ESLint rule to prevent regressions. |
| P3 | Transcription policy (§5) — Parakeet ban invariant, onboarding fork, settings UI filtering, Arabic-only Whisper model enforcement |
| P4 | Prompts/templates (§6) — externalize 5 hardcoded prompts, locale-suffix loader extension, fix `defaults.rs` 2/6 embed gap, Arabic prompt authoring |
| P5 | Rust backend strings (§8) + testing & polish |

Effort estimates intentionally omitted until P0 is scoped against current
maintainer availability.

---

## 12. Open Questions

1. **Where does initial locale default come from on first run?** Options:
   (a) always `en`, user picks Arabic in onboarding;
   (b) detect from `navigator.language` in the renderer and seed the initial
   SQLite write.
   Recommendation: (b), falling back to (a) if detection is ambiguous.

2. **Does the Arabic onboarding flow block on Whisper large-v3 download?**
   large-v3 is ~3GB vs Parakeet's ~600MB. The default onboarding UX should
   probably allow finishing onboarding before download completes, with a
   "ready to record" gate that waits for the model.

3. **BlockNote fallback strategy** if the §7 spike finds blockers: read-only
   rendered Arabic summary (no editing) vs. ship Arabic behind a "beta editor"
   flag vs. delay Arabic launch. To be decided after spike.

4. **Locale switching mid-session** — do we reload the app or hot-swap?
   Recommendation: reload, because the `NextIntlClientProvider` + `dir`
   attribute + recording-path invariants are all easier to reason about on a
   fresh mount. Worth confirming against user expectation.

---

## 13. Non-goals of this spec document

This spec does **not**:

- Prescribe the Arabic translation strings themselves
- Design the Arabic meeting templates
- Estimate engineering days (deferred to P0 scoping)
- Mandate specific commit granularity
- Cover backend Python FastAPI i18n (backend is developer-facing only; not user-visible)
