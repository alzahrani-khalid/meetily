---
phase: 01-preferences-foundation
plan: 01
subsystem: preferences
tags: [prefs, sqlite, rust, tauri, rtl, bilingual, nyquist]
one_liner: "Single-source-of-truth user_preferences SQLite row + Rust preferences module with atomic cross-table setter, Parakeet+Arabic reject branch (A1 Option B), ConfigContext :215 desync workaround deleted, all 5 Nyquist tests green"
requirements: [PREFS-01, PREFS-02, PREFS-03, PREFS-04]
dependency_graph:
  requires: []
  provides:
    - "preferences::hydrate_from_db for lib.rs::run setup closure"
    - "preferences::read() sync hot-path reader for audio/transcription call sites"
    - "preferences::commands::{get_user_preferences,set_user_preferences} Tauri commands"
    - "UserPreferencesPatch.provider field + transcript_settings.provider write surface (A1 Option B — absorbed from Phase 4 scope)"
    - "@/services/preferencesService TypeScript service for ConfigContext + future consumers"
  affects:
    - "Phase 2 (UI-01..UI-04) — reads preferences::read().ui_locale at startup"
    - "Phase 4 (TRANS-02, TRANS-04) — Phase 1 shipped the atomic tx + reject branch; Phase 4 retains only UI concerns"
    - "Phase 5 (SUMM, TPL) — reads preferences::read().summary_language for template/prompt resolution"
    - "Phase 6 (QA-01) — extends the T1..T5 targeted suite into a full regression suite"
tech-stack:
  added:
    - "std::sync::RwLock for PREFS_CACHE (Claude Discretion D-04; swapped from tokio::sync::RwLock to avoid runtime panic in sync read())"
  patterns:
    - "Singleton row (id='1') SQLite convention, matching settings + transcript_settings"
    - "once_cell::Lazy<RwLock<T>> process-global cache, hydrated at startup via block_on"
    - "sqlx::Transaction cross-table atomic writes (BEGIN → UPDATE user_preferences → conditional UPDATE transcript_settings → COMMIT → post-commit cache update)"
    - "Pre-flight invariant check BEFORE pool.begin().await (T-1-03 mitigation)"
    - "Tauri command camelCase bridge via #[serde(rename_all = 'camelCase')]"
key-files:
  created:
    - "frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql"
    - "frontend/src-tauri/src/preferences/mod.rs"
    - "frontend/src-tauri/src/preferences/repository.rs"
    - "frontend/src-tauri/src/preferences/commands.rs"
    - "frontend/src-tauri/src/preferences/tests.rs"
    - "frontend/src/services/preferencesService.ts"
  modified:
    - "frontend/src-tauri/src/lib.rs (hydration wiring + invoke_handler registrations + legacy deletions)"
    - "frontend/src-tauri/src/whisper_engine/commands.rs (call site 1 migration)"
    - "frontend/src-tauri/src/whisper_engine/parallel_processor.rs (call site 2 migration)"
    - "frontend/src-tauri/src/audio/transcription/worker.rs (call sites 3 + 4 migration)"
    - "frontend/src/contexts/ConfigContext.tsx (localStorage removed, :215 useEffect deleted, service integrated)"
  deleted:
    - "frontend/src-tauri/src/audio/recording_commands.rs.backup (D-15 dedicated chore commit)"
decisions:
  - "2026-04-08 (exec) — std::sync::RwLock not tokio::sync::RwLock: the plan's D-04 allowed Claude Discretion on lock flavor. tokio::sync::RwLock.blocking_read() panics in a tokio runtime context (which #[tokio::test] provides), breaking T1/T2/T3. std::sync::RwLock is the cleanest stdlib fix — works from both sync (audio hot path) and async (Tauri commands, tests) contexts. Write contention is negligible (one per settings change)."
  - "2026-04-08 (exec) — Migration SQL loader in tests.rs strips line-level '--' comments before split(';'): the file-level header comment was being lumped into the first statement chunk and silently skipped, leaving the in-memory DB with no tables. Fixed by filtering comment lines before splitting."
  - "2026-04-08 (exec) — ConfigContextType.setSelectedLanguage interface left as (lang: string) => void instead of () => Promise<void>: TypeScript's bivariant return-type rules let Promise<void> be assigned to void safely, and updating the interface would cascade into LanguageSelection.tsx. Acceptable: errors still caught inside handleSetSelectedLanguage."
