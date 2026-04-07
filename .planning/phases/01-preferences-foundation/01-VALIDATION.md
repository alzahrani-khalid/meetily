---
phase: 1
slug: preferences-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-07
updated: 2026-04-07
plan_bound: 01-01-PLAN.md
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.
>
> **Source:** Derived from `01-RESEARCH.md` § Validation Architecture (lines 693–770)
> and CONTEXT D-22 (targeted Phase 1 test matrix).
>
> **Scope note:** This is Phase 1's *targeted* test matrix — the minimum to de-risk
> the migration per ROADMAP risk note #2. The full regression suite (QA-01) is
> Phase 6 and is explicitly out of scope here.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in `cargo test` harness with `#[tokio::test]` for async (no external framework) |
| **Config file** | None — uses Rust defaults; `tokio = "1.32"` with `full` features already in `frontend/src-tauri/Cargo.toml` |
| **Quick run command** | `cd frontend/src-tauri && cargo test preferences::` |
| **Full suite command** | `cd frontend/src-tauri && cargo test` |
| **Estimated runtime** | Quick: ~3–8s (scoped, in-memory SQLite). Full: bounded by existing `src-tauri` test count. |
| **Test DB strategy** | `sqlx::SqlitePool::connect("sqlite::memory:")` per test, migration SQL run against the in-memory pool, seed row inserted inline. **No mocks.** |
| **Shared helper** | `async fn test_pool_with_migration() -> SqlitePool` in `preferences/tests.rs` (created by task 1-01-06) |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend/src-tauri && cargo test preferences::` (quick scope).
- **After every plan wave:** Run `cd frontend/src-tauri && cargo test` (full suite) to catch regressions in migrated call sites.
- **Before `/gsd-verify-work`:** Full suite must be green AND `preferences::` scope must be green AND all 5 D-22 tests (T1..T5) must be present and passing.
- **Max feedback latency:** 10 seconds for the quick scope; ~60 seconds for the full suite (acceptable given phase risk).

---

## Per-Task Verification Map

> Bound to `01-01-PLAN.md` task IDs. Format: `{phase}-{plan}-{task_num}` (e.g., `1-01-07`).
> The `plan-checker` flips `nyquist_compliant: true` once binding is reviewed.

| Test ID | Plan | Wave | Task ID | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|---------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| T1 (hydration) | 01-01 | 3 | 1-01-07 | PREFS-01 | — | Startup reads `ui_locale='ar'` from seeded SQLite row → `preferences::read().ui_locale == "ar"` immediately after `hydrate_from_db` returns | integration | `cd frontend/src-tauri && cargo test preferences::tests::hydration_reflects_seeded_row -- --test-threads=1` | ❌ W0 (created by 1-01-06) | ⬜ pending |
| T2 (atomic auto-repoint) | 01-01 | 3 | 1-01-08 | PREFS-02 | T-1-02 | `set_user_preferences({ui_locale:'ar'})` while `transcript_settings.provider='parakeet'` → BOTH rows updated in one commit, `RwLock` updated AFTER commit, `read().ui_locale=="ar"` | integration | `cd frontend/src-tauri && cargo test preferences::tests::atomic_write_auto_repoints_parakeet -- --test-threads=1` | ❌ W0 (created by 1-01-06) | ⬜ pending |
| T3 (rollback invariance) | 01-01 | 3 | 1-01-09 | PREFS-02 | T-1-02 | Force the `transcript_settings` UPDATE to fail via `DROP TABLE transcript_settings` → `user_preferences` row unchanged AND `PREFS_CACHE` unchanged AND error flows through `Result` (not `panic!` — Anti-Sampling Rule #4) | integration | `cd frontend/src-tauri && cargo test preferences::tests::rollback_leaves_cache_and_row_unchanged -- --test-threads=1` | ❌ W0 (created by 1-01-06) | ⬜ pending |
| T4 (REAL reject branch) | 01-01 | 3 | 1-01-10 | PREFS-02 | T-1-03 | Direct patch `{provider:'parakeet'}` while `ui_locale=='ar'` → `Err(PreferencesError::InvalidCombination { .. })` matched by variant (Anti-Sampling Rule #5), BEFORE `pool.begin().await` is called. **A1 resolved 2026-04-07: Option B — REAL test, `UserPreferencesPatch` ships with `provider` field this phase.** | integration | `cd frontend/src-tauri && cargo test preferences::tests::reject_parakeet_while_arabic -- --test-threads=1` | ❌ W0 (created by 1-01-06) | ⬜ pending |
| T5 (concurrent setter) | 01-01 | 3 | 1-01-11 | PREFS-02 | T-1-04 | `tokio::try_join!(set1, set2)` with both futures constructed before any `.await` (Anti-Sampling Rule #3), 2-second deadlock-bounded timeout, `multi_thread` flavor → no partial state, final result equals one of the two inputs | integration | `cd frontend/src-tauri && cargo test preferences::tests::concurrent_setters_serialize -- --test-threads=1` | ❌ W0 (created by 1-01-06) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Note on T2 Threat Ref correction:** The previous draft of this table listed T2's
Threat Ref as T-1-01 (filesystem tampering). T-1-01 is `accept`-dispositioned (out of
scope for automated test) so it cannot bind to a test row. The correct binding is
T-1-02 (cache-then-commit ordering), which T2 mitigates via the post-commit cache update
ordering enforced inside `set_user_preferences`. T3 also binds to T-1-02 because
rollback invariance is the failure mode that exposes ordering inversions.

**Coverage matrix — requirements to tests:**

| Requirement | Success Criterion | Covered By |
|-------------|-------------------|------------|
| PREFS-01 | Criterion 1 (UI locale survives restart, no `useEffect` rehydrate) | T1 (1-01-07) + manual M1 (restart persistence) |
| PREFS-02 | Criterion 2 (transcription_language honored on next recording) | T2 (1-01-08) + T3 (1-01-09) atomic + T5 (1-01-11) concurrent + manual M3 |
| PREFS-02 | Criterion 3 (parakeet + arabic rejected before SQLite touched) | T2 (1-01-08) auto-repoint branch + T4 (1-01-10) REAL reject branch |
| PREFS-02 | Criterion 4 (no observable window where SQLite and RwLock disagree) | T2 (1-01-08) post-commit ordering + T3 (1-01-09) rollback invariance + T5 (1-01-11) concurrent window |
| PREFS-03 | (implicit — call sites read new module) | Compilation gate in 1-01-12, 1-01-13 + full `cargo test` (existing tests exercise migrated sites) + grep acceptance in 1-01-13 |
| PREFS-04 | (implicit — `ConfigContext.tsx:215` useEffect deleted, `localStorage` gone) | Manual M2 (frontend audit) + grep-verified acceptance in 1-01-15, 1-01-16 |

---

## Wave 0 Requirements

- [ ] `frontend/src-tauri/src/preferences/tests.rs` — created by task 1-01-06 (test scaffolding) before any T1..T5 task runs
- [ ] `frontend/src-tauri/src/preferences/mod.rs` — created by task 1-01-02 in Wave 2; tests.rs is its child module via `#[cfg(test)] mod tests;` declared in mod.rs
- [ ] Shared helper `async fn test_pool_with_migration() -> SqlitePool` — implemented in task 1-01-06: creates `sqlite::memory:`, runs the migration SQL via `include_str!("../../migrations/20260407000000_add_user_preferences.sql")`, seeds a minimal `transcript_settings` shim sufficient for T2/T3/T4
- [ ] **No framework install required** — `tokio::test` available via existing `tokio = "1.32"` `full` features
- [ ] **No new `Cargo.toml` entries required**

