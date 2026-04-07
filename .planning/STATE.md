# Project State

## Project Reference

See: `.planning/PROJECT.md` (updated 2026-04-07)

**Core value:** Record a meeting, get an accurate transcript and a useful summary — in your own language, without any audio or content leaving the machine.

**Current focus:** Phase 1 — planned, ready to execute

## Current Position

**Phase:** 01 — preferences-foundation
**Plan:** `01-01-PLAN.md` (17 tasks, 6 waves, bound to T1..T5 Nyquist tests)
**Status:** Planned — plan-checker PASSED on iteration 2 after 1 revision (4 HIGH + 2 MEDIUM + 1 LOW fixes). Ready for `/gsd-execute-phase 1`.
**Last activity:** 2026-04-07 — Phase 1 plan approved (`f8b3bde`)

**Progress:** `[▓░░░░░░░░░░░░░░░░░░░] 0/6 phases complete · 1/6 planned`

## Performance Metrics

| Metric | Value |
|--------|-------|
| Phases planned | 6 |
| Phases with executable PLAN.md | 1 |
| Phases complete | 0 |
| Plans complete | 0 |
| Requirements mapped | 30 / 30 |
| Coverage | 100% |

## Accumulated Context

### Decisions

- **2026-04-07 — A1 Option B (Phase 1):** `UserPreferencesPatch` ships with `provider: Option<String>` in Phase 1, not Phase 4. The Parakeet reject branch is a REAL test (T4, variant match), not a stub. **Impact:** Phase 4 scope SHRINKS — TRANS-02's `transcript_settings` write-surface work lands in Phase 1. Phase 4 keeps only UI concerns (hidden dropdown, banner, onboarding fork).
- **2026-04-07 — D-07 revised (CONTEXT.md):** Original D-07 wording ("acquire RwLock write guard across transaction") contradicted D-10/D-11 (commit-then-cache) and RESEARCH Pitfall 2 (holding RwLock across `.await` is a tokio deadlock footgun). D-07 rewritten to: read short-lived → clone → merge → invariant pre-flight → sqlx tx → commit → THEN acquire write-guard post-commit → update cache → drop. Bounded post-commit window is the T-1-02 threat item (mitigated, not eliminated).
- **2026-04-07 — D-13 "6+" reconciliation (CONTEXT.md):** RESEARCH call-site audit verified 4 LIVE sites, not 6. The "6+" in REQUIREMENTS.md PREFS-03 text counts 4 live + 2+ dead refs in `recording_commands.rs.backup` (non-compiled). The dead refs are eliminated by the dedicated `.backup` chore commit in Phase 1 wave 6 (D-15), not by source-level substitution.

### Open Todos

- **Spec file tracking:** `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` has been untracked in git since session start. It is the authoritative design reference cited by every phase's CONTEXT.md canonical_refs. Decide whether to commit it (as `docs(specs): authoritative arabic bilingual design v2`) or whether it intentionally lives outside version control.
- **Pre-existing `<domain>` inconsistency (Phase 1 CONTEXT.md line 14):** The `<domain>` block still reads "All 6+ recording-path call sites" while D-13 now correctly says "4 LIVE". Plan-checker flagged this as pre-existing (not a regression), cosmetic, non-blocking. Clean up whenever convenient.

### Blockers

(none)

### Key Discoveries

- Phase 1 has ~4 live recording-path call sites, not 6 — audit confirmed by researcher.
- `once_cell`, `tokio`, `thiserror`, `sqlx` already present in `frontend/src-tauri/Cargo.toml` — **zero new dependencies** needed for Phase 1.
- Migration pattern `let mut tx = pool.begin().await?` + `&mut *tx` reborrow is already proven in `database/repositories/meeting.rs:26-80` and `audio/import.rs:720-735` — mirror exactly.
- Hydration integration point: `lib.rs:482-485` — wrap `preferences::hydrate_from_db(state.db_manager.pool()).await` in `tauri::async_runtime::block_on(...)` AFTER `initialize_database_on_startup` but BEFORE the Whisper/Parakeet spawns (defeats Pitfall 4 cache race).
- ConfigContext.tsx has exactly 3 `localStorage`/`primaryLanguage` touchpoints (lines 142, 215, 477) — the full migration + useEffect deletion fits in a single commit per D-18.
- Phase 1 has no new UI surface — ConfigContext work is pure deletion + service rewiring, so no UI-SPEC needed.

## Session Continuity

**Last session:** 2026-04-07 — Phase 1 planned end-to-end (research → validation → A1 resolution → plan → revision 1 → plan-check PASSED)

**Next action:** `/clear` then `/gsd-execute-phase 1` — runs the 17 tasks in 6 strictly-sequential waves following D-21's commit order (migration → module + patch + hydration → targeted tests → call-site migration → frontend migration → .backup cleanup).

**Watch-outs to remember:**
- Phase 1 is the **highest-risk single phase** (ROADMAP risk note #2) — T1..T5 tests MUST ship alongside the implementation, in the same commit stream
- **A1 Option B in effect** — Phase 1 now owns `UserPreferencesPatch.provider` and the `transcript_settings` write surface; Phase 4's TRANS-02/TRANS-04 scope is correspondingly reduced (UI concerns only)
- **D-07 post-commit cache ordering is load-bearing** — any executor change to `commands.rs` that inverts the `apply_patch_atomic` → `PREFS_CACHE.write()` order breaks T3 rollback invariance
- **`.backup` removal is a SEPARATE commit** (D-15) — never bundle it with functional changes
- **`ConfigContext.tsx:215` useEffect deletion + localStorage removal land in ONE commit** (PROJECT.md hard constraint)
- The BlockNote spike (Phase 3, first plan) gates SUMM-04 scope — Phase 5 plans must branch on its outcome
- ESLint guardrail (QA-07) lands in Phase 3 *before* hotspot conversion, not at the end

---
*State initialized: 2026-04-07*
