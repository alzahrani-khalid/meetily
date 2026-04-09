---
phase: 02-i18n-framework-locale-bootstrap
plan: 01
subsystem: preferences
tags: [prefs, sqlite, rust, tauri, bootstrapped, i18n]
one_liner: "Additive bootstrapped INTEGER column + Rust/TS type extension so Phase 2 first-run detector can mark detection done exactly once per install"
requirements: [UI-01, UI-03]
dependency_graph:
  requires:
    - "Phase 1 preferences module (01-01-SUMMARY.md)"
    - "Phase 1 migration 20260407000000_add_user_preferences.sql"
  provides:
    - "UserPreferences.bootstrapped: bool field over Tauri IPC"
    - "bootstrapped column in user_preferences SQLite table"
    - "setUserPreferences({ bootstrapped: true }) atomic write path"
  affects:
    - "Phase 2 Plan 02 (bootstrapLocale helper) — can now read bootstrapped: boolean from UserPreferences"
    - "Phase 6 (QA-01) — bootstrapped column available for regression assertions"
key-files:
  created:
    - "frontend/src-tauri/migrations/20260409000000_add_preferences_bootstrapped_flag.sql"
  modified:
    - "frontend/src-tauri/src/preferences/mod.rs"
    - "frontend/src-tauri/src/preferences/repository.rs"
    - "frontend/src-tauri/src/preferences/tests.rs"
    - "frontend/src/services/preferencesService.ts"
decisions:
  - "2026-04-09 (exec) — tests.rs modified with ..Default::default(): The plan specified tests.rs should remain UNMODIFIED, but Phase 1 tests construct UserPreferencesPatch with explicit field initialization (no ..Default::default()). Adding the bootstrapped: Option<bool> field to the struct is a compilation-breaking change for those constructors. Fixed by adding ..Default::default() to each struct literal — semantically identical (None default for Option<bool>), preserves all T1..T5 test logic and assertions. Also added Phase 2 migration include_str! so the in-memory test DB has the bootstrapped column."
  - "2026-04-09 (exec) — No CHECK constraint on bootstrapped column per D-Discretion #2: SQLite stores booleans as INTEGER 0/1 by convention. The sole writer is apply_patch_atomic; a CHECK constraint adds no safety over the type system."
metrics:
  duration_minutes: ~8
  tasks_completed: 2
  commits: 2
  tests_passing: 5
  completed: 2026-04-09
---

# Phase 2 Plan 01: Bootstrapped Preferences Flag Summary

## What Shipped

An additive SQLite migration adds a `bootstrapped INTEGER NOT NULL DEFAULT 0` column to the `user_preferences` table, and the Rust `UserPreferences` struct + TypeScript `UserPreferences` interface are extended to expose it over the Tauri IPC boundary as `bootstrapped: boolean`.

**Why a column, not re-detect-on-every-launch (D-01..D-04):** The `bootstrapped` flag makes the first-run detection event a single, persistent fact. Without it, a user who deliberately picks English on a US-locale machine and later moves to a Saudi-locale machine would silently flip to Arabic on the next launch. That is a correctness bug the column prevents.

## Rust Struct Shape (post-change)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    #[serde(skip_deserializing)]
    pub id: String,
    pub ui_locale: String,
    pub summary_language: String,
    pub transcription_language: String,
    pub updated_at: i64,
    pub bootstrapped: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesPatch {
    #[serde(default)] pub ui_locale: Option<String>,
    #[serde(default)] pub summary_language: Option<String>,
    #[serde(default)] pub transcription_language: Option<String>,
    #[serde(default)] pub provider: Option<String>,
    #[serde(default)] pub bootstrapped: Option<bool>,
}
```

## TypeScript Type Addition

```typescript
export interface UserPreferences {
  uiLocale: UiLocale;
  summaryLanguage: SummaryLanguage;
  transcriptionLanguage: string;
  bootstrapped: boolean;
}
```

`UserPreferencesPatch` inherits `bootstrapped?: boolean` via `Partial<UserPreferences>`.

## Phase 1 T1..T5 Test Results (unchanged logic, all green)

```
test preferences::tests::hydration_reflects_seeded_row ... ok
test preferences::tests::atomic_write_auto_repoints_parakeet ... ok
test preferences::tests::concurrent_setters_serialize ... ok
test preferences::tests::rollback_leaves_cache_and_row_unchanged ... ok
test preferences::tests::reject_parakeet_while_arabic ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out
```

The additive migration is transparent to all existing test assertions. Tests were updated only to include the Phase 2 migration SQL in the in-memory DB setup and to use `..Default::default()` for the new struct field.

## Commit Sequence

| # | Hash | Message |
|---|------|---------|
| 1 | `fa76e3e` | feat(prefs): UI-01 UI-03 add bootstrapped column migration + Rust struct field |
| 2 | `90ccb6f` | feat(prefs): UI-01 UI-03 wire bootstrapped into repository SELECT/UPDATE + TypeScript types |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] tests.rs struct construction incompatible with new field**
- **Found during:** Task 2
- **Issue:** Phase 1 tests construct `UserPreferencesPatch` with explicit field enumeration (no `..Default::default()`). Adding `bootstrapped: Option<bool>` to the struct causes "missing field `bootstrapped`" compilation errors in all 5 test functions.
- **Fix:** Added `..Default::default()` to each `UserPreferencesPatch` struct literal in tests.rs and added the Phase 2 migration `include_str!` + execution to `test_pool_with_migration()`. All T1..T5 test assertions remain identical.
- **Files modified:** `frontend/src-tauri/src/preferences/tests.rs`
- **Commit:** `90ccb6f`

**2. [Rule 3 - Blocker] cmake not in PATH**
- **Found during:** Task 1 verification (cargo check)
- **Fix:** `brew install cmake` (cmake 4.3.1). Environmental fix, not committed.

**3. [Rule 3 - Blocker] Missing llama-helper binary**
- **Found during:** Task 1 verification (cargo check)
- **Fix:** Built via `cargo build -p llama-helper --release` and copied to `frontend/src-tauri/binaries/`. Environmental fix, not committed.

**4. [Rule 3 - Blocker] pnpm not installed in worktree**
- **Found during:** Task 2 TypeScript verification
- **Fix:** `npm install -g pnpm` via nvm node v22, then `pnpm install` in frontend/. Environmental fix, not committed.

## Read-for-Next-Plan Hint

Plan 02 (bootstrapLocale helper) can now import `UserPreferences` from `@/services/preferencesService` and see the `bootstrapped: boolean` field. The helper function should check `prefs.bootstrapped === true` to skip detection on subsequent launches, and call `setUserPreferences({ uiLocale, bootstrapped: true })` to mark detection as done.

## Self-Check: PASSED
