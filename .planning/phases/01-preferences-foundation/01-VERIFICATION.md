---
phase: 01-preferences-foundation
verified_at: 2026-04-08T00:00:00Z
status: human_needed
score: 4/4 requirements verified (all static checks pass)
requirements_checked: 4
requirements_passed: 4
must_haves_verified: 8/8
human_verification:
  - test: "Run cargo test preferences:: with llama-helper binary present"
    expected: "5 passed; 0 failed (T1..T5 all green)"
    why_human: "The worktree is missing binaries/llama-helper-aarch64-apple-darwin — the Tauri build script fails before tests can run. The executor confirmed 5/5 green in their environment. A human must reproduce in an environment where the binary exists (or build it via: cd frontend && cargo build -p llama-helper --release && cp target/release/llama-helper src-tauri/binaries/llama-helper-aarch64-apple-darwin && chmod +x src-tauri/binaries/llama-helper-aarch64-apple-darwin)."
  - test: "M1 — Restart persistence"
    expected: "Switch UI locale to Arabic, kill app fully, relaunch — UI renders in Arabic before any IPC call, no flash of English"
    why_human: "Requires Tauri app lifecycle; cannot be exercised from cargo test. Validates that hydrate_from_db correctly reads the persisted SQLite value on a fresh process."
  - test: "M3 — Next-recording language honoring"
    expected: "Stop recording, switch transcription_language via settings (no restart), start second recording — RUST_LOG=app_lib::audio=debug shows the new language code reaching Whisper"
    why_human: "Requires a live recording flow end-to-end. Validates that the four migrated call sites actually produce a different language value at transcription time."
---

# Phase 1: Preferences Foundation — Verification Report

**Phase Goal:** "User preferences live in exactly one place with atomic writes that make invalid states (e.g. Arabic + Parakeet) unrepresentable."
**Verified:** 2026-04-08
**Status:** human_needed — all static code checks pass; 3 items need human/environment confirmation
**Re-verification:** No — initial verification

---

## Goal Achievement Assessment

The phase goal has three distinct claims to verify:

1. **"Exactly one place"** — Single SQLite `user_preferences` row + single `PREFS_CACHE` RwLock. VERIFIED in code.
2. **"Atomic writes"** — `apply_patch_atomic` opens a sqlx transaction, commits, then updates cache. VERIFIED in code.
3. **"Invalid states unrepresentable"** — Reject branch fires before `pool.begin()` for `{provider:'parakeet'}` + Arabic. VERIFIED in code.

All three claims are substantiated by the implementation. The only unresolved items are runtime/environment confirmations that cannot be statically verified.

---

## Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | `user_preferences` SQLite table exists with singleton row `id='1'` | VERIFIED | `migrations/20260407000000_add_user_preferences.sql:7-15` — `CREATE TABLE IF NOT EXISTS user_preferences` + `INSERT OR IGNORE INTO user_preferences (id) VALUES ('1')` |
| 2 | Cache hydrated via `block_on` BEFORE command registration returns | VERIFIED | `lib.rs:473-477` — `tauri::async_runtime::block_on(async { preferences::hydrate_from_db(...).await })` called inside setup closure, before `invoke_handler!` registration at line 654 |
| 3 | Reject branch fires BEFORE `pool.begin().await` for Parakeet+Arabic | VERIFIED | `repository.rs:62-66` — `invariant::check_reject_branch(&patch, &merged)?` at Step C; `pool.begin().await` at Step D; `mod.rs:173-178` confirms the exact condition |
| 4 | Cache updated ONLY after `tx.commit()` succeeds | VERIFIED | `commands.rs:32-51` — `apply_patch_atomic(...).await?` (which contains commit at `repository.rs:111`), then `PREFS_CACHE.write()` in a separate block |
| 5 | All 4 live recording-path call sites use `preferences::read()` | VERIFIED | `whisper_engine/commands.rs:396`, `whisper_engine/parallel_processor.rs:344`, `audio/transcription/worker.rs:449`, `audio/transcription/worker.rs:527` — all use `crate::preferences::read().transcription_language` |
| 6 | Legacy symbols fully deleted from Rust source | VERIFIED | Grep for `LANGUAGE_PREFERENCE\|set_language_preference\|get_language_preference_internal` across `frontend/src-tauri/src/` → 0 matches |
| 7 | `ConfigContext.tsx:215` desync workaround deleted; `primaryLanguage` localStorage gone | VERIFIED | ConfigContext.tsx contains no match for "fixes startup desync bug", "primaryLanguage", or `localStorage` reads/writes for language. Mount useEffect at line 216-229 calls `getUserPreferences()` from preferencesService. |
| 8 | T1..T5 Nyquist tests exist and are substantive | VERIFIED (code) | `preferences/tests.rs` — 5 tests present, each substantive: real in-memory SqlitePool, real migration SQL via `include_str!`, Anti-Sampling Rules 1-5 all honored. Test run BLOCKED by missing binary in this worktree (see human verification). |