**Wave 0 dependency chain:** 1-01-01 (migration SQL) → 1-01-02 (mod.rs declares `#[cfg(test)] mod tests;`) → 1-01-06 (tests.rs + helper) → 1-01-07..1-01-11 (T1..T5).

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| M1: Restart persistence | PREFS-01 (criterion 1) | Tauri app lifecycle + SQLite file write — cannot exercise `run()` from `cargo test` | 1) `pnpm run tauri:dev`  2) Switch UI locale to Arabic via settings  3) Close app fully (not reload)  4) Relaunch  5) Assert: UI renders in Arabic BEFORE any network/IPC call, no flash of English |
| M2: ConfigContext workaround deleted | PREFS-04 | Static audit — no test can prove a `useEffect` *doesn't exist* | `grep -n "fixes startup desync bug" frontend/src/contexts/ConfigContext.tsx` returns 0 matches; `grep -n "primaryLanguage" frontend/src/contexts/ConfigContext.tsx` returns 0 matches; `grep -rn "set_language_preference" frontend/src/` returns 0 matches |
| M3: Next-recording honoring | PREFS-02 (criterion 2) | Requires a real recording flow end-to-end | 1) Start recording  2) Stop  3) Switch `transcription_language` via settings (no app restart)  4) Start a second recording  5) Assert: Whisper receives the new language code (verify via Rust logs with `RUST_LOG=app_lib::audio=debug`) |

---

## Anti-Sampling Rules (from RESEARCH § Anti-Sampling)

These rule out tests that would falsely pass. Plan-checker must verify each plan task that binds to T1..T5 honors these:

