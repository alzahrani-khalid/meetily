---
phase: 02-i18n-framework-locale-bootstrap
plan: 03
subsystem: ui
tags: [next-intl, i18n, shadcn, radix-ui, arabic, rtl, message-catalogue]

# Dependency graph
requires:
  - phase: 01-preferences-foundation
    provides: UserPreferences Tauri commands (get/set), ConfigProvider
provides:
  - "next-intl@3.26.5 installed and pinned to ^3.26.x (D-05)"
  - "I18nProvider wrapper as sole next-intl importer (D-07 single-file coupling)"
  - "useTranslations re-export from @/providers/I18nProvider"
  - "Bilingual message catalogues (en.json, ar.json) with 15 UI-SPEC keys"
  - "shadcn alert-dialog and radio-group primitives for Plan 05"
affects: [04-layout-wiring, 05-language-switcher, 06-qa-regression]

# Tech tracking
tech-stack:
  added: [next-intl@3.26.5, "@radix-ui/react-alert-dialog@1.1.15", "@radix-ui/react-radio-group@1.3.8"]
  patterns: [single-file-coupling-D07, client-provider-only-D06, message-catalogue-nesting]

key-files:
  created:
    - frontend/src/providers/I18nProvider.tsx
    - frontend/src/messages/en.json
    - frontend/src/messages/ar.json
    - frontend/src/components/ui/alert-dialog.tsx
    - frontend/src/components/ui/radio-group.tsx
  modified:
    - frontend/package.json

key-decisions:
  - "Used AbstractIntlMessages type from next-intl instead of Record<string, unknown> for type safety"
  - "Message catalogues ship 15 leaf keys (settings.language.empty excluded per UI-SPEC n/a status)"

patterns-established:
  - "D-07 single-file coupling: only I18nProvider.tsx imports from next-intl; all consumers use @/providers/I18nProvider"
  - "Message catalogue nesting: boot.* and settings.language.* with {lang} interpolation placeholders"

requirements-completed: [UI-02]

# Metrics
duration: 7min
completed: 2026-04-09
---

# Phase 2 Plan 03: i18n Library, Provider Wrapper & Message Catalogues Summary

**next-intl 3.26.5 with single-file I18nProvider wrapper, shadcn alert-dialog/radio-group primitives, and bilingual en/ar message catalogues (15 UI-SPEC keys)**

## Performance

- **Duration:** 7 min
- **Started:** 2026-04-09T07:21:47Z
- **Completed:** 2026-04-09T07:28:32Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Installed next-intl@3.26.5 pinned to ^3.26.x (D-05 enforced, no v4)
- Created I18nProvider.tsx as the sole next-intl importer in the codebase (D-07 single-file coupling verified by grep)
- Shipped bilingual message catalogues with identical 15-key structures in MSA Arabic and English
- Installed shadcn alert-dialog and radio-group primitives for Plan 05's LanguageConfirmDialog and LanguageSwitcher

## Task Commits

Each task was committed atomically:

1. **Task 1: Install next-intl + shadcn primitives + create I18nProvider wrapper** - `82f5971` (feat)
2. **Task 2: Create messages/en.json and messages/ar.json** - `95ee679` (feat)

## I18nProvider Public API

```typescript
// frontend/src/providers/I18nProvider.tsx

export type Locale = 'en' | 'ar';
export type Messages = AbstractIntlMessages;  // from next-intl

interface I18nProviderProps {
  locale: Locale;
  messages: Messages;
  children: ReactNode;
}

export function I18nProvider({ locale, messages, children }: I18nProviderProps)
export { useTranslations } from 'next-intl';  // re-export for D-07 compliance
```

**Consumer pattern for Plan 04 layout.tsx:**
```tsx
import { I18nProvider } from '@/providers/I18nProvider';
<I18nProvider locale={uiLocale} messages={messages[uiLocale]}>
  {children}
</I18nProvider>
```

**Consumer pattern for Plan 05 components:**
```tsx
import { useTranslations } from '@/providers/I18nProvider';
const t = useTranslations('settings.language');
```

## shadcn Primitives Created

| Component | File | Radix Peer Added |
|-----------|------|-----------------|
| AlertDialog | `frontend/src/components/ui/alert-dialog.tsx` | `@radix-ui/react-alert-dialog@1.1.15` |
| RadioGroup | `frontend/src/components/ui/radio-group.tsx` | `@radix-ui/react-radio-group@1.3.8` |

Both generated via `pnpm dlx shadcn@latest add` with `new-york` preset per `components.json`.

## Message Catalogue Keys (15 keys)

