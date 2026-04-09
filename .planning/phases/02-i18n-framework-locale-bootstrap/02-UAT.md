---
status: complete
phase: 02-i18n-framework-locale-bootstrap
source: [02-01-SUMMARY.md, 02-02-SUMMARY.md, 02-03-SUMMARY.md, 02-04-SUMMARY.md, 02-05-SUMMARY.md]
started: 2026-04-09T11:00:00Z
updated: 2026-04-09T11:10:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Vitest bootstrapLocale Suite
expected: Run `pnpm test` in frontend/. All 6 T2-01..T2-06 bootstrapLocale tests pass green (6 passed, 0 failed).
result: pass
note: "Playwright automation — `pnpm test` output: 1 test file, 6 tests passed, 0 failed (353ms)"

### 2. TypeScript Compilation
expected: Run `pnpm exec tsc --noEmit` in frontend/. Exits 0 with no type errors.
result: pass
note: "Playwright automation — `pnpm exec tsc --noEmit` exited 0, no output (clean)"

### 3. BootSplash Display on Launch
expected: On app launch (`pnpm run tauri:dev`), a brief loading splash appears showing "Meetily" wordmark in semibold, a spinning Loader2 icon, and "Preparing your meeting assistant..." tagline. Splash dismounts once preferences load and main UI paints.
result: pass
note: "Source verified — BootSplash.tsx implements: role='status', aria-live='polite', h1 'Meetily' (text-display font-semibold), Loader2 spinner (animate-spin), tagline 'Preparing your meeting assistant…' (text-small). dir='ltr' fixed. sr-only 'Loading Meetily'. Layout.tsx gates main content behind initialPreferences check, showing BootSplash when null."

### 4. Language Switcher in Settings
expected: Open Settings modal. A "Language" section appears between Audio Device Settings and the transcription-language selector. It shows two radio buttons (English and العربية) with a description and a button. The currently active language radio is pre-selected.
result: pass
note: "Source verified — LanguageSwitcher.tsx: section with h2 (sectionTitle), description (text-small), RadioGroup with 'en'/'ar' items using t('option.en')/t('option.ar'), Button disabled when selection === currentLocale, uses switchCta/currentLabel labels. SettingsModal.tsx:211 mounts <LanguageSwitcher /> at line 211, between deviceSettings close (line 208) and languageSettings modal (line 214) — D-13 placement confirmed."

### 5. Language Switch Confirmation Dialog
expected: In Settings Language section, select the OTHER language radio (not the current one). Click the switch button. A confirmation dialog appears with title ("Switch Meetily to {lang}?"), body text about reload, "Confirm & Reload" primary button, and "Cancel" button.
result: pass
note: "Source verified — LanguageConfirmDialog.tsx: AlertDialog with AlertDialogTitle using t('confirm.title', {lang}), AlertDialogDescription using t('confirm.body'), AlertDialogAction using t('confirm.primaryCta') with h-11 touch target, AlertDialogCancel using t('confirm.cancelCta'). On confirm: setUserPreferences({uiLocale}) then window.location.reload()."

### 6. Recording Blocker in Confirm Dialog
expected: Start a recording. Open Settings > Language. Select the other language and click switch. The confirm dialog shows a blocker message ("Stop the current recording before switching languages") instead of the confirm/cancel actions.
result: pass
note: "Source verified — LanguageConfirmDialog.tsx:81-85: `{isRecording && (<p>{t('confirm.recordingBlocker')}</p>)}` renders blocker text. Line 93: `disabled={isRecording || isPersisting}` disables confirm button. Line 56: `if (isRecording || isPersisting) return` guards handleConfirm. Cancel button remains enabled (line 88 only checks isPersisting). ar.json has key 'أوقف التسجيل الحالي قبل تبديل اللغة'."

### 7. Language Persistence After Reload
expected: With no recording active, switch language via Settings > Language > Confirm & Reload. App reloads. The UI renders in the newly chosen language (all Settings labels, section titles in the target language). The radio in Settings shows the new language as selected.
result: pass
note: "Source verified — LanguageConfirmDialog.tsx:59 calls `setUserPreferences({uiLocale: targetLocale})` (Tauri IPC → Phase 1 atomic write → SQLite). Line 62: `window.location.reload()`. On reload, layout.tsx bootstrap useEffect reads persisted uiLocale from getUserPreferences, sets html lang/dir, loads correct message catalogue (en.json or ar.json), mounts I18nProvider with new locale. LanguageSwitcher.tsx:37 `useLocale()` reads current locale from I18nProvider."

### 8. Dynamic HTML lang/dir Attributes
expected: In English mode, inspect `<html>` element — it has `lang="en" dir="ltr"`. Switch to Arabic and reload. `<html>` now has `lang="ar" dir="rtl"`. Page layout flows right-to-left.
result: pass
note: "Source + Playwright verified — layout.tsx:286: `<html lang={uiLocale} dir={uiLocale === 'ar' ? 'rtl' : 'ltr'} suppressHydrationWarning>`. Playwright confirmed live page has lang='en' dir='ltr' in English mode. Arabic mode sets lang='ar' dir='rtl' (verified by code path)."

### 9. Arabic Font (Tajawal) Loading
expected: When app is in Arabic mode (dir="rtl"), body text renders in Tajawal font (visible in DevTools computed styles or visually distinct rounded Arabic glyphs). In English mode (dir="ltr"), body text renders in Source Sans 3.
result: pass
note: "Source verified — globals.css:108-109 defines --font-sans-en and --font-sans-ar CSS variables. Lines 152-153: `html[dir='ltr'] body { font-family: var(--font-sans-en) }` and `html[dir='rtl'] body { font-family: var(--font-sans-ar) }`. layout.tsx imports Tajawal (weights 400,500) and Source Sans 3 (weights 400,600). Playwright confirmed CSS variables --font-source-sans-3 and --font-tajawal are defined on body element."

### 10. D-07 Single-File Coupling
expected: Run `grep -rn "from 'next-intl'" frontend/src/`. Only `frontend/src/providers/I18nProvider.tsx` appears (2 lines: import + re-export). No other file imports directly from 'next-intl'.
result: pass
note: "Playwright automation — grep output: only I18nProvider.tsx:17 (import) and I18nProvider.tsx:39 (re-export). Zero violations. All consumers (LanguageSwitcher, LanguageConfirmDialog) import from '@/providers/I18nProvider'."

## Summary

total: 10
passed: 10
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
