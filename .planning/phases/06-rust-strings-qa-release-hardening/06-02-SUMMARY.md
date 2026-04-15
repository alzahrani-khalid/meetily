---
phase: "06"
plan: "02"
subsystem: preferences, summary
tags: [qa, regression-tests, locale, parakeet-ban, fallback]
dependency_graph:
  requires: [01-01, 05-01, 05-02]
  provides: [QA-01, QA-02, QA-03]
  affects: []
tech_stack:
  added: []
  patterns: [tokio-test-mutex-serialization, variant-match-assertion, 3-case-fallback-matrix]
key_files:
  created: []
  modified:
    - frontend/src-tauri/src/preferences/tests.rs
    - frontend/src-tauri/src/summary/prompts/loader.rs
    - frontend/src-tauri/src/summary/templates/loader.rs
decisions:
  - "Worktree required merge from main to access Phase 5 locale-aware APIs (get_prompt, get_template with locale param)"
  - "3 pre-existing audio test failures (device_detection, vad) confirmed unrelated to changes -- hardware/environment dependent"
metrics:
  duration: "4min"
  completed: "2026-04-15"
  tasks: 2
  files: 3
---

# Phase 6 Plan 02: QA-01/QA-02/QA-03 Automated Regression Tests Summary

11 new regression tests covering preference desync, Parakeet-ban enforcement, and template/prompt locale fallback

## Task Summary

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | QA-01 and QA-02 regression tests (T6-T10) | 5e01ff1 | preferences/tests.rs |
| 2 | QA-03 template/prompt fallback tests | 1048547 | prompts/loader.rs, templates/loader.rs |

## What Was Built

### Task 1: QA-01 + QA-02 Preference Regression Tests (T6-T10)

5 new async tests appended to `preferences/tests.rs` after existing T1-T5:

- **T6** (`qa01_startup_ar_locale_hydrates_correctly`): Seeds `ui_locale='ar'` before hydration, verifies `read()` returns `"ar"`. Strengthens T1 with explicit QA-01 binding.
- **T7** (`qa01_runtime_locale_switch_visible_in_cache`): Hydrates with default `"en"`, applies patch to `"ar"`, mirrors commands.rs cache ordering, asserts `read()` reflects switch.
- **T8** (`qa01_concurrent_locale_setters_no_partial_state`): Two concurrent `apply_patch_atomic` calls (`"ar"` vs `"en"`) via `try_join!` with 2s timeout. Final DB state must be one of the two inputs.
- **T9** (`qa02_en_to_ar_switch_repoints_parakeet`): Seeds parakeet provider while `"en"`, switches to `"ar"`, verifies provider auto-repointed away from parakeet.
- **T10** (`qa02_direct_parakeet_ar_patch_rejected`): Sets locale to `"ar"`, then attempts `provider: "parakeet"` patch. Asserts `InvalidCombination` variant match.

### Task 2: QA-03 Template/Prompt Fallback Tests

6 new synchronous tests implementing the D-06 3-case fallback matrix:

**prompts/loader.rs** (3 tests):
- `qa03_ar_locale_returns_ar_content`: AR prompt contains Arabic comma (U+060C)
- `qa03_unknown_locale_falls_back_to_en`: `"fr"` locale returns EN content (no Arabic comma)
- `qa03_nonexistent_id_returns_error`: Unknown ID returns error containing "not found"

**templates/loader.rs** (3 tests):
- `qa03_ar_locale_returns_ar_template`: AR template name contains Arabic Unicode chars (U+0600..U+06FF)
- `qa03_unknown_locale_falls_back_to_en`: `"fr"` locale returns EN template (no Arabic chars in name)
- `qa03_nonexistent_id_returns_error`: Unknown template ID returns error

## Verification Results

- `cargo test --lib preferences::tests`: 10/10 passed (T1-T5 existing + T6-T10 new)
- `cargo test --lib summary`: 47/47 passed (all prompt and template tests)
- `cargo test --lib`: 148 passed, 3 failed (pre-existing audio hardware tests), 3 ignored

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Worktree missing Phase 5 code**
- **Found during:** Pre-execution setup
- **Issue:** Worktree branch was behind `main` and lacked Phase 5 locale-aware APIs (`get_prompt(id, locale)`, `get_template(id, locale)`)
- **Fix:** Merged `main` into worktree branch to bring in Phase 5 commits
- **Files modified:** None (git merge only)

**2. [Rule 3 - Blocking] Missing llama-helper binary**
- **Found during:** Task 1 verification
- **Issue:** `binaries/llama-helper-aarch64-apple-darwin` missing in worktree (gitignored binary)
- **Fix:** Copied from main repo working directory
- **Files modified:** None (gitignored binary)

## Known Stubs

None -- all tests exercise real code paths with real assertions.

## Self-Check: PASSED

- [x] `preferences/tests.rs` contains all 5 new QA-01/QA-02 test functions
- [x] `prompts/loader.rs` contains all 3 QA-03 test functions
- [x] `templates/loader.rs` contains all 3 QA-03 test functions
- [x] Commit 5e01ff1 exists
- [x] Commit 1048547 exists
