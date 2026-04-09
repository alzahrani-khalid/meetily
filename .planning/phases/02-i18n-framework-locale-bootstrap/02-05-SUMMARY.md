---
phase: 02-i18n-framework-locale-bootstrap
plan: 05
subsystem: ui
tags: [bootsplash, language-switcher, confirm-dialog, settings, rtl, i18n, shadcn]
one_liner: "BootSplash visual contract, LanguageSwitcher with RadioGroup, LanguageConfirmDialog with recording blocker and reload flow, mounted in SettingsModal per D-13"
requirements: [UI-02, UI-03]
dependency_graph:
  requires:
    - "Plan 03: I18nProvider + useTranslations re-export, shadcn alert-dialog + radio-group"
    - "Plan 04: Root layout with I18nProvider mounted, BootSplash stub, ConfigProvider initialPreferences"
  provides:
    - "BootSplash full UI-SPEC visual contract (replaces Plan 04 stub)"
    - "LanguageSwitcher settings section with RadioGroup and confirm flow"
    - "LanguageConfirmDialog with recording blocker and setUserPreferences + reload"
    - "SettingsModal D-13 integration: Interface Language section between Audio Device Settings and transcription-language modal"
  affects:
    - "Phase 3+ (RTL conversion) — new components already use logical-property classes"
    - "Phase 6 QA — LanguageSwitcher is the primary language-change surface"
tech-stack:
  added: []
  patterns: [useLocale-re-export-from-I18nProvider, recording-blocker-pattern]
key-files:
  created:
    - frontend/src/components/settings/LanguageSwitcher.tsx
    - frontend/src/components/settings/LanguageConfirmDialog.tsx
  modified:
    - frontend/src/components/BootSplash.tsx
    - frontend/src/app/_components/SettingsModal.tsx
    - frontend/src/providers/I18nProvider.tsx
decisions:
  - "2026-04-09 (exec) — Used useLocale() from I18nProvider instead of useConfig().uiLocale: ConfigContext does not expose uiLocale (it only has selectedLanguage for transcription). Re-exported useLocale from next-intl via I18nProvider.tsx to maintain D-07 single-file coupling."
metrics:
  duration_minutes: ~3
  tasks_completed: 3
  commits: 3
  completed: 2026-04-09
---

# Phase 2 Plan 05: BootSplash Visual + LanguageSwitcher + LanguageConfirmDialog Summary

**BootSplash visual contract, LanguageSwitcher with RadioGroup, LanguageConfirmDialog with recording blocker and reload flow, mounted in SettingsModal per D-13**

## Performance

- **Duration:** ~3 min
- **Started:** 2026-04-09T07:42:19Z
- **Completed:** 2026-04-09T07:45:42Z
- **Tasks:** 3/3
- **Files modified:** 5

## Accomplishments

- Replaced Plan 04 BootSplash stub with full UI-SPEC visual contract: wordmark (text-display font-semibold), Loader2 spinner, tagline (text-small font-normal), role="status", aria-live="polite", fixed dir="ltr"
- Created LanguageSwitcher with shadcn RadioGroup (2 locale options), primary Button with switchCta label, and LanguageConfirmDialog mount
- Created LanguageConfirmDialog with shadcn AlertDialog, recording-blocker via useRecordingState().isRecording, setUserPreferences + window.location.reload() confirm flow, error toast on persist failure
- Mounted LanguageSwitcher in SettingsModal at D-13 position: after Audio Device Settings close (line 208) and before transcription-language modal open (line 214)
- Transcription-language modal (D-14) verified unchanged via git diff
- D-07 single-file coupling preserved: only I18nProvider.tsx imports from 'next-intl'
- Zero physical-direction Tailwind classes in all new/modified components
- 4-size/2-weight typography contract enforced (14/16/24/32 sizes, 400/600 weights only)

## Task Commits

| # | Hash | Message |
|---|------|---------|
| 1 | `7098feb` | feat(i18n): UI-02 UI-03 implement BootSplash visual contract |
| 2 | `9c1c49e` | feat(i18n): UI-02 UI-03 implement LanguageSwitcher and LanguageConfirmDialog |
| 3 | `6918687` | feat(i18n): UI-02 UI-03 D-13 mount LanguageSwitcher in SettingsModal |

## BootSplash Implementation

BootSplash renders BEFORE I18nProvider mounts, so it uses **inlined English literals** matching `en.json` — no `useTranslations` import. This is intentional per UI-SPEC: the brief English flash on an Arabic first-run is acceptable because the splash dismounts before the real Arabic UI paints.

