---
phase: 2
slug: i18n-framework-locale-bootstrap
status: ready-for-research
gathered: 2026-04-09
milestone: v1.0 — Arabic Bilingual Support
requirements: [UI-01, UI-02, UI-03, UI-04]
upstream_artifacts:
  - .planning/phases/02-i18n-framework-locale-bootstrap/02-UI-SPEC.md (approved 2026-04-08)
  - .planning/phases/01-preferences-foundation/01-01-SUMMARY.md
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md §3, §12.1, §12.4
---

# Phase 2: i18n Framework & Locale Bootstrap — Context

**Gathered:** 2026-04-09
**Status:** Ready for research

<domain>
## Phase Boundary

Bootstrap Meetily's bilingual UI surface so that every later phase inherits a working `<html lang dir>` switch, a loaded Arabic font, and a `next-intl` provider tree. Concretely, Phase 2 delivers exactly four user-visible outcomes (UI-01..UI-04):

1. **First-run detection (UI-01)** — A user whose system reports `navigator.language` starting with `ar` launches Meetily for the very first time and sees Arabic strings, with `en` as the safe fallback.
2. **Settings switcher (UI-02)** — A user can switch the UI language between English and Arabic from `SettingsModal`.
3. **Reload + persistence (UI-03)** — Switching the language triggers a full `window.location.reload()` (no hot-swap) and the choice persists across restarts.
4. **Tajawal font + RTL render (UI-04)** — Arabic text renders right-to-left in Tajawal (loaded via `next/font/google`); English continues to render in Source Sans 3. Neither font ever leaks into the wrong locale.

**Strict scope boundaries inherited from `02-UI-SPEC.md` Out-of-Scope table — Phase 2 plans MUST NOT touch these:**

- Migrating any **pre-existing** component strings into the message catalogue (Phase 3+)
- Any RTL conversion of the 286 directional Tailwind hits across the 65 `.tsx` files (Phase 3)
- ESLint `no-restricted-syntax` rule banning physical-direction classes (Phase 3)
- BlockNote RTL spike, Inter font fix, or any change to `BlockNoteEditor/Editor.tsx` (Phase 3)
- Sidebar collapse `translate-x` direction branch (Phase 3)
- Any Parakeet UI filter, onboarding fork, or "ready to record" gate (Phase 4)
- Tray menu / system notification Arabic strings (Phase 6)
- Editing `tailwind.config.ts` typography or color tokens (Phase 2 narrows *which* tokens are consumed; it does not edit the file)
- Seeding `summary_language` or `transcription_language` from `navigator.language` — only `ui_locale` is seeded; the other two stay at the Phase 1 defaults

**Phase 1 surface this phase consumes** (already shipped, do NOT redesign):