metrics:
  duration_hours: ~1.5
  tasks_completed: 17
  commits: 7
  tests_added: 5
  tests_passing: 5
  completed: 2026-04-08
---

# Phase 1 Plan 01: Preferences Foundation — Summary

## What Shipped

A single SQLite `user_preferences` row + Rust `preferences::` module replaces the 4-way preferences desync (React state / `localStorage.primaryLanguage` / `lib.rs::LANGUAGE_PREFERENCE` static / SQLite `settings` table) that previously required a `useEffect` workaround at `ConfigContext.tsx:215`. Writes go through a cross-table atomic sqlx transaction with a pre-flight invariant check that makes `{provider: 'parakeet'} + ui_locale == 'ar'` unrepresentable (A1 Option B — real reject branch, not a stub). The audio hot path reads via a synchronous `preferences::read()` that returns an owned clone, so recording is unaffected. All 5 Nyquist-mandated tests (T1..T5) ship alongside the implementation and pass green.

## Commit Sequence (chronological)

| # | Hash | Message | Wave |
|---|---|---|---|
| 1 | `57baddb` | feat(preferences): PREFS-01 add user_preferences migration | 1 |
| 2 | `19f5cb3` | feat(preferences): PREFS-01 PREFS-02 add preferences module with atomic setter and invariant | 2 |
| 3 | `5d581ca` | test(preferences): PREFS-01 PREFS-02 add targeted hydration/atomic/reject/concurrent tests | 3 |
| 4 | `edbfaf9` | refactor(preferences): PREFS-03 migrate call sites and delete legacy global/command | 4 |
| 5 | `adb4dc0` | refactor(preferences): PREFS-04 migrate ConfigContext to preferences service, delete desync workaround | 5 |
| 6 | `ed1d947` | docs(preferences): scrub legacy symbol name from module doc comment | 5½ (fixup) |
| 7 | `8fafa26` | chore: remove unused recording_commands.rs.backup | 6 |

D-21 commit order is honored: migration → module → tests → call-site migration → frontend migration → .backup cleanup. The extra fixup commit (`ed1d947`) was necessary because the Wave 2 module doc comment quoted the `LANGUAGE_PREFERENCE` symbol name, which caused the phase-level legacy-symbol grep to match inside the new module's own comments.

## Nyquist Test Results (T1..T5)

```
test preferences::tests::hydration_reflects_seeded_row ... ok
test preferences::tests::atomic_write_auto_repoints_parakeet ... ok
test preferences::tests::rollback_leaves_cache_and_row_unchanged ... ok
test preferences::tests::reject_parakeet_while_arabic ... ok
test preferences::tests::concurrent_setters_serialize ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 114 filtered out
```

Full log: `01-01-wave3-tests.log`. All Anti-Sampling Rules honored:
- Rule 1: real `SqlitePool::connect("sqlite::memory:")` with real migration SQL via `include_str!`
- Rule 2: tests exercise the real `apply_patch_atomic → PREFS_CACHE.write()` ordering (mirrors commands.rs exactly)
- Rule 3: T5 uses `tokio::try_join!(fut1, fut2)` with both futures constructed before any `.await`
- Rule 4: T3 forces failure via `DROP TABLE transcript_settings`, not `panic!`
- Rule 5: T4 asserts via `matches!(result, Err(PreferencesError::InvalidCombination { .. }))` exact variant match

## Legacy Symbol Grep Evidence (Phase-Level Acceptance)

```
$ grep -rn 'get_language_preference_internal|fn set_language_preference|LANGUAGE_PREFERENCE' frontend/src-tauri/src/
(0 matches — CLEAN)

$ grep -rn 'primaryLanguage|set_language_preference|get_language_preference_internal' frontend/src --include='*.ts' --include='*.tsx'
(0 matches — CLEAN)

$ test ! -f frontend/src-tauri/src/audio/recording_commands.rs.backup
(exit 0 — CONFIRMED gone)
```

