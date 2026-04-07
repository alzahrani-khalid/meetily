---
phase: 1
slug: preferences-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-07
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
| **Shared helper** | `async fn test_pool_with_migration() -> SqlitePool` in `preferences/tests.rs` |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend/src-tauri && cargo test preferences::` (quick scope).
- **After every plan wave:** Run `cd frontend/src-tauri && cargo test` (full suite) to catch regressions in migrated call sites.
- **Before `/gsd-verify-work`:** Full suite must be green AND `preferences::` scope must be green AND all 5 D-22 tests (T1..T5) must be present and passing.
- **Max feedback latency:** 10 seconds for the quick scope; ~60 seconds for the full suite (acceptable given phase risk).

---

## Per-Task Verification Map

> **Task IDs** (`1-NN-NN` form) are placeholders until `gsd-planner` emits PLAN.md
> files and binds each of the D-22 tests (T1..T5) to a specific task. The
> `plan-checker` will update this table and flip `nyquist_compliant: true` once
> binding is verified.

| Test ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| T1 (hydration) | 01 | 1 | PREFS-01 | — | Startup reads `ui_locale='ar'` from seeded SQLite row → `preferences::read().await.ui_locale == "ar"` immediately after `hydrate_from_db` returns | integration | `cargo test preferences::tests::hydration_reflects_seeded_row` | ❌ W0 | ⬜ pending |
| T2 (atomic auto-repoint) | 01 | 2 | PREFS-02 | T-1-01 | `set_user_preferences({ui_locale:'ar'})` while `transcript_settings.provider='parakeet'` → BOTH rows updated in one commit, `RwLock` updated AFTER commit, `read().await.ui_locale=="ar"` | integration | `cargo test preferences::tests::atomic_write_auto_repoints_parakeet` | ❌ W0 | ⬜ pending |
| T3 (rollback invariance) | 01 | 2 | PREFS-02 | T-1-02 | Force the `transcript_settings` UPDATE to fail via invalid column → `user_preferences` row unchanged AND `RwLock` unchanged AND error flows through `Result`, not `panic` | integration | `cargo test preferences::tests::rollback_leaves_cache_and_row_unchanged` | ❌ W0 | ⬜ pending |
| T4 (reject branch) | 01 | 2 | PREFS-02 | T-1-03 | Direct patch `{provider:'parakeet'}` while `ui_locale=='ar'` → `Err(PreferencesError::InvalidCombination { .. })` matched by variant (NOT just "an error"), BEFORE `BEGIN` is issued. **See Open Question A1 below — scope may be scaffold-only for Phase 1.** | integration | `cargo test preferences::tests::reject_parakeet_while_arabic` | ❌ W0 | ⬜ pending |
| T5 (concurrent setter) | 01 | 2 | PREFS-02 | T-1-04 | `tokio::try_join!(set1, set2)` with both futures started before any `.await` → no partial state, final result equals one of the two inputs, no deadlock within 2s | integration | `cargo test preferences::tests::concurrent_setters_serialize` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

**Coverage matrix — requirements to tests:**

| Requirement | Success Criterion | Covered By |
|-------------|-------------------|------------|
| PREFS-01 | Criterion 1 (UI locale survives restart, no `useEffect` rehydrate) | T1 + manual M1 (restart persistence) |
| PREFS-02 | Criterion 2 (transcription_language honored on next recording) | T2 + T3 (atomic) + T5 (concurrent) |
| PREFS-02 | Criterion 3 (parakeet + arabic rejected before SQLite touched) | T2 (auto-repoint branch) + T4 (reject branch, subject to A1) |
| PREFS-02 | Criterion 4 (no observable window where SQLite and RwLock disagree) | T2 (post-commit ordering) + T5 (concurrent window) |
| PREFS-03 | (implicit — call sites read new module) | Compilation + full `cargo test` (no dedicated test; migrated sites exercised by existing tests) |
| PREFS-04 | (implicit — `ConfigContext.tsx:215` useEffect deleted, `localStorage` gone) | Manual M2 (frontend audit) + grep-verified acceptance in plan tasks |

---

## Wave 0 Requirements

- [ ] `frontend/src-tauri/src/preferences/tests.rs` — new test module containing T1..T5 and the `test_pool_with_migration()` helper
- [ ] `frontend/src-tauri/src/preferences/mod.rs` — module root (created by Wave 1; tests.rs is a child)
- [ ] Shared helper `async fn test_pool_with_migration() -> SqlitePool` — creates `sqlite::memory:`, runs the migration SQL (loaded via `include_str!` from the migration file), seeds the single `user_preferences` row
- [ ] **No framework install required** — `tokio::test` is available via existing `tokio = "1.32"` `full` features
- [ ] **No new `Cargo.toml` entries required**

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| M1: Restart persistence | PREFS-01 (criterion 1) | Tauri app lifecycle + SQLite file write — cannot exercise `run()` from `cargo test` | 1) `pnpm run tauri:dev`  2) Switch UI locale to Arabic via settings  3) Close app fully (not reload)  4) Relaunch  5) Assert: UI renders in Arabic BEFORE any network/IPC call, no flash of English |
| M2: ConfigContext workaround deleted | PREFS-04 | Static audit — no test can prove a `useEffect` *doesn't exist* | `grep -n "useEffect" frontend/src/contexts/ConfigContext.tsx` returns 0 results for the `:215` block; `grep -n "localStorage" frontend/src/contexts/ConfigContext.tsx` returns 0 results for `primaryLanguage`; `grep -rn "set_language_preference" frontend/src/` returns 0 results |
| M3: Next-recording honoring | PREFS-02 (criterion 2) | Requires a real recording flow end-to-end | 1) Start recording  2) Stop  3) Switch `transcription_language` via settings (no app restart)  4) Start a second recording  5) Assert: Whisper receives the new language code (verify via Rust logs with `RUST_LOG=app_lib::audio=debug`) |

---

## Anti-Sampling Rules (from RESEARCH § Anti-Sampling)

These rule out tests that would falsely pass. Plan-checker must verify each plan task that binds to T1..T5 honors these:

1. **No mocked SQLite rows.** Every test uses `sqlx::SqlitePool::connect("sqlite::memory:")` and runs the real migration SQL. A `sqlx::query!` with a mocked row would pass even if the transaction logic is broken.
2. **No direct `PREFS_CACHE` inspection.** Tests must call `set_user_preferences` via the real code path (or the extracted `apply_patch_atomic` helper that does commit-then-cache in that exact order). Reading `PREFS_CACHE` directly would miss commit-then-cache ordering inversions.
3. **No serial `await` in T5.** T5 must use `tokio::try_join!(setter1, setter2)` with BOTH futures constructed before the first `.await`. Sequential `setter1.await; setter2.await;` would pass even with a race condition.
4. **No `panic!` for rollback.** T3 must force the error via invalid SQL (e.g., a deliberately non-existent column in a test-only query) or a real sqlx constraint violation. `panic!` poisons the tokio runtime, not the sqlx transaction, and gives a false signal.
5. **No "an error happened" assertions.** T4 must match the exact error variant: `assert!(matches!(result, Err(PreferencesError::InvalidCombination { .. })))`. A generic `assert!(result.is_err())` would pass on any unrelated error (network, serialization, even a typo).

---

## Nyquist Sufficiency (from RESEARCH § Map to D-22's 5 Tests)

| Failure Mode | Nyquist Rate | Covered By |
|--------------|--------------|------------|
| SQLite ↔ RwLock desync | ≥1 test per transaction outcome (commit success, commit failure) — 2 min | T1 (hydration), T2 (atomic commit), T3 (rollback) → **3 tests, Nyquist-sufficient** |
| Invariant bypass | ≥1 test per hybrid branch (auto-repoint, reject) — 2 min | T2 (auto-repoint), T4 (reject hook) → **2 tests, Nyquist-sufficient pending A1** |
| Concurrent racing | ≥2 parallel writers exercising `RwLock::write().await` serialization | T5 (try_join! two setters) → **1 test with 2 writers, Nyquist-sufficient** |
| Hot-path read staleness | ≥1 write→read assertion with fresh-value check | T2's post-commit read + T1's post-hydration read → **2 tests, Nyquist-sufficient** |

**Conclusion from RESEARCH:** D-22's 5 tests are Nyquist-sufficient for Phase 1 scope. The full QA-01 regression suite (Phase 6) will expand sampling, not replace it.

---

## Open Question (raised by RESEARCH, must be resolved before planning)

**A1 — D-09 reject branch scope** (RESEARCH § Assumptions Log, assumption 1):

D-09 says "reject before SQLite is touched"; D-01's `UserPreferencesPatch` has no `provider` field. Two resolutions:

- **Option A (scaffold):** Phase 1 wires the invariant hook as a documented no-op with a `#[should_panic]`-style stub test; Phase 4 (TRANS-02) extends the patch with `provider` and replaces T4 with the real reject assertion. Keeps Phase 1 strictly additive to `user_preferences`.
- **Option B (full implementation):** Phase 1 extends `UserPreferencesPatch` with optional `provider` tied to `transcript_settings`, and T4 asserts the real reject path. Expands Phase 1 scope into `transcript_settings` write surface.

**Researcher's strong recommendation: Option A.** Plan-checker will block on this until resolved.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies *(populated by planner)*
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify *(populated by planner)*
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s (quick scope)
- [ ] `nyquist_compliant: true` set in frontmatter *(flipped by plan-checker once task binding is verified)*
- [ ] Open Question A1 resolved *(pending user decision, see above)*

**Approval:** pending