1. **No mocked SQLite rows.** Every test uses `sqlx::SqlitePool::connect("sqlite::memory:")` and runs the real migration SQL via `include_str!`. A `sqlx::query!` with a mocked row would pass even if the transaction logic is broken. **Bound to:** 1-01-06 helper.
2. **No direct `PREFS_CACHE` inspection.** Tests must call `set_user_preferences` via the real code path (or `repository::apply_patch_atomic` + post-commit cache write that mirrors `commands.rs` ordering exactly). Reading `PREFS_CACHE` directly would miss commit-then-cache ordering inversions. **Bound to:** 1-01-07, 1-01-08, 1-01-09.
3. **No serial `await` in T5.** T5 must use `tokio::try_join!(setter1, setter2)` with BOTH futures constructed before the first `.await`. Sequential `setter1.await; setter2.await;` would pass even with a race condition. **Bound to:** 1-01-11.
4. **No `panic!` for rollback.** T3 must force the error via invalid SQL (e.g., `DROP TABLE transcript_settings` before the auto-repoint UPDATE) or a real sqlx constraint violation. `panic!` poisons the tokio runtime, not the sqlx transaction, and gives a false signal. **Bound to:** 1-01-09.
5. **No "an error happened" assertions.** T4 must match the exact error variant: `assert!(matches!(result, Err(PreferencesError::InvalidCombination { .. })))`. A generic `assert!(result.is_err())` would pass on any unrelated error. **Bound to:** 1-01-10.

---

## Nyquist Sufficiency (from RESEARCH § Map to D-22's 5 Tests)

| Failure Mode | Nyquist Rate | Covered By |
|--------------|--------------|------------|
| SQLite ↔ RwLock desync | ≥1 test per transaction outcome (commit success, commit failure) — 2 min | T1 (1-01-07 hydration), T2 (1-01-08 atomic commit), T3 (1-01-09 rollback) → **3 tests, Nyquist-sufficient** |
| Invariant bypass | ≥1 test per hybrid branch (auto-repoint, reject) — 2 min | T2 (1-01-08 auto-repoint), T4 (1-01-10 REAL reject) → **2 tests, Nyquist-sufficient (A1 Option B)** |
| Concurrent racing | ≥2 parallel writers exercising `RwLock::write().await` serialization | T5 (1-01-11 try_join! two setters) → **1 test with 2 writers, Nyquist-sufficient** |
| Hot-path read staleness | ≥1 write→read assertion with fresh-value check | T2's post-commit read + T1's post-hydration read → **2 tests, Nyquist-sufficient** |

**Conclusion from RESEARCH:** D-22's 5 tests are Nyquist-sufficient for Phase 1 scope. The full QA-01 regression suite (Phase 6) will expand sampling, not replace it.

---

## Open Question — RESOLVED

**A1 — D-09 reject branch scope** (RESEARCH § Assumptions Log, assumption 1):

D-09 says "reject before SQLite is touched"; D-01's `UserPreferencesPatch` originally had no `provider` field.

**Resolution (user decision, 2026-04-07):** **Option B — Full implementation now.**

Phase 1 extends `UserPreferencesPatch` with an optional `provider: Option<String>` field that writes to `transcript_settings.provider`. The reject branch is REAL, not a stub: attempting to patch `{provider: 'parakeet'}` while current `ui_locale == 'ar'` returns `Err(PreferencesError::InvalidCombination { .. })` before any SQL is issued. T4 (task 1-01-10) asserts this by variant match.

**Implications honored by `01-01-PLAN.md`:**
1. `UserPreferencesPatch` struct (D-01) gains `provider: Option<String>` — task 1-01-02.
2. `set_user_preferences` writes to two rows in the same transaction in the "patch carries provider" case — task 1-01-03 (`apply_patch_atomic` step G).
3. `apply_patch_atomic` performs the invariant check *before* `pool.begin().await` so reject returns without opening a transaction at all — task 1-01-03 (step C before step D).
4. Success criterion #3 is fully testable in Phase 1 (no deferral to Phase 4).
5. **Phase 4 (TRANS-02) scope shrinks:** documented in `01-01-PLAN.md` `<cross_phase_impact>` block.
6. **D-21 commit order** is honored by the 6-wave structure of `01-01-PLAN.md`.

**Researcher's recommendation was Option A; user overrode to Option B with full awareness of the Phase 4 impact.**

---

## Validation Sign-Off

- [x] All T1..T5 tests bound to plan task IDs (1-01-07..1-01-11)
- [x] Wave 0 dependency chain documented (1-01-01 → 1-01-02 → 1-01-06 → T-tests)
- [x] All Anti-Sampling Rules bound to specific tasks
- [x] Threat IDs T-1-02..T-1-04 bound to T-test rows; T-1-01 documented as accept-disposition (not testable)
- [x] Open Question A1 resolved — Option B (full implementation)
- [ ] All tasks have `<automated>` verify or Wave 0 dependencies *(verified by plan-checker)*
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify *(verified by plan-checker — every task in 01-01-PLAN.md has an `<automated>` block)*
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s (quick scope)
- [ ] `nyquist_compliant: true` set in frontmatter *(flipped by plan-checker once task binding is reviewed)*

**Approval:** pending plan-checker review of `01-01-PLAN.md`