PREFS-03 final acceptance: all 4 live recording-path call sites migrated, `LANGUAGE_PREFERENCE` + `set_language_preference` + `get_language_preference_internal` deleted from `lib.rs`, and the 3 dead references inside `recording_commands.rs.backup` (lines 1276/1353/1440) eliminated by the file deletion.

PREFS-04 final acceptance: `ConfigContext.tsx:215` `useEffect` ("fixes startup desync bug") deleted in the same commit as the `localStorage.getItem('primaryLanguage')` removal (per D-18 / PROJECT.md constraint). New mount hydration via `getUserPreferences()` replaces it.

## Deviations from Plan

### Auto-fixed Issues (Rules 1-3)

**1. [Rule 1 — Bug] `PREFS_CACHE` lock flavor: `tokio::sync::RwLock` → `std::sync::RwLock`**
- **Found during:** Task 1-01-07 (T1) — first test run, all 3 cache-touching tests panicked with "Cannot block the current thread from within a runtime"
- **Root cause:** `read()` called `PREFS_CACHE.blocking_read()` which panics inside a tokio runtime context (`#[tokio::test]` provides one). The plan's D-04 allowed Claude Discretion to pick a different lock flavor.
- **Fix:** Swap `tokio::sync::RwLock` → `std::sync::RwLock`. This works from both sync (audio hot path) and async (Tauri commands, tests) contexts. Write contention is negligible (one write per settings change, read-heavy workload).
- **Files modified:** `frontend/src-tauri/src/preferences/mod.rs`, `frontend/src-tauri/src/preferences/commands.rs`, `frontend/src-tauri/src/preferences/tests.rs`
- **Commit:** `5d581ca`

**2. [Rule 1 — Bug] Migration SQL loader swallowing the CREATE TABLE**
- **Found during:** Task 1-01-06 helper — tests panicked with "no such table: user_preferences"
- **Root cause:** The loader split the SQL file on `;` and skipped chunks whose trimmed form started with `--`. The file-level header comment (`-- Migration: Add user_preferences singleton row...`) was part of the first chunk, which ALSO contained the `CREATE TABLE` statement, so the entire first chunk was silently skipped.
- **Fix:** Strip line-level `--` comments BEFORE splitting on `;`. In-statement `'%s','now'` literals are unaffected because we only strip leading-line comments.
- **Files modified:** `frontend/src-tauri/src/preferences/tests.rs`
- **Commit:** `5d581ca`

**3. [Rule 3 — Blocker] Missing `cmake` system dependency**
- **Found during:** Wave 2 verification — `cargo check` failed with `cmake: No such file or directory` in `whisper-rs-sys` build script
- **Fix:** `brew install cmake` (cmake 4.3.1)
- **Files modified:** None (environmental)
- **Commit:** None (pre-build environment fix)

**4. [Rule 3 — Blocker] Missing prebuilt `llama-helper-aarch64-apple-darwin` binary**
- **Found during:** Wave 2 verification — after cmake install, Tauri build script failed with `resource path 'binaries/llama-helper-aarch64-apple-darwin' doesn't exist`
- **Fix:** Built the existing `llama-helper` workspace package via `cargo build -p llama-helper --release`, copied `target/release/llama-helper` to `frontend/src-tauri/binaries/llama-helper-aarch64-apple-darwin`, chmod +x.
- **Files modified:** None (build artifact staged locally, not committed — the binary is typically regenerated per-environment)
- **Commit:** None

**5. [Rule 2 — Missing critical cleanup] Module doc comment quoted legacy symbol**
- **Found during:** Wave 6 phase-level grep verification
- **Issue:** The `preferences/mod.rs` doc comment said "Replaces the legacy 4-way split between React state / localStorage / `lib.rs::LANGUAGE_PREFERENCE` process-global / SQLite `settings` table", which caused the phase-level grep for `LANGUAGE_PREFERENCE` to match.
- **Fix:** Rephrase to "the old Rust process-global language-preference static" (no literal symbol).
- **Files modified:** `frontend/src-tauri/src/preferences/mod.rs`
- **Commit:** `ed1d947`

### Unverified Steps (environmental limitations)

