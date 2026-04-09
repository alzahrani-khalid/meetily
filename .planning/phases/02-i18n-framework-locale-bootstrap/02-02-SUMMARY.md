---
phase: 02-i18n-framework-locale-bootstrap
plan: 02
subsystem: i18n
tags: [vitest, typescript, pure-function, locale-detection, tdd]

# Dependency graph
requires:
  - phase: 01-preferences-foundation
    provides: UserPreferences type with uiLocale field
provides:
  - "bootstrapLocale() pure function for first-run locale detection"
  - "Vitest test infrastructure (node environment, @ path alias)"
  - "6-case T2-01..T2-06 test suite covering all detection branches"
affects: [04-layout-integration, 05-settings-switcher]

# Tech tracking
tech-stack:
  added: [vitest@2.1.9]
  patterns: [pure-function-extraction, tdd-red-green, type-only-imports]

key-files:
  created:
    - frontend/src/lib/bootstrapLocale.ts
    - frontend/src/lib/__tests__/bootstrapLocale.test.ts
    - frontend/vitest.config.ts
  modified:
    - frontend/package.json
    - frontend/src/services/preferencesService.ts

key-decisions:
  - "Added bootstrapped field to UserPreferences interface in preferencesService.ts (required for type import)"

patterns-established:
  - "Pure function extraction: locale logic isolated from I/O, testable without mocks"
  - "Vitest node environment: no jsdom/happy-dom for pure-function tests (D-11)"
  - "Type-only imports: import type { UserPreferences } compiles even if source module changes"

requirements-completed: [UI-01]

# Metrics
duration: 5min
completed: 2026-04-09
---

# Phase 2 Plan 02: bootstrapLocale Pure Function + Vitest Suite Summary

**Pure-function locale detector with 6-case TDD suite: detects Arabic navigator on first run, persists bootstrapped flag, never re-detects**

## Performance

- **Duration:** 5 min
- **Started:** 2026-04-09T07:21:22Z
- **Completed:** 2026-04-09T07:27:00Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Shipped `bootstrapLocale(prefs, navigatorLanguage)` pure function with zero I/O, zero globals
- Installed Vitest with minimum footprint (no jsdom, no happy-dom, no @vitest/ui per D-11)
- All 6 T2-01..T2-06 tests pass covering the complete detection truth table
- TypeScript compiles clean (`tsc --noEmit` exits 0)

## Function Signature

```typescript
export function bootstrapLocale(
  prefs: UserPreferences,
  navigatorLanguage: string | undefined,
): BootstrapResult;

export interface BootstrapResult {
  uiLocale: 'en' | 'ar';
  persist: Partial<UserPreferencesPatch> | null;
}
```

Plan 04's `layout.tsx` imports this verbatim.

## Test Suite (T2-01..T2-06)

| Test  | prefs.bootstrapped | prefs.uiLocale | navigatorLanguage | Expected uiLocale | Expected persist                       |
|-------|--------------------|----------------|-------------------|--------------------|----------------------------------------|
| T2-01 | true               | 'ar'           | 'en-US'           | 'ar'               | null                                   |
| T2-02 | true               | 'en'           | 'ar-SA'           | 'en'               | null                                   |
| T2-03 | false              | 'en'           | 'ar-SA'           | 'ar'               | { uiLocale: 'ar', bootstrapped: true } |
| T2-04 | false              | 'en'           | 'ar'              | 'ar'               | { uiLocale: 'ar', bootstrapped: true } |
| T2-05 | false              | 'en'           | 'en-US'           | 'en'               | { bootstrapped: true }                 |
| T2-06 | false              | 'en'           | undefined         | 'en'               | { bootstrapped: true }                 |

## Vitest Installation Footprint

**Installed:** `vitest@^2.1.9` (devDependency only)

**Excluded per D-11:**
- `@vitest/ui` -- not needed for pure-function tests
- `jsdom` -- no DOM testing in this plan
- `happy-dom` -- no DOM testing in this plan
- `@testing-library/react` -- no component tests
- `@testing-library/dom` -- no DOM tests

**Configuration:** `vitest.config.ts` with `environment: 'node'`, `globals: false`, `@` path alias matching tsconfig.

## Runtime Dependencies

The `bootstrapLocale` function has ZERO runtime dependencies beyond the `UserPreferences` type import (erased at compile time via `import type`). The function body uses only JavaScript builtins (`typeof`, `String.prototype.toLowerCase`, `String.prototype.startsWith`).

## Task Commits

Each task was committed atomically:

1. **Task 1: Install Vitest + config + RED test suite** - `ae6ef50` (test)
2. **Task 2: GREEN -- implement bootstrapLocale.ts pure function** - `423cbdd` (feat)

## Files Created/Modified
- `frontend/src/lib/bootstrapLocale.ts` - Pure-function locale detector (D-08)
- `frontend/src/lib/__tests__/bootstrapLocale.test.ts` - 6-case Vitest suite (D-10)
- `frontend/vitest.config.ts` - Minimum Vitest config, node environment (D-11)
- `frontend/package.json` - Added vitest devDep + test script
- `frontend/src/services/preferencesService.ts` - Added bootstrapped field to UserPreferences

## Decisions Made
- Added `bootstrapped: boolean` field to `UserPreferences` interface in `preferencesService.ts` since Plan 01 had not yet added it. This is required for the type import to be complete.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added bootstrapped field to UserPreferences interface**
- **Found during:** Task 1 (test suite creation)
- **Issue:** `preferencesService.ts` did not have `bootstrapped` field on `UserPreferences` -- needed for type import and test fixtures
- **Fix:** Added `bootstrapped: boolean` to the `UserPreferences` interface
- **Files modified:** `frontend/src/services/preferencesService.ts`
- **Verification:** `tsc --noEmit` passes, all tests pass
- **Committed in:** ae6ef50 (Task 1 commit)

**2. [Rule 3 - Blocking] Cleared macOS quarantine on esbuild binary**
- **Found during:** Task 1 (RED verification)
- **Issue:** esbuild binary had macOS quarantine attribute causing EPERM spawn error
- **Fix:** Removed com.apple.quarantine xattr from the esbuild darwin-arm64 binary
- **Files modified:** None (binary attribute only)
- **Verification:** `pnpm test` runs successfully after clearing

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `bootstrapLocale` is ready for Plan 04's `layout.tsx` to import and call
- Vitest infrastructure is ready for any future pure-function tests
- The `bootstrapped` field addition to `UserPreferences` will need to align with the Rust backend type when Plan 01's migration lands

---
*Phase: 02-i18n-framework-locale-bootstrap*
*Completed: 2026-04-09*