Strings used:
- Wordmark: "Meetily" (matches `boot.appName`)
- Tagline: "Preparing your meeting assistant..." (matches `boot.loading`)
- SR-only: "Loading Meetily"

## LanguageSwitcher Keys Used

All strings via `useTranslations('settings.language')`:
- `t('sectionTitle')` — section heading
- `t('description')` — description text
- `t('option.en')` / `t('option.ar')` — radio option labels
- `t('currentLabel')` — button label when selection matches current
- `t('switchCta', { lang: targetLanguageName })` — button label when different

## LanguageConfirmDialog Keys Used

All strings via `useTranslations('settings.language')`:
- `t('confirm.title', { lang: targetLanguageName })` — dialog title
- `t('confirm.body')` — dialog body
- `t('confirm.primaryCta')` — confirm button
- `t('confirm.cancelCta')` — cancel button
- `t('confirm.recordingBlocker')` — shown when isRecording is true
- `t('error.persistFailed')` — error toast on setUserPreferences failure

## SettingsModal Insertion (post-edit line numbers)

- Line 174: `<h3>Audio Device Settings</h3>` (inside deviceSettings modal)
- Line 208: `)}` closes `{modals.deviceSettings && (...)}`
- Line 211: `<LanguageSwitcher />` -- NEW (D-13 placement)
- Line 214: `{modals.languageSettings && (` -- UNCHANGED (D-14 preserved)

## D-07 Single-File Coupling Verification

```
$ grep -rn "from 'next-intl'" frontend/src/
frontend/src/providers/I18nProvider.tsx:17:import { NextIntlClientProvider, type AbstractIntlMessages } from 'next-intl';
frontend/src/providers/I18nProvider.tsx:39:export { useTranslations, useLocale } from 'next-intl';
```

Only I18nProvider.tsx imports from 'next-intl'. All consumers import from `@/providers/I18nProvider`.

## Files Created/Modified

- `frontend/src/components/BootSplash.tsx` -- Full UI-SPEC visual contract (replaced Plan 04 stub)
- `frontend/src/components/settings/LanguageSwitcher.tsx` -- RadioGroup with 2 locale options + primary switch button
- `frontend/src/components/settings/LanguageConfirmDialog.tsx` -- AlertDialog with recording blocker + reload flow
- `frontend/src/app/_components/SettingsModal.tsx` -- Added import + LanguageSwitcher mount at D-13 position
- `frontend/src/providers/I18nProvider.tsx` -- Added useLocale re-export for D-07 compliance

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Used useLocale() instead of useConfig().uiLocale**
- **Found during:** Task 2 (LanguageSwitcher creation)
- **Issue:** Plan specified `useConfig() as { uiLocale?: UiLocale }` but ConfigContext does not expose a `uiLocale` field. It only has `selectedLanguage` (transcription language). The plan's Step D anticipated this possibility.
- **Fix:** Re-exported `useLocale` from `next-intl` via `I18nProvider.tsx` (maintaining D-07 single-file coupling). LanguageSwitcher calls `useLocale()` to get the current locale from the I18nProvider, which receives it from layout.tsx's bootstrap state.
- **Files modified:** `frontend/src/providers/I18nProvider.tsx`, `frontend/src/components/settings/LanguageSwitcher.tsx`
- **Committed in:** `9c1c49e`

**2. [Rule 3 - Blocking] Removed useConfig import from LanguageSwitcher**
- **Found during:** Task 2
- **Issue:** Plan specified `import { useConfig } from '@/contexts/ConfigContext'` but since uiLocale is not in ConfigContext, the import is unnecessary.
- **Fix:** Replaced with `import { useTranslations, useLocale } from '@/providers/I18nProvider'` which provides both hooks needed.
- **Impact:** LanguageSwitcher no longer depends on ConfigContext, only on I18nProvider. This is architecturally cleaner.
- **Committed in:** `9c1c49e`

---

**Total deviations:** 2 auto-fixed (both Rule 3 blocking issues, anticipated by plan Step D)
**Impact on plan:** Locale source changed from ConfigContext to I18nProvider. Same data (current uiLocale), different access path. All acceptance criteria still met except the exact `import { useConfig }` line (replaced with `useLocale` approach).

## Known Stubs

None. All components are fully implemented with real data sources.

## Self-Check: PASSED

All 5 files verified present. All 3 commit hashes verified in git log.
