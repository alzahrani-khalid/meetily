---
phase: 02-i18n-framework-locale-bootstrap
plan: 04
subsystem: ui
tags: [layout, tajawal, rtl, bootstrap, i18n-provider, font-loading, css-variables]
one_liner: "Root layout wired with Tajawal font, bootstrap useEffect calling bootstrapLocale(), I18nProvider in provider tree, dynamic html lang/dir, CSS font variables for direction-based font switching"
requirements: [UI-01, UI-03, UI-04]
dependency_graph:
  requires:
    - "Plan 01: UserPreferences.bootstrapped field (Rust + TS)"
    - "Plan 02: bootstrapLocale() pure function"
    - "Plan 03: I18nProvider wrapper + message catalogues"
  provides:
    - "Root layout with dynamic <html lang dir> based on detected locale"
    - "Bootstrap useEffect that runs bootstrapLocale() exactly once on mount"
    - "I18nProvider mounted between ConfigProvider and OllamaDownloadProvider"
    - "ConfigProvider.initialPreferences prop for seeding state without double-fetch"
    - "CSS font variables --font-sans-en and --font-sans-ar with html[dir] selectors"
    - "BootSplash stub component (Plan 05 replaces with visual contract)"
  affects:
    - "Plan 05 (LanguageSwitcher + BootSplash visual) — relies on I18nProvider being mounted"
    - "Phase 3+ (all UI components) — can now use useTranslations() from I18nProvider"
key-files:
  created:
    - "frontend/src/components/BootSplash.tsx"
  modified:
    - "frontend/src/app/layout.tsx"
    - "frontend/src/app/globals.css"
    - "frontend/src/contexts/ConfigContext.tsx"
decisions:
  - "2026-04-09 (exec) — Bootstrap error toast is hard-coded English (not t('...')), because it runs BEFORE I18nProvider mounts. This is intentional per UI-SPEC error state: app falls back to en, user told why."
  - "2026-04-09 (exec) — Source Sans 3 weights narrowed from [400,500,600,700] to [400,600] per Phase 2 2-weight contract. Tajawal uses [400,500] (Google Fonts does not publish Tajawal 600)."
  - "2026-04-09 (exec) — BootSplash created as stub (Option A from plan). Plan 05 will replace with full UI-SPEC visual contract."
metrics:
  duration_minutes: ~6
  tasks_completed: 2
  commits: 2
  completed: 2026-04-09
---

# Phase 2 Plan 04: Root Layout Integration Summary

**Root layout wired with Tajawal font, bootstrap useEffect calling bootstrapLocale(), I18nProvider in provider tree, dynamic html lang/dir, CSS font variables for direction-based font switching**

## Performance

- **Duration:** ~6 min
- **Started:** 2026-04-09T07:34:35Z
- **Completed:** 2026-04-09T07:40:00Z
- **Tasks:** 2/2
- **Files modified:** 4

## Accomplishments

- ConfigProvider now accepts `initialPreferences?: UserPreferences` prop; seeds `selectedLanguage` state from it and short-circuits the mount `getUserPreferences()` call when provided
- globals.css declares `--font-sans-en` and `--font-sans-ar` CSS variables in `:root`, plus `html[dir="ltr"] body` and `html[dir="rtl"] body` font-family selectors
- layout.tsx imports Tajawal (arabic subset, weights 400/500) alongside Source Sans 3 (narrowed to 400/600)
- Bootstrap `useEffect` calls `bootstrapLocale(prefs, navigator.language)` and persists the result atomically via `setUserPreferences(persist)`
- `<html lang={uiLocale} dir={uiLocale === 'ar' ? 'rtl' : 'ltr'} suppressHydrationWarning>` replaces hardcoded `<html lang="en">`
- I18nProvider inserted between ConfigProvider and OllamaDownloadProvider in the provider tree
- Provider tree gated by `initialPreferences ? (...providers...) : <BootSplash />`
- BootSplash stub created at `frontend/src/components/BootSplash.tsx` for Plan 05 to replace

## Provider Tree (post-Plan 04)

```
<html lang={uiLocale} dir={...} suppressHydrationWarning>
  <body className="${sourceSans3.variable} ${tajawal.variable} font-sans antialiased">
    {initialPreferences ? (
      <AnalyticsProvider>
        <RecordingStateProvider>
          <TranscriptProvider>
            <ConfigProvider initialPreferences={initialPreferences}>
              <I18nProvider locale={uiLocale} messages={messages[uiLocale]}>
                <OllamaDownloadProvider>
                  ...rest of Phase 1 tree unchanged...
                </OllamaDownloadProvider>
              </I18nProvider>
            </ConfigProvider>
          </TranscriptProvider>
        </RecordingStateProvider>
      </AnalyticsProvider>
    ) : (
      <BootSplash />
    )}
    <Toaster position="bottom-center" richColors closeButton />
  </body>
</html>
```

## ConfigProvider initialPreferences Contract

```typescript
interface ConfigProviderProps {
  children: ReactNode;
  initialPreferences?: UserPreferences;
}

// When initialPreferences is provided:
// 1. selectedLanguage seeded from initialPreferences.transcriptionLanguage
// 2. Mount effect short-circuits (no getUserPreferences() call)
// When initialPreferences is undefined (backward compatible):
// Phase 1 fetch-on-mount behavior preserved
```

## CSS Contract (globals.css additions)

```css
:root {
  --font-sans-en: var(--font-source-sans-3), ui-sans-serif, system-ui, sans-serif;
  --font-sans-ar: var(--font-tajawal), var(--font-source-sans-3), ui-sans-serif, system-ui, sans-serif;
}

html[dir="ltr"] body { font-family: var(--font-sans-en); }
html[dir="rtl"] body { font-family: var(--font-sans-ar); }
```

## Bootstrap Error Toast Note

The layout.tsx bootstrap error toast uses a hard-coded English string ("Couldn't load preferences. Starting in English...") rather than `t('settings.language.error.bootstrapFailed')` because the error occurs BEFORE I18nProvider mounts (chicken-and-egg: we need prefs to know the locale). This is intentional per UI-SPEC copywriting contract: the en-fallback code path speaks English.

## Task Commits

| # | Hash | Message |
|---|------|---------|
| 1 | `ef2a254` | feat(i18n): UI-01 UI-03 add initialPreferences prop to ConfigProvider + CSS font variables |
| 2 | `3f7a95f` | feat(i18n): UI-01 UI-03 UI-04 wire Tajawal font, bootstrap useEffect, I18nProvider in layout |

## Files Created/Modified

- `frontend/src/contexts/ConfigContext.tsx` -- Added initialPreferences prop, type import, mount effect gate
- `frontend/src/app/globals.css` -- Added --font-sans-en/ar CSS variables and html[dir] body selectors
- `frontend/src/app/layout.tsx` -- Tajawal import, bootstrap useEffect, I18nProvider wiring, dynamic html lang/dir
- `frontend/src/components/BootSplash.tsx` -- Stub component (Plan 05 replaces)

## Deviations from Plan

None -- plan executed exactly as written. The ConfigContext.tsx had exactly the state variables the plan anticipated (`selectedLanguage` seeded from `transcriptionLanguage`), and the mount effect was located where the plan expected (lines 216-229).

## Known Stubs

| File | Description | Resolved by |
|------|-------------|-------------|
| `frontend/src/components/BootSplash.tsx` | Minimal loading stub with "Loading Meetily..." text | Plan 05 replaces with UI-SPEC visual contract |

## Self-Check: PASSED