| # | Key | English | Arabic (MSA) |
|---|-----|---------|-------------|
| 1 | `boot.appName` | Meetily | Meetily |
| 2 | `boot.loading` | Preparing your meeting assistant... | جاري تجهيز مساعد اجتماعاتك... |
| 3 | `settings.language.sectionTitle` | Language | اللغة |
| 4 | `settings.language.description` | Choose the language Meetily uses... | اختر لغة واجهة Meetily... |
| 5 | `settings.language.option.en` | English | الإنجليزية |
| 6 | `settings.language.option.ar` | العربية | العربية |
| 7 | `settings.language.currentLabel` | Current language | اللغة الحالية |
| 8 | `settings.language.switchCta` | Switch to {lang} | التبديل إلى {lang} |
| 9 | `settings.language.confirm.title` | Switch Meetily to {lang}? | تبديل Meetily إلى {lang}؟ |
| 10 | `settings.language.confirm.body` | Meetily will reload to apply... | سيتم إعادة تشغيل Meetily... |
| 11 | `settings.language.confirm.primaryCta` | Confirm & Reload | تأكيد وإعادة التشغيل |
| 12 | `settings.language.confirm.cancelCta` | Cancel | إلغاء |
| 13 | `settings.language.confirm.recordingBlocker` | Stop the current recording... | أوقف التسجيل الحالي... |
| 14 | `settings.language.error.persistFailed` | Couldn't save language preference... | تعذر حفظ تفضيل اللغة... |
| 15 | `settings.language.error.bootstrapFailed` | Couldn't load preferences... | تعذر تحميل التفضيلات... |

Note: `settings.language.empty` from UI-SPEC excluded (marked n/a -- no empty state in Phase 2).

## Files Created/Modified

- `frontend/src/providers/I18nProvider.tsx` - Sole next-intl wrapper with D-07 single-file coupling
- `frontend/src/messages/en.json` - English message catalogue (15 keys)
- `frontend/src/messages/ar.json` - Arabic MSA message catalogue (15 keys, matching structure)
- `frontend/src/components/ui/alert-dialog.tsx` - shadcn AlertDialog primitive (new-york preset)
- `frontend/src/components/ui/radio-group.tsx` - shadcn RadioGroup primitive (new-york preset)
- `frontend/package.json` - Added next-intl, @radix-ui/react-alert-dialog, @radix-ui/react-radio-group

## Decisions Made

- **AbstractIntlMessages type:** Used `AbstractIntlMessages` from next-intl instead of `Record<string, unknown>` (plan's interface). The plan's type was incompatible with next-intl's `NextIntlClientProvider` props. `AbstractIntlMessages` is the correct opaque type that satisfies the provider's type constraint while remaining opaque to consumers.
- **15 keys not 16:** The plan references "16 keys from UI-SPEC" but the UI-SPEC table includes `settings.language.empty` marked as "n/a -- this phase has no empty state". The plan's own JSON blocks contain exactly 15 leaf keys. Shipped 15.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed Messages type to use AbstractIntlMessages**
- **Found during:** Task 1 (I18nProvider creation)
- **Issue:** Plan specified `export type Messages = Record<string, unknown>` but `unknown` is not assignable to next-intl's `AbstractIntlMessages` type (`string | AbstractIntlMessages`), causing `tsc --noEmit` to fail with TS2322.
- **Fix:** Changed to `export type Messages = AbstractIntlMessages` imported from `next-intl`, combined into the existing import statement to maintain single-import-line.
- **Files modified:** `frontend/src/providers/I18nProvider.tsx`
- **Verification:** `pnpm exec tsc --noEmit` exits 0
- **Committed in:** `82f5971` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Type fix was necessary for TypeScript compilation. No scope creep. The exported `Messages` type remains opaque to consumers.

## Issues Encountered

- **shadcn CLI prompted for overwrites:** The `--yes` flag alone was insufficient; required `--overwrite` flag. RadioGroup showed as "skipped (files might be identical)" but the file was present from a previous shadcn dependency resolution. Both files verified present with correct exports.
- **Node v24 TypeScript mode:** Node v24.9.0 defaults to TypeScript evaluation mode which interfered with `!==` in `-e` inline scripts (shell escaping). Worked around by writing verification scripts to temp files.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- I18nProvider ready for Plan 04 to wire into layout.tsx
- useTranslations re-export ready for Plan 05 LanguageSwitcher/LanguageConfirmDialog
- Message catalogues ready for useTranslations('settings.language') calls
- shadcn alert-dialog and radio-group primitives ready for Plan 05 component composition
- D-07 single-file coupling verified -- all future next-intl access goes through @/providers/I18nProvider

---
*Phase: 02-i18n-framework-locale-bootstrap*
*Plan: 03*
*Completed: 2026-04-09*