- `preferences::read()` — sync hot-path reader returning an owned `UserPreferences` clone
- `preferences::commands::{get_user_preferences, set_user_preferences}` Tauri commands
- `frontend/src/services/preferencesService.ts` — typed client wrapper
- `ConfigProvider` already accepts an `initialPreferences` prop (shipped in Phase 1 commit `adb4dc0`)
- The atomic Parakeet-ban invariant inside `set_user_preferences` (any `setUserPreferences({ uiLocale: 'ar' })` Phase 2 issues will auto-repoint `transcript_settings.provider` if it currently equals `parakeet` — Phase 2 does not need to coordinate this; it's free)

</domain>

<decisions>
## Implementation Decisions

### First-run detection persistence (the one decision UI-SPEC explicitly deferred)

- **D-01 — Add a `bootstrapped` column.** Phase 2 ships a lightweight migration `frontend/src-tauri/migrations/20260409000000_add_preferences_bootstrapped_flag.sql` that adds `bootstrapped INTEGER NOT NULL DEFAULT 0` to the `user_preferences` table. The bootstrap detector flips it to `1` the first time it runs successfully (whether it kept `en` or switched to `ar`).
- **D-02 — Why a column, not "always re-detect on `en`".** The "always re-detect" fallback in UI-SPEC is unsafe: a user who deliberately picks English on a US-locale machine and later moves to a Saudi-locale machine would silently flip to Arabic on the next launch. That's a correctness bug, not benign behavior. The `bootstrapped` column makes the detection event a single, persistent fact.
- **D-03 — Migration lives in the Phase 1 `preferences::repository` module.** The new column extends the row Phase 1 created. The migration file lives next to `20260407000000_add_user_preferences.sql`, the field is added to the `UserPreferences` struct in `preferences/repository.rs`, and `hydrate_from_db` reads it. **No new Rust module is created.** Tauri command shape (`get_user_preferences` / `set_user_preferences`) does NOT change at the FFI boundary — `bootstrapped` is exposed in the response payload (TypeScript `UserPreferences` type gains `bootstrapped: boolean`).
- **D-04 — `setUserPreferences` is the writer.** When the bootstrap detector decides to persist (either flipping locale to `'ar'` or just marking the row as bootstrapped), it issues a single `setUserPreferences({ uiLocale, bootstrapped: true })` call. The Phase 1 atomic transaction handles it. The detector NEVER writes directly to SQLite or to a separate `markBootstrapped` command.

### `next-intl` integration

- **D-05 — Pin to `next-intl@^3.26.x` (latest 3.x at install time).** Phase 2 uses only `NextIntlClientProvider` and `useTranslations`, both of which are API-stable across 3.x and 4.x. v3 is the lower-risk choice for our Tauri client-only SPA: zero migration concerns, widely documented, and we don't use any v4-only feature (no server-side helpers, no middleware, no `getTranslations`). Pin via `pnpm add next-intl@^3.26.0` and let pnpm resolve the latest 3.x patch at install time.
- **D-06 — Client-provider mode ONLY.** The root layout is already `'use client'` and stays that way. No `i18n.ts` server-side config file. No `app/[locale]/` route segment. No middleware. The provider tree wraps `messages` directly via `<NextIntlClientProvider locale={uiLocale} messages={messages[uiLocale]}>`. (This is locked by PROJECT.md Constraints — re-stated here as a load-bearing reminder for the planner because next-intl docs default to the server-component flavor.)
- **D-07 — `I18nProvider` is a thin wrapper.** A new file `frontend/src/providers/I18nProvider.tsx` wraps `NextIntlClientProvider` and is the ONLY component in the app that imports from `next-intl`. The wrapper takes `locale: 'en' | 'ar'` and `messages: Messages` props. Rationale: future swap of i18n library (or version bump) is contained to one file; this also gives us a single place to add a `timeZone` prop and other defaults.

### Bootstrap helper structure & test strategy

- **D-08 — Pure-function extraction.** A new file `frontend/src/lib/bootstrapLocale.ts` exports a single pure function:
  ```ts
  export function bootstrapLocale(
    prefs: UserPreferences,
    navigatorLanguage: string | undefined
  ): { uiLocale: 'en' | 'ar'; persist: Partial<UserPreferences> | null }
  ```
  - If `prefs.bootstrapped === true` → `{ uiLocale: prefs.uiLocale, persist: null }` (no write).
  - If `prefs.bootstrapped === false` AND `navigatorLanguage?.startsWith('ar')` → `{ uiLocale: 'ar', persist: { uiLocale: 'ar', bootstrapped: true } }`.
  - If `prefs.bootstrapped === false` AND not Arabic (or `undefined`) → `{ uiLocale: 'en', persist: { bootstrapped: true } }` (mark as bootstrapped without changing the locale).
  - Pure function: no I/O, no globals, no `window.*`. The caller (`layout.tsx` `useEffect`) supplies `navigator.language`.
- **D-09 — `layout.tsx` calls the helper.** The `useEffect` in `RootLayout` does:
  ```ts
  const prefs = await invoke<UserPreferences>('get_user_preferences');
  const { uiLocale, persist } = bootstrapLocale(prefs, navigator.language);
  if (persist) {
    await setUserPreferences(persist);  // single Phase 1 atomic write
  }
  setInitialPreferences({ ...prefs, ...(persist ?? {}), uiLocale, bootstrapped: true });
  ```
  No detection logic lives inline.
- **D-10 — Phase 2 ships exactly these tests** (mirrors Phase 1's T1..T5 discipline):
  - `frontend/src/lib/__tests__/bootstrapLocale.test.ts` (Vitest, pure-function tests):
    - **T2-01:** already-bootstrapped Arabic → returns `{ uiLocale: 'ar', persist: null }`
    - **T2-02:** already-bootstrapped English → returns `{ uiLocale: 'en', persist: null }`
    - **T2-03:** first-run, `navigator.language === 'ar-SA'` → `{ uiLocale: 'ar', persist: { uiLocale: 'ar', bootstrapped: true } }`
    - **T2-04:** first-run, `navigator.language === 'ar'` → same as T2-03
    - **T2-05:** first-run, `navigator.language === 'en-US'` → `{ uiLocale: 'en', persist: { bootstrapped: true } }`
    - **T2-06:** first-run, `navigator.language === undefined` → `{ uiLocale: 'en', persist: { bootstrapped: true } }`
- **D-11 — No new test infrastructure.** If Vitest is not yet configured in the frontend, the planner ships the **minimum** Vitest setup (`vitest.config.ts`, `package.json` script, `@vitest/ui` NOT included) needed to run a single pure-function test file. **No** React Testing Library, no jsdom dependencies for component rendering, no Playwright. Component-level RTL coverage is deferred to the Phase 6 manual regression pass per ROADMAP. The planner verifies during research whether Vitest/Jest exists; if it does, no infra additions are needed.
- **D-12 — Frontend Rust-side tests are unaffected.** Phase 1's T1..T5 Rust tests still pass after the migration adds the `bootstrapped` column (the migration is additive). The planner re-runs `cargo test --package app_lib --test preferences` after the migration lands and confirms green before continuing.

### `LanguageSwitcher` placement in `SettingsModal`

- **D-13 — New "Interface Language" section in the main `SettingsModal` scroll view.** The new `LanguageSwitcher` row lives as a standalone `<h3>` section inside `frontend/src/app/_components/SettingsModal.tsx`, alongside the existing "AI Model Configuration" (`SettingsModal.tsx:87`) and "Audio Device Settings" (`SettingsModal.tsx:173`) sections. Recommended placement: **after "Audio Device Settings" and before any modal-trigger sections at the bottom**. The planner makes the final visual ordering decision.
- **D-14 — Do NOT touch the existing `Language Settings` modal at `SettingsModal.tsx:209-244`.** That modal is for **transcription language** (`selectedLanguage`, `setSelectedLanguage`, `transcriptModelConfig.provider`) and belongs to Phase 4's TRANS scope. Phase 2 plans must explicitly leave it untouched. The new "Interface Language" section in SettingsModal and the existing "Language Settings" modal are two distinct surfaces with no shared components or state.
- **D-15 — Section title copy in English: "Interface Language".** This disambiguates from the transcription-language modal in the user's mental model. The Arabic copy is `"لغة الواجهة"`. Both keys ship in `messages/{en,ar}.json` under `settings.language.sectionTitle` per the UI-SPEC Copywriting Contract — UI-SPEC's existing key already maps to the right copy; no new keys needed.
- **D-16 — Recording-blocker source: `useRecordingState()` from `frontend/src/contexts/RecordingStateContext.tsx`.** The `LanguageConfirmDialog`'s `disabled` state for the confirm button keys on `useRecordingState().isRecording`. Confirmed this hook exists in Phase 1's provider tree (`layout.tsx` `RecordingStateProvider`).

### Claude's Discretion

The planner has freedom on these — they were not deeply specified and don't need user input:

- Exact placement of the "Interface Language" section relative to the other sections in `SettingsModal` (after Audio Device Settings is recommended; planner picks final order based on visual flow)
- Whether the migration adds an explicit `CHECK (bootstrapped IN (0, 1))` constraint or relies on the type system + insertion path
- Whether `bootstrapLocale` returns a discriminated union or two separate fields (functional equivalent — pick whichever reads cleanest in TypeScript)
- Whether Tajawal `weight: ['400', '500']` is loaded with `fallback` arrays or relies on the CSS variable cascade in `globals.css`
- The exact `display: 'swap'` vs `'optional'` choice for Tajawal (UI-SPEC says `swap`; planner verifies this still avoids FOIT in a Tauri WebView)
- Whether the `RootLayout` `useEffect` uses `Promise.all` with the 150ms artificial floor or a separate `setTimeout` chain
- Whether to refactor `RootLayout`'s existing onboarding-completion handler to read from `initialPreferences` (out of scope unless trivially intersecting)

</decisions>

<specifics>
## Specific Ideas

The UI-SPEC at `02-UI-SPEC.md` is the authoritative visual reference — every component, copy string, font choice, and CSS contract is locked there. No additional product references were raised in discussion. The planner should treat the UI-SPEC as canonical for any visual question and the four decision blocks above as canonical for any structural question.

**One non-visual specific worth surfacing:** the BootSplash tagline may briefly read English even on an Arabic first-run because the Arabic catalogue isn't loaded yet. UI-SPEC accepts this. Plans should NOT try to "fix" it by pre-loading Arabic — the fix would inflate the BootSplash time budget for zero user-visible benefit.

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents (researcher, planner) MUST read these before starting their work.**

### Phase 2 design contract (read first, every section)
- `.planning/phases/02-i18n-framework-locale-bootstrap/02-UI-SPEC.md` — The approved UI design contract. Component Inventory, Locale Bootstrap Sequence, BootSplash Visual Contract, Settings Language Switcher Visual Contract, Provider Tree, Font Loading, Applied font family, Copywriting Contract, Out of Scope. **This is the highest-priority document for Phase 2.**

### Primary spec (authoritative for every cross-phase decision)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §3.3 — Root layout integration code example (the `useEffect` pattern this phase implements)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §12.1 — `navigator.language` first-run detection (D-01..D-04 source)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §12.4 — Reload-not-hot-swap on locale switch (D-06 source)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §9 — Out-of-scope list (locks D-13..D-16's "do not touch transcript language modal")

### GSD planning artifacts (authoritative for scope & success criteria)
- `.planning/ROADMAP.md` § "Phase 2: i18n Framework & Locale Bootstrap" — Goal, dependencies, requirements, success criteria
- `.planning/REQUIREMENTS.md` § "UI (Bilingual UI Surface)" UI-01..UI-04 — Requirement text for traceability
- `.planning/PROJECT.md` § "Constraints" — `next-intl` client-provider mode, Tajawal via `next/font/google`, RTL logical-property rule, Arabic translation MSA-only

### Phase 1 outputs this phase consumes
- `.planning/phases/01-preferences-foundation/01-01-SUMMARY.md` — What Phase 1 actually shipped (commits `57baddb`..`8fafa26`), the `preferences::` module shape, `ConfigProvider initialPreferences` wiring, Phase 1 D-17/D-18 deletions
- `frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql` — Phase 1's migration; Phase 2's new migration follows it numerically

### Code touchpoints (file:line precision)

**Modified by Phase 2:**
- `frontend/src/app/layout.tsx:30` — Add `Tajawal` import alongside `Source_Sans_3`
- `frontend/src/app/layout.tsx` (current `<html lang="en">` literal) — Replace with `<html lang={uiLocale} dir={uiLocale === 'ar' ? 'rtl' : 'ltr'} suppressHydrationWarning>`
- `frontend/src/app/layout.tsx` (the `RootLayout` component body) — Add `useState<UserPreferences | null>(null)` for initialPreferences, add `useEffect` calling `get_user_preferences` + `bootstrapLocale`, add `BootSplash` gate, wrap children in `I18nProvider`
- `frontend/src/app/globals.css` — Add `--font-sans-en` / `--font-sans-ar` CSS variables and `html[dir]` font-family selectors per UI-SPEC "Applied font family" section
- `frontend/src/contexts/ConfigContext.tsx` — Surface `bootstrapped: boolean` if it exposes the full `UserPreferences` shape; if it only exposes a subset, no change
- `frontend/src/services/preferencesService.ts` — Update `UserPreferences` TypeScript type to include `bootstrapped: boolean`
- `frontend/src-tauri/src/preferences/repository.rs` — Add `bootstrapped: bool` field to `UserPreferences` struct, update SQL hydration query, update `apply_patch_atomic` to handle the new field in `UserPreferencesPatch`
- `frontend/src-tauri/src/preferences/commands.rs` — Update `UserPreferencesPatch` to include `bootstrapped: Option<bool>` with `#[serde(rename_all = "camelCase")]`
- `frontend/src/app/_components/SettingsModal.tsx` — Add new "Interface Language" `<h3>` section after "Audio Device Settings" (`:173`)
- `frontend/package.json` — Add `next-intl@^3.26.0` and `next/font/google` (already a transitive dep via Source_Sans_3, but verify)

**Created by Phase 2:**
- `frontend/src-tauri/migrations/20260409000000_add_preferences_bootstrapped_flag.sql` — `ALTER TABLE user_preferences ADD COLUMN bootstrapped INTEGER NOT NULL DEFAULT 0;`
- `frontend/src/lib/bootstrapLocale.ts` — Pure function from D-08
- `frontend/src/lib/__tests__/bootstrapLocale.test.ts` — 6-case Vitest suite from D-10
- `frontend/src/providers/I18nProvider.tsx` — Thin wrapper around `NextIntlClientProvider` from D-07
- `frontend/src/components/BootSplash.tsx` — Per UI-SPEC BootSplash Visual Contract
- `frontend/src/components/settings/LanguageSwitcher.tsx` — Per UI-SPEC Settings Language Switcher Visual Contract
- `frontend/src/components/settings/LanguageConfirmDialog.tsx` — Per UI-SPEC Visual Contract; uses `useRecordingState()` from D-16
- `frontend/src/messages/en.json` — 16 keys per UI-SPEC Copywriting Contract message catalogue table
- `frontend/src/messages/ar.json` — 16 keys (MSA Arabic) per same table

**Untouched (the planner enforces this — common over-reach risk):**
- `frontend/src/app/_components/SettingsModal.tsx:209-244` (the existing transcription-language modal — D-14)
- `frontend/src/contexts/RecordingStateContext.tsx` (Phase 2 only **reads** via `useRecordingState()`)
- `frontend/src-tauri/src/preferences/tests.rs` (Phase 1 tests — additive migration must keep them green per D-12; not modified)
- All 65 `.tsx` files containing physical-direction Tailwind classes (Phase 3)
- `BlockNoteEditor/Editor.tsx` and `@blocknote/core/fonts/inter.css` (Phase 3 spike)
- `frontend/src-tauri/src/lib.rs` tray-menu / notification call sites (Phase 6)
- `tailwind.config.ts` (Phase 2 narrows tokens consumed; does not edit)

### Cross-phase contract surface (Phase 2 produces, later phases consume)
- **Phase 3** consumes the `<html dir>` switch from D-06 / UI-SPEC Locale Bootstrap Sequence as the precondition for visually verifying RTL classes
- **Phase 4** consumes the `bootstrap detector → setUserPreferences({ uiLocale: 'ar' })` path as the trigger for the auto-repoint to `localWhisper` + `large-v3` (Phase 1 ships the invariant; Phase 2 ships the trigger; Phase 4 ships the UI surfaces around it)
- **Phase 5** consumes `useTranslations()` and the `messages/` directory shape as the pattern templates/prompts will mirror
- **Phase 6** consumes the `bootstrapped` column for QA-01 desync regression suite assertions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`preferences::read()`** (Phase 1 sync hot-path reader): not used directly by Phase 2 client code (the client uses `get_user_preferences` Tauri command), but `bootstrapped` flowing through this reader means Rust call sites (audio path, transcription) get the new field for free if they ever need it.
- **`preferencesService.getUserPreferences()` / `setUserPreferences()`** (Phase 1 TypeScript service at `frontend/src/services/preferencesService.ts`): the planner **wraps** these in the bootstrap helper rather than calling `invoke` directly. This keeps Phase 2 from coupling to the Tauri API surface.
- **`ConfigProvider` already accepts `initialPreferences` prop** (Phase 1 commit `adb4dc0`): Phase 2 simply passes the result of the bootstrap step into this prop. No `ConfigProvider` API change needed.
- **`useRecordingState()` from `frontend/src/contexts/RecordingStateContext.tsx`**: provides the `isRecording` flag the `LanguageConfirmDialog` uses to disable the confirm button. Already in the provider tree above where `SettingsModal` mounts.
- **shadcn `AlertDialog`, `Button`, `RadioGroup`, `Label`** (`frontend/src/components/ui/*`): all four components Phase 2 needs already exist locally. No `shadcn add` commands required.
- **`next/font/google`** (already used by `Source_Sans_3` at `layout.tsx:30`): the `Tajawal` import follows the identical pattern. Zero new font infrastructure.
- **`window.location.reload()` precedent** (`layout.tsx` `handleOnboardingComplete`): Phase 2's switch flow uses the **same** reload mechanism the existing onboarding completion already uses. Established pattern, no new "exit and restart" logic.
- **Migration sequence convention** (`frontend/src-tauri/migrations/YYYYMMDDHHMMSS_description.sql`): Phase 1's `20260407000000_add_user_preferences.sql` is the latest. Phase 2's new file slots in immediately after.

### Established Patterns
- **Tauri command camelCase bridge** via `#[serde(rename_all = "camelCase")]` on Rust types — Phase 2's `bootstrapped` field follows the same pattern (Rust `bootstrapped` ↔ TS `bootstrapped`; no field name divergence to worry about, but the macro stays for consistency with `uiLocale`).
- **`'use client'` root layout** with `useState` + `useEffect` — Phase 2 extends the existing pattern; does NOT introduce any RSC/server-component code.
- **CSS variables on `<body>` className for fonts** — existing pattern. Phase 2 adds `${tajawal.variable}` alongside `${sourceSans3.variable}`. The dynamic locale-based selection happens in `globals.css` via `html[dir]` selectors, NOT via dynamic body className (per UI-SPEC).
- **shadcn `AlertDialog` for confirmations** — Phase 2's `LanguageConfirmDialog` follows the same pattern other modals in the app already use. No custom dialog primitives.
- **Sonner toaster** (`Toaster` at `layout.tsx`) — Phase 2's error states (`settings.language.error.persistFailed`, `settings.language.error.bootstrapFailed`) call `toast.error(t('...'))` from `sonner`. Already imported, already in the tree.

### Integration Points
- **`RootLayout` ↔ `bootstrapLocale.ts` ↔ Phase 1 `set_user_preferences`** — the bootstrap flow's only Tauri call is `setUserPreferences({ uiLocale, bootstrapped: true })` (when `persist != null`). The Phase 1 atomic transaction handles everything else. No new IPC.
- **`I18nProvider` ↔ `messages/{en,ar}.json`** — `next-intl` loads messages once at provider mount. Phase 2 imports both JSON files at the top of `layout.tsx` (or `I18nProvider.tsx`) and selects via `messages[uiLocale]`. No dynamic import. Bundle impact: ~2KB gzipped per language for the 16 keys; trivial.
- **`SettingsModal` ↔ `LanguageSwitcher` ↔ `LanguageConfirmDialog`** — composition only. The `SettingsModal` adds one new `<h3>` section that mounts `<LanguageSwitcher />`. The switcher manages its own confirm-dialog state (`useState<boolean>(false)` for `confirmOpen`). No SettingsModal state changes.
- **`bootstrapped` column ↔ Phase 1 `apply_patch_atomic`** — the new column flows through the existing atomic transaction (Phase 1 D-07 sequence: read → clone → merge → invariant → tx → commit → cache). Phase 2 does NOT alter the merge order or invariant logic. The `bootstrapped` field is treated as a simple last-write-wins boolean with no invariants.

### Audio pipeline risk (for planner awareness)
None directly. Phase 2 changes only the layout/preferences hydration path, never the audio capture or transcription path. The migration is additive (one new INTEGER column with default 0), so all existing recording-path code continues to read `preferences::read()` unchanged.

</code_context>

<deferred>
## Deferred Ideas

No scope-creep ideas were raised during discussion. The conversation stayed inside the four UI-SPEC-defined deliverables and the four gray-area decisions. The "Out of Scope for Phase 2" table in `02-UI-SPEC.md` remains the authoritative deferral list — read it during planning and again before any implementation commit that feels "while we're at it…".

</deferred>

---

*Phase: 02-i18n-framework-locale-bootstrap*
*Context gathered: 2026-04-09*
*Discussion mode: discuss (4 gray areas surfaced, all 4 selected, recommended option locked on each)*