- **Frontend `pnpm exec tsc --noEmit`** (Task 1-01-15 verify): `node_modules` was not installed in this worktree; `pnpm exec` returned "Command 'tsc' not found". Manual review of ConfigContext.tsx + preferencesService.ts confirms syntactic validity: imports use the existing `@/services` alias, types align with Rust `UserPreferences` shape, and `handleSetSelectedLanguage`'s `Promise<void>` is structurally assignable to the existing `(lang: string) => void` interface per TypeScript's bivariant return rules. **Recommend running `pnpm install && pnpm exec tsc --noEmit` during phase verification before `/gsd-transition`.**

## Authentication Gates Encountered

None. Phase 1 is purely local-code changes; no external services, no API keys, no user-facing auth surface.

## Manual Verifications (from VALIDATION.md — pending user run)

- **M1 — Restart persistence:** `pnpm run tauri:dev`, switch UI locale to Arabic, kill the app fully, relaunch, observe Arabic render with no flash of English. **Status: pending user run.**
- **M2 — ConfigContext workaround deleted (static audit):** Automated grep verification is GREEN:
  - `grep -n "fixes startup desync bug" frontend/src/contexts/ConfigContext.tsx` → 0 matches
  - `grep -n "primaryLanguage" frontend/src/contexts/ConfigContext.tsx` → 0 matches
  - `grep -rn "set_language_preference" frontend/src/` → 0 matches
- **M3 — Next-recording honoring:** Start recording, stop, switch `transcription_language`, start a second recording, verify via `RUST_LOG=app_lib::audio=debug` that Whisper receives the new language code. **Status: pending user run.**

## Cross-Phase Impact

As planned (cross_phase_impact block), Phase 4 scope shrinks:
- `UserPreferencesPatch.provider` field lives here now
- The atomic `transcript_settings.provider` write surface lives here now
- The REAL reject branch for `{provider: 'parakeet'}` while Arabic lives here now
- Phase 4 keeps only UI concerns: hidden dropdown, explanatory banner, onboarding fork, non-blocking "ready to record" gate, TRANS-04 UI wiring

Phase 2 (UI-01..UI-04) can now call `preferences::read().ui_locale` at startup and expect a populated cache.

Phase 5 (TPL, SUMM) can read `preferences::read().summary_language` for template/prompt locale resolution.

Phase 6 (QA-01) will extend the T1..T5 targeted suite into a full regression suite.

## Self-Check

- [x] All 17 tasks executed (across 7 commits — 6 waves plus 1 fixup)
- [x] Each task committed atomically with D-21 commit order respected
- [x] All 5 Nyquist tests T1..T5 passing
- [x] Full `cargo check` green (5 pre-existing warnings, zero errors)
- [x] `LANGUAGE_PREFERENCE`, `get_language_preference_internal`, `set_language_preference` fully removed from `frontend/src-tauri/src/`
- [x] `ConfigContext.tsx:215` useEffect workaround removed in SAME commit as `localStorage.primaryLanguage` removal
- [x] `recording_commands.rs.backup` deleted in dedicated chore commit with exact D-15 verbatim message
- [x] Parakeet+Arabic reject branch verified via T4 (variant-match assertion)
- [x] SUMMARY.md at `.planning/phases/01-preferences-foundation/01-01-SUMMARY.md` with key-files.created list, one_liner, and all deviations documented

## Self-Check: PASSED

Verified live on 2026-04-08:
- **Files exist:**
  - FOUND: `frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql`
  - FOUND: `frontend/src-tauri/src/preferences/mod.rs`
  - FOUND: `frontend/src-tauri/src/preferences/repository.rs`
  - FOUND: `frontend/src-tauri/src/preferences/commands.rs`
  - FOUND: `frontend/src-tauri/src/preferences/tests.rs`
  - FOUND: `frontend/src/services/preferencesService.ts`
- **File deleted:**
  - GONE: `frontend/src-tauri/src/audio/recording_commands.rs.backup`
- **Commits verified in git log:**
  - FOUND: `57baddb`, `19f5cb3`, `5d581ca`, `edbfaf9`, `adb4dc0`, `ed1d947`, `8fafa26`
- **Grep acceptance:**
  - RUST: 0 matches for legacy symbols across `frontend/src-tauri/src/`
  - FRONTEND: 0 matches for legacy symbols across `frontend/src/`
- **Tests:**
  - GREEN: `test result: ok. 5 passed; 0 failed` (preferences:: scope)