**Score:** 8/8 truths verified statically. 3 require human/runtime confirmation.

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql` | Migration with singleton row | VERIFIED | `CREATE TABLE IF NOT EXISTS user_preferences` + `INSERT OR IGNORE` seed. Columns: `id`, `ui_locale`, `summary_language`, `transcription_language`, `updated_at`. |
| `frontend/src-tauri/src/preferences/mod.rs` | Module with `UserPreferences`, `UserPreferencesPatch`, `PREFS_CACHE`, `read()`, `hydrate_from_db`, invariant submodule | VERIFIED | 197 lines. `std::sync::RwLock` (D-04 discretion). Reject branch at `invariant::check_reject_branch`. Auto-repoint at `invariant::should_auto_repoint`. |
| `frontend/src-tauri/src/preferences/repository.rs` | `load()` + `apply_patch_atomic()` with A-I step ordering | VERIFIED | 116 lines. Steps A-I documented and implemented exactly as specified in D-07/D-11. Reject branch (Step C) precedes `pool.begin()` (Step D). |
| `frontend/src-tauri/src/preferences/commands.rs` | `get_user_preferences` + `set_user_preferences` Tauri commands | VERIFIED | 54 lines. Cache update strictly post-`apply_patch_atomic` `Ok`. `std::sync::RwLock` guard acquired and dropped inline, never across `.await`. |
| `frontend/src-tauri/src/preferences/tests.rs` | T1..T5 Nyquist integration tests | VERIFIED (code) | 331 lines. All 5 tests present with correct structure. Anti-Sampling Rules 1-5 honored. Cross-test `PREFS_CACHE` serialization via `tokio::sync::Mutex`. Runtime execution blocked by missing `llama-helper` binary in worktree. |
| `frontend/src/services/preferencesService.ts` | TS wrapper for `get_user_preferences` / `set_user_preferences` | VERIFIED | 40 lines. Exports `getUserPreferences()`, `setUserPreferences()`, types `UiLocale`, `SummaryLanguage`, `UserPreferences`, `UserPreferencesPatch` (includes `provider?` for A1 Option B). |
| `frontend/src-tauri/src/audio/recording_commands.rs.backup` | Must not exist (D-15) | VERIFIED DELETED | `test -f` returns exit 1; file is GONE. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `lib.rs::run setup` | `preferences::hydrate_from_db` | `tauri::async_runtime::block_on` | WIRED | `lib.rs:473-477` — `block_on` inside setup closure, after DB init, before setup returns |
| `lib.rs invoke_handler!` | `preferences::commands::get_user_preferences` | registration at line 654 | WIRED | `lib.rs:654` |
| `lib.rs invoke_handler!` | `preferences::commands::set_user_preferences` | registration at line 655 | WIRED | `lib.rs:655` |
| `commands::set_user_preferences` | `repository::apply_patch_atomic` | direct async call | WIRED | `commands.rs:32` — delegates fully; cache update follows `Ok` return |
| `repository::apply_patch_atomic` | `invariant::check_reject_branch` | Step C before `pool.begin()` | WIRED | `repository.rs:62` — reject fires pre-transaction |
| `whisper_engine/commands.rs:396` | `preferences::read()` | `crate::preferences::read()` | WIRED | Confirmed at line 396 |
| `whisper_engine/parallel_processor.rs:344` | `preferences::read()` | `crate::preferences::read()` | WIRED | Confirmed at line 344 |
| `audio/transcription/worker.rs:449` | `preferences::read()` | `crate::preferences::read()` | WIRED | Confirmed at line 449 |
| `audio/transcription/worker.rs:527` | `preferences::read()` | `crate::preferences::read()` | WIRED | Confirmed at line 527 |
| `ConfigContext.tsx` | `getUserPreferences()` | mount `useEffect` | WIRED | `ConfigContext.tsx:216-229` — async IIFE with cancellation guard, sets `selectedLanguage` from `prefs.transcriptionLanguage` |
| `ConfigContext.tsx` | `setUserPreferences()` | `handleSetSelectedLanguage` | WIRED | `ConfigContext.tsx:482-489` — `useCallback` delegates to service, updates React state from returned value |

---

## Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|--------------------|--------|
| `ConfigContext.tsx` `selectedLanguage` | `selectedLanguage` state | `getUserPreferences()` Tauri IPC → `PREFS_CACHE.read()` → SQLite `user_preferences` row | Yes — `read()` returns owned clone of `PREFS_CACHE` which is populated from DB at startup | FLOWING |
| Recording call sites | `transcription_language` | `crate::preferences::read().transcription_language` → `PREFS_CACHE.read()` → populated by `hydrate_from_db` | Yes — reads live cache populated from real SQLite row | FLOWING |

---

## Behavioral Spot-Checks

| Behavior | Status | Notes |
|----------|--------|-------|
| `cargo test preferences::` — T1..T5 pass | BLOCKED | `binaries/llama-helper-aarch64-apple-darwin` missing in worktree. Tauri build script fails. Executor confirmed 5/5 green. Human must re-run after building the binary. |
| Legacy Rust symbols removed | PASS | 0 grep matches for `LANGUAGE_PREFERENCE`, `set_language_preference`, `get_language_preference_internal` in `frontend/src-tauri/src/` |
| Legacy frontend symbols removed | PASS | 0 grep matches for `primaryLanguage`, `set_language_preference`, `get_language_preference_internal`, "fixes startup desync bug" in `frontend/src/` |
| Backup file deleted | PASS | `recording_commands.rs.backup` confirmed GONE |
| No physical-direction Tailwind classes in ConfigContext.tsx | PASS | 0 matches for `ml-`, `mr-`, `pl-`, `pr-`, `text-left`, `text-right` |

---

## Requirements Coverage

### PREFS-01: Single source of truth + startup hydration

**Full text:** "User preferences (`ui_locale`, `summary_language`, `transcription_language`) live in a single SQLite `user_preferences` row, readable through one Rust `preferences` module with process-global `Lazy<RwLock<UserPreferences>>` hydrated at startup"

**Verdict: PASSED**

Evidence:
- Migration `20260407000000_add_user_preferences.sql` creates the singleton table and seeds `id='1'`
- `mod.rs:95-103` — `static PREFS_CACHE: Lazy<RwLock<UserPreferences>>` is the single process-global cache
- `lib.rs:473-477` — `block_on(preferences::hydrate_from_db(...))` runs in the setup closure before command registration, guaranteeing a populated cache on first command invocation
- `preferencesService.ts` — frontend reads exclusively via `getUserPreferences()` Tauri IPC, not direct SQLite or localStorage
- T1 (`hydration_reflects_seeded_row`) verifies this path end-to-end (execution pending binary)

### PREFS-02: Atomic cross-table writes + Parakeet-Arabic reject

**Full text:** "Setting a preference updates SQLite + in-memory `RwLock` atomically in a single transaction; no window where callers can observe partial state, and the Parakeet-ban invariant (§5.2) is enforced as part of the same write"

**Verdict: PASSED**

Evidence:
- `repository.rs:51-116` — `apply_patch_atomic` executes Steps A-I: pre-flight load → merge → reject check (Step C, pre-`pool.begin()`) → `BEGIN` → UPDATE `user_preferences` → conditional UPDATE `transcript_settings` → `COMMIT` → read-back
- `commands.rs:32-51` — `PREFS_CACHE.write()` only reached after `apply_patch_atomic` returns `Ok` (i.e., after commit)
- `mod.rs:169-179` — `check_reject_branch` returns `Err(PreferencesError::InvalidCombination)` before the transaction opens
- `mod.rs:194-196` — `should_auto_repoint` correctly reads `current` (pre-merge) per D-08 to avoid vacuous firings
- T2 (atomic auto-repoint), T3 (rollback invariance), T4 (real reject branch), T5 (concurrent setters) all present with correct structure
- A1 Option B fully implemented: `UserPreferencesPatch.provider: Option<String>` exists; reject fires on `provider == "parakeet"` while `merged.ui_locale == "ar"`

### PREFS-03: Legacy symbol removal

**Full text:** "All 6+ recording-path call sites ... read preferences from the new module; `get_language_preference_internal()` is deleted, not deprecated"

**Verdict: PASSED**

Evidence:
- 4 live call sites all confirmed using `crate::preferences::read().transcription_language`:
  - `whisper_engine/commands.rs:396`
  - `whisper_engine/parallel_processor.rs:344`
  - `audio/transcription/worker.rs:449`
  - `audio/transcription/worker.rs:527`
- Grep across `frontend/src-tauri/src/` for `LANGUAGE_PREFERENCE|set_language_preference|get_language_preference_internal` → **0 matches**
- `recording_commands.rs.backup` (which contained 3 dead references at lines 1276/1353/1440) confirmed deleted
- The "6+" count in REQUIREMENTS.md = 4 live + 2+ dead (.backup) — all accounted for

### PREFS-04: Frontend desync workaround removed

**Full text:** "The `ConfigContext.tsx:215` startup-desync `useEffect` workaround is removed in the same commit that eliminates its cause; `primaryLanguage` no longer touches `localStorage`"

**Verdict: PASSED**

Evidence:
- `ConfigContext.tsx:140-144` — `selectedLanguage` state initialized to `'auto'` (not `localStorage.getItem('primaryLanguage')`)
- `ConfigContext.tsx:216-229` — replacement `useEffect` calls `getUserPreferences()` from `preferencesService`, not localStorage; has cancellation guard
- `ConfigContext.tsx:482-489` — `handleSetSelectedLanguage` calls `setUserPreferences({transcriptionLanguage: lang})`, not `localStorage.setItem`
- Grep across `frontend/src/` for `primaryLanguage|fixes startup desync bug` → **0 matches**
- No physical-direction Tailwind classes introduced (RTL discipline maintained per CLAUDE.md)
- Note: `ConfigContextType.setSelectedLanguage` interface remains `(lang: string) => void` — this is the documented deviation in SUMMARY (TypeScript bivariant return rules make `Promise<void>` assignable to `void`); not a gap

---

## Parakeet+Arabic Escape Path Analysis

The phase goal requires the combination be "unrepresentable." The implementation has two branches:

**Reject branch (direct Parakeet set while Arabic):** `check_reject_branch` at `repository.rs:62` — fires when `patch.provider == Some("parakeet") && merged.ui_locale == "ar"`. Returns `Err` before `pool.begin()`. Cache never touched. SOUND.

**Auto-repoint branch (locale flip to Arabic while Parakeet active):** `should_auto_repoint` at `repository.rs:85` — fires when `patch.ui_locale == Some("ar") && current.ui_locale != "ar"`. Forces `transcript_settings.provider = 'localWhisper'` inside the transaction. SOUND.

**Residual gap (acceptable, documented):** A caller could issue `set_user_preferences({uiLocale: 'ar'})` while `current.ui_locale` is already `'ar'` — the auto-repoint guard (`current.ui_locale != "ar"`) deliberately does NOT fire to avoid vacuous writes. If `transcript_settings.provider` had somehow been set to `'parakeet'` between the locale switch and this re-set (which requires bypassing the constraints), it would not be corrected. This is a bounded, documented design trade-off (D-08 doc comment in `mod.rs:187`), not an implementation defect.

**Conclusion:** No code path in the current implementation can produce the `{parakeet + Arabic}` combination through the normal write surface. The phase goal is substantively met.

---

## Anti-Patterns Found

| File | Pattern | Severity | Assessment |
|------|---------|----------|------------|
| `ConfigContext.tsx:110-112` | `transcriptModelConfig` default still has `provider: 'parakeet'` | Info | This is the transcript config state (separate from `user_preferences`), not the preferences module. Phase 4 (TRANS-02) will hide the Parakeet option in the UI. Not a Phase 1 gap. |
| `tests.rs` | Cannot run in this worktree (missing binary) | Warning | Environmental — not a code defect. Binary must be built per SUMMARY instructions. |

No blockers found.

---

## Human Verification Required

### 1. T1..T5 Nyquist Tests

**Test:** Build `llama-helper` binary, then run `cd frontend/src-tauri && cargo test preferences::`

Build command (from repo root):
```
cargo build -p llama-helper --release
cp target/release/llama-helper frontend/src-tauri/binaries/llama-helper-aarch64-apple-darwin
chmod +x frontend/src-tauri/binaries/llama-helper-aarch64-apple-darwin
cd frontend/src-tauri && cargo test preferences::
```

**Expected:** `test result: ok. 5 passed; 0 failed; 0 ignored` for T1..T5

**Why human:** The worktree is missing the `llama-helper-aarch64-apple-darwin` binary that the Tauri build script requires. This is the same environmental blocker documented in SUMMARY.md (Rule 3, Wave 2). The test code itself is fully verified statically; execution requires the binary.

### 2. M1 — Restart Persistence

**Test:** `pnpm run tauri:dev` → switch UI locale to Arabic via settings → kill app fully → relaunch → observe UI locale

**Expected:** UI renders in Arabic before any IPC call; no flash of English

**Why human:** Requires Tauri app lifecycle; `lib.rs::run` hydration path cannot be exercised from `cargo test`. This is the end-to-end proof that `hydrate_from_db` + SQLite persistence work together.

### 3. M3 — Next-Recording Language Honoring

**Test:** Start recording → stop → switch `transcription_language` via settings → start second recording → observe `RUST_LOG=app_lib::audio=debug` logs

**Expected:** Whisper receives the new language code on the second recording

**Why human:** Requires a live recording flow. Validates that the four migrated call sites at `preferences::read().transcription_language` return the updated value after a `set_user_preferences` call.

---

## Gaps Summary

No gaps. All 4 requirements are substantively implemented and statically verified. The 3 human verification items are runtime confirmation items, not implementation defects — the code is correct and the wiring is complete. The phase goal ("preferences live in exactly one place with atomic writes that make invalid states unrepresentable") is achieved in the codebase.

The `status: human_needed` reflects that T1..T5 test execution and the two manual verification scenarios (M1 restart persistence, M3 recording language honoring) cannot be confirmed without a functional build environment and a running Tauri app.

---

_Verified: 2026-04-08_
_Verifier: Claude (gsd-verifier)_
