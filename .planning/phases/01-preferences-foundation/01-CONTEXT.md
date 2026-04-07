# Phase 1: Preferences Foundation - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

User preferences live in exactly one place (SQLite `user_preferences` row, hydrated
into a process-global Rust cache) with atomic writes that make invalid states
(e.g. `ui_locale == 'ar'` + `provider == 'parakeet'`) unrepresentable. All 6+
recording-path call sites migrate to the new module in the same commit stream;
the `ConfigContext.tsx:215` startup-desync `useEffect` workaround is deleted in
the same commit as the migration that eliminates its cause.

**Requirements covered:** PREFS-01, PREFS-02, PREFS-03, PREFS-04

**Fixed out-of-scope for this phase** (carried by downstream phases):
- `navigator.language` first-run detection → Phase 2 (UI-01)
- `<html lang dir>` attribute switching and `next-intl` wiring → Phase 2 (UI-02..UI-04)
- Parakeet-ban *UI* surfacing (hidden dropdown, banner, onboarding fork) → Phase 4
  (TRANS-02, TRANS-03). Phase 1 only builds the invariant hook in `set_user_preferences`;
  TRANS-04 atomic provider repoint is wired by Phase 4.
- Template/prompt locale resolution → Phase 5
- QA-01 desync regression suite → Phase 6. Phase 1 ships its own targeted tests
  alongside the migration (per ROADMAP risk note #2); the full suite is Phase 6.

</domain>

<decisions>
## Implementation Decisions

### SQLite schema & migration
- **D-01:** New table `user_preferences` with columns `id TEXT PRIMARY KEY DEFAULT '1'`,
  `ui_locale TEXT NOT NULL DEFAULT 'en'`, `summary_language TEXT NOT NULL DEFAULT 'en'`,
  `transcription_language TEXT NOT NULL DEFAULT 'auto'`, `updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`.
  Seeded with `INSERT OR IGNORE INTO user_preferences (id) VALUES ('1')`. (Spec §3.2)
- **D-02:** Migration lands as a new `.sql` file under
  `frontend/src-tauri/migrations/` following the existing timestamp naming
  convention (pattern of 11 existing migrations: `YYYYMMDDHHMMSS_description.sql`).
  Applied at startup by the existing sqlx migrator.

### Rust module layout
- **D-03:** New module `frontend/src-tauri/src/preferences/` containing:
  - `mod.rs` — `UserPreferences` struct, `UserPreferencesPatch` (all fields `Option<T>`),
    process-global `Lazy<RwLock<UserPreferences>>` cache, `pub fn read() -> UserPreferences`
    convenience reader.
  - `commands.rs` — two Tauri commands: `get_user_preferences` / `set_user_preferences`.
  - `repository.rs` — SQL adapters (mirrors `database/repositories/setting.rs` style).
- **D-04:** Concurrency primitive for the cache: `tokio::sync::RwLock` to match
  `NotificationManagerState` and the existing `use tokio::sync::RwLock` pattern in `lib.rs`.
  Claude's discretion if profiling shows contention later.
- **D-05:** `preferences::hydrate_from_db(pool).await` runs inside `lib.rs::run` as
  part of the `setup` closure, **before** command registration returns control, so
  the first `get_user_preferences` invocation always resolves against a populated cache.

### Tauri command shape (setter = partial patch)
- **D-06:** `set_user_preferences` accepts a **partial patch** at both boundaries:
  - Frontend: `setUserPreferences(p: Partial<UserPreferences>): Promise<UserPreferences>`
  - Rust: `async fn set_user_preferences(patch: UserPreferencesPatch) -> Result<UserPreferences, String>`
    where every field is `Option<T>`.
- **D-07:** Merge order inside `set_user_preferences` (REVISED 2026-04-07 for
  consistency with D-10/D-11 and RESEARCH Pitfall 2; the original draft held the
  `RwLock` write guard across `.await`, which is a known tokio deadlock footgun
  and contradicted the commit-then-cache rule in D-10/D-11): read current state
  via a **short-lived** `PREFS_CACHE.read()` guard → clone + drop the guard →
  merge `patch` over the cloned current (non-`None` fields win) → run invariant
  (D-10..D-12) pre-flight (pure function, no locks, no I/O) → open sqlx
  transaction, write `user_preferences` UPDATE (and conditionally
  `transcript_settings` UPDATE per D-08) → `COMMIT` → **only after commit
  succeeds**, acquire `PREFS_CACHE.write()` guard, assign `*guard = merged`,
  drop the guard → return the merged result.
  - No guard is ever held across an `.await` boundary (per RESEARCH Pitfall 2).
  - On rollback or reject, the RwLock is never written — cache stays consistent
    with disk (D-11, D-12).
  - The "window" between `COMMIT` and `PREFS_CACHE.write()` is bounded by the
    duration of a single uncontended write-lock acquisition (microseconds) and
    is the T-1-02 threat item — mitigated (not eliminated) by tests T2/T3/T5.
    The original D-07 draft claimed "no window" but held a lock across `.await`
    to achieve that guarantee, which is unsafe. The revised D-07 prefers safety
    and documents the bounded window.

### Parakeet-ban invariant (hybrid: auto-repoint + reject)
- **D-08:** **Auto-repoint on locale change.** When the merge result flips
  `ui_locale` to `'ar'` and the current `transcript_settings.provider` is
  `'parakeet'`, `set_user_preferences` atomically forces the provider to
  `localWhisper` + `large-v3` inside the same SQL transaction.
  This matches TRANS-04 ("atomic provider repoint on locale switch") and spec §5.2.
- **D-09:** **Reject direct violating writes.** When the merge result has
  `ui_locale == 'ar'` AND the caller's patch is directly setting a Parakeet-shaped
  configuration (e.g. via a future coupled call site), the invariant returns
  `Err(PreferencesError::InvalidCombination { ... })` **before** touching SQLite.
  This matches QA-02 / TRANS-02 / spec §10.2 rejection tests.
- **D-10:** **Direction-of-change rule as the tie-breaker.** The auto-repoint
  branch fires when the *user intent* is "change the locale"; the reject branch
  fires when the user intent is "set a provider that contradicts the current
  locale". The `UserPreferencesPatch` is the source of truth for intent — the
  invariant reads the patch fields, not the merged state, to decide which branch.

### Cross-table atomic transaction
- **D-11:** `set_user_preferences` opens a single `sqlx` transaction that spans
  both `user_preferences` UPDATE and (conditionally) the `transcript_settings`
  UPDATE required by the auto-repoint branch. `BEGIN` → preferences UPDATE →
  (if Arabic auto-repoint) transcript_settings UPDATE → `COMMIT`. RwLock update
  happens *after* successful commit; on rollback the RwLock is never touched so
  the in-memory cache stays consistent with disk.
- **D-12:** If the transaction fails (any reason, including invariant rejection
  being detected mid-transaction rather than pre-flight), the command returns an
  error and the RwLock is not mutated. Failed writes are invisible to readers.

### Call-site migration
- **D-13:** `get_language_preference_internal()` is **deleted**, not deprecated.
  All **4 LIVE** recording-path call sites migrate in a single commit to
  `preferences::read().transcription_language`:
  - `frontend/src-tauri/src/whisper_engine/commands.rs:396`
  - `frontend/src-tauri/src/whisper_engine/parallel_processor.rs:344`
  - `frontend/src-tauri/src/audio/transcription/worker.rs:449`
  - `frontend/src-tauri/src/audio/transcription/worker.rs:526`

  *Reconciliation with REQUIREMENTS.md PREFS-03 "6+" wording:* The RESEARCH
  call-site audit (2026-04-07) verified these are the only **4 compiled sites**.
  The "6+" in REQUIREMENTS.md counts the 4 live sites above PLUS 2+ dead
  references inside `audio/recording_commands.rs.backup` (non-compiled file,
  wrong extension — Rust ignores it). Those dead references are eliminated by
  **D-15's dedicated chore commit** (Phase 1 wave 6), not by source-level
  substitution. Phase-level PREFS-03 acceptance is reached AFTER both the
  4-site migration commit AND the `.backup` deletion commit have landed.
- **D-14:** `set_language_preference` Tauri command at `lib.rs:376` is deleted
  along with the `LANGUAGE_PREFERENCE` global static. Frontend callers (if any
  remain) are updated to invoke `set_user_preferences({ transcriptionLanguage })`.
- **D-15:** `frontend/src-tauri/src/audio/recording_commands.rs.backup` is
  **deleted in a dedicated chore commit** within this phase. Rationale: the file
  is 82KB (vs 46KB for the live `recording_commands.rs`), has 3 stale references
  to `get_language_preference_internal()` at lines 1276/1353/1440, and is not
  compiled (wrong extension for Rust to pick up). Git history retains it if a
  future refactor wants to resurrect it. Commit message: `chore: remove unused
  recording_commands.rs.backup`.

### Frontend migration (ConfigContext + service)
- **D-16:** New `frontend/src/services/preferencesService.ts` exports
  `getUserPreferences()` and `setUserPreferences(p: Partial<UserPreferences>)`
  wrapping the two Tauri commands, plus the TypeScript types
  `UiLocale = 'en' | 'ar'`, `SummaryLanguage = 'en' | 'ar'`,
  `UserPreferences { uiLocale, summaryLanguage, transcriptionLanguage }`.
- **D-17:** `ConfigContext.tsx:140` — direct `localStorage.getItem('primaryLanguage')`
  access is removed. Replaced with a single `getUserPreferences()` call at mount
  time, hydrating React state from the returned payload.
- **D-18:** `ConfigContext.tsx:215` — the `useEffect` "fixes startup desync bug"
  workaround is deleted in the **same commit** as the localStorage removal.
  Per PROJECT.md: "Don't leave dead workarounds; the comment lies once the cause
  is gone."
- **D-19:** `primaryLanguage` entries in `localStorage` are left alone (no
  migration write). On next mount the new service reads from SQLite; the stale
  localStorage key is simply never read again. No need for a one-shot cleanup
  since it's harmless residue.

### Rollout posture
- **D-20:** **Clean cut-over, no feature flag.** The spec §11 P0 wording
  ("ship behind a feature flag; no user-visible UI yet") is treated as stale
  against the ROADMAP Phase 1 success criteria, which explicitly require the new
  module to be wired end-to-end (recording honors new preference on next
  recording; concurrent-setter test passes; desync workaround deleted). A flag
  cannot coexist with those criteria or with PROJECT.md's desync-workaround
  deletion constraint.
- **D-21:** Phase 1 commit order (executor-level, subject to plan refinement):
  1. Migration SQL + schema lands (no code readers yet)
  2. `preferences/` module + Tauri commands + hydration in `lib.rs::run`
  3. Targeted Phase-1 tests (atomic write, invariant hybrid, concurrent setter)
  4. 6 recording-path call sites migrated in one commit; old global + old command deleted
  5. Frontend service + ConfigContext migration (localStorage removal + workaround deletion in one commit)
  6. `chore: remove unused recording_commands.rs.backup` (dedicated commit per D-15)

### Targeted Phase 1 tests (not Phase 6 QA)
- **D-22:** Phase 1 ships these tests alongside the code (Phase 6 QA-01 is the
  full regression suite; these are the minimum to de-risk the migration):
  - Startup hydration: SQLite has `ui_locale='ar'` → `preferences::read()` returns `'ar'` immediately after `run` setup completes.
  - Atomic-write invariant: `set_user_preferences({ ui_locale: 'ar' })` while `transcript_settings.provider == 'parakeet'` → both rows updated in one commit, RwLock updated after commit.
  - Rollback invariance: force the `transcript_settings` UPDATE to fail → `user_preferences` row is unchanged AND the RwLock is unchanged.
  - Reject-branch invariant: direct patch `{ provider: 'parakeet' }` while current `ui_locale == 'ar'` → `Err(InvalidCombination)` before SQLite is touched.
  - Concurrent setter: two parallel `set_user_preferences` calls → no partial state, final result equals one of the two inputs.

### Claude's Discretion
- RwLock flavor if `tokio::sync::RwLock` shows contention in profiling (may swap to `parking_lot::RwLock` wrapped in a std-sync shim; Claude can decide during execution).
- Whether `updated_at` is bumped via a SQL trigger or explicitly in `repository.rs` (explicit bump in the repository is the stylistic match with the rest of `database/repositories/setting.rs`).
- Internal ConfigContext API shape: single `setPreferences(partial)` vs a small set of granular wrappers (`setUiLocale`, `setSummaryLanguage`, `setTranscriptionLanguage`) that all delegate to the same service call. Claude picks whichever reads better in context.
- Exact error variant names inside `PreferencesError` enum.
- Whether to use `sqlx::query!` macro or string literals — mirror whatever `database/repositories/setting.rs` already does to minimize style churn.
- Migration filename timestamp (just needs to be strictly greater than the highest existing one, `20251229000000_add_gemini_api_key.sql`).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary spec (authoritative for every decision)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §3 — Preferences Model (3.1 current state, 3.2 v2 design, 3.3 root layout integration)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §5.2 — Parakeet ban invariant code example (D-08 source)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §10.1 — Preference desync regression tests (D-22 source; full suite is Phase 6 QA-01)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §10.2 — Parakeet ban enforcement tests (D-09 source)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §11 — Implementation Phases (P0 wording is stale; see D-20)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §12 — Open Questions (§12.1 navigator.language is Phase 2's job, not Phase 1)

### GSD planning artifacts (authoritative for scope & success criteria)
- `.planning/PROJECT.md` — "Key Decisions" row "Single SQLite `user_preferences` row..." and Constraints row "Desync workaround deletion: `ConfigContext.tsx:215` `useEffect` is removed in the same commit as the preferences migration"
- `.planning/REQUIREMENTS.md` — PREFS-01 through PREFS-04 full text (traceability table maps them to Phase 1)
- `.planning/ROADMAP.md` — Phase 1 section: 4 success criteria (lines 28-32) and risk note #2 ("Phase 1 is the highest-risk single phase... Phase 1 should land its own targeted tests")
- `.planning/ROADMAP.md` — Risk note #6: "`audio/recording_commands.rs.backup` is referenced in PREFS-03 — confirm during Phase 1 plan-check whether the `.backup` file is live code or vestigial" (answered in D-15)

### Codebase intel (read before planning the migration)
- `.planning/codebase/STACK.md` — Rust async runtime, sqlx usage, locks in the codebase
- `.planning/codebase/CONVENTIONS.md` — Rust naming (snake_case files, PascalCase types), import organization, `UserPreferences` fits standard naming
- `.planning/codebase/ARCHITECTURE.md` — where `preferences/` sits in the module graph
- `.planning/codebase/CONCERNS.md` — audio-pipeline hot-zone context (relevant to D-13 migration risk)

### Code touchpoints (file:line precision)
- `frontend/src-tauri/src/lib.rs:361` — current `LANGUAGE_PREFERENCE` static (`Lazy<StdMutex<String>>`), to be deleted
- `frontend/src-tauri/src/lib.rs:376` — `set_language_preference` command, to be deleted
- `frontend/src-tauri/src/lib.rs:386` — `get_language_preference_internal` helper, to be deleted
- `frontend/src-tauri/src/lib.rs:662` — current `set_language_preference` registration in `invoke_handler`, to be replaced with `get_user_preferences` / `set_user_preferences` registrations
- `frontend/src-tauri/src/database/repositories/setting.rs` — style reference for new `preferences/repository.rs`
- `frontend/src-tauri/src/database/mod.rs` — submodule wiring (preferences/ is *not* under database/repositories/, it's a top-level module per spec §3.2)
- `frontend/src-tauri/migrations/` — 11 existing migrations follow `YYYYMMDDHHMMSS_description.sql`; new file must continue the sequence
- `frontend/src-tauri/src/whisper_engine/commands.rs:396` — call site 1
- `frontend/src-tauri/src/whisper_engine/parallel_processor.rs:344` — call site 2
- `frontend/src-tauri/src/audio/transcription/worker.rs:449` — call site 3
- `frontend/src-tauri/src/audio/transcription/worker.rs:526` — call site 4
- `frontend/src-tauri/src/audio/recording_commands.rs.backup:1276,1353,1440` — dead references, file deleted per D-15
- `frontend/src/components/ConfigContext.tsx:140` — `localStorage.getItem('primaryLanguage')` to remove (D-17)
- `frontend/src/components/ConfigContext.tsx:215` — "fixes startup desync bug" `useEffect` to delete (D-18)

### Cross-phase contract surface (Phase 1 produces, later phases consume)
- **Phase 2 will read** `preferences::read().ui_locale` at `lib.rs::run` startup (spec §3.3) — Phase 1 must have hydration complete before any UI component mounts
- **Phase 4 will call** `set_user_preferences({ uiLocale: 'ar' })` and expect the auto-repoint to fire atomically (TRANS-04) — Phase 1 must land D-08 + D-11
- **Phase 5 will read** `preferences::read().summary_language` as the resolution key for templates and prompts — Phase 1 must expose `summary_language` on the struct
- **Phase 6 will write** the full regression suite on top of Phase 1's targeted tests (D-22) — Phase 1 tests are the minimum viable; Phase 6 expands coverage

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **`database/repositories/setting.rs`** — exact style reference for
  `preferences/repository.rs`. Same sqlx patterns (`query_as`, `query` with
  bind, upsert with `ON CONFLICT(id) DO UPDATE`, `id = '1'` singleton row
  convention). Mirror this file's shape.
- **Existing sqlx migrator** — new migration file just drops into
  `frontend/src-tauri/migrations/` with the right timestamp prefix. No wiring
  changes needed; existing startup path picks it up.
- **`Lazy<...>` + locks pattern** already used across the codebase
  (`LANGUAGE_PREFERENCE` at `lib.rs:361`, `WHISPER_ENGINE` inside
  `whisper_engine/commands.rs`, `NotificationManagerState` at `lib.rs` setup).
  The new `Lazy<RwLock<UserPreferences>>` fits cleanly.
- **`tokio::sync::RwLock`** is already imported in `lib.rs` and used for
  `NotificationManagerState` — D-04 reuses this choice.

### Established Patterns
- **Rust command naming**: `snake_case` on `#[tauri::command]`s, `PascalCase`
  on request/response structs with `#[derive(serde::Deserialize)]` +
  `#[serde(rename = "camelCase")]` for fields that need to bridge to TS.
  The frontend contract in spec §3.2 uses camelCase (`uiLocale`,
  `summaryLanguage`, `transcriptionLanguage`) — that means the Rust struct
  fields need `#[serde(rename = ...)]` annotations.
- **Singleton-row SQLite pattern**: `settings` and `transcript_settings`
  tables both use `id = '1'` as the singleton row convention. `user_preferences`
  follows the same pattern (D-01).
- **Deletion-not-deprecation for internal helpers**: the codebase already has
  precedent (see `lib_old_complex.rs` in CONCERNS — anti-pattern left behind).
  Phase 1's `get_language_preference_internal()` deletion matches the
  "don't leave dead workarounds" constraint.
- **Commit-per-concern discipline**: existing migration commits (see
  `20251229000000_add_gemini_api_key.sql` etc.) land one migration + its
  immediate plumbing per commit. D-21 commit sequence matches.

### Integration Points
- **Where new code connects**:
  - `lib.rs::run` setup closure → calls `preferences::hydrate_from_db` before
    command registration
  - `lib.rs` `invoke_handler![...]` → adds `get_user_preferences`,
    `set_user_preferences`; removes `set_language_preference`
  - 4 `crate::get_language_preference_internal()` call sites → replaced with
    `crate::preferences::read().transcription_language`
  - `ConfigContext.tsx` mount effect → calls new `preferencesService`
- **What new code does NOT touch**:
  - `database/` submodule internals (preferences/ is its own top-level module
    per spec §3.2, though it uses `SqlitePool` from `database::` as its
    persistence substrate)
  - Audio pipeline hot path beyond the 4 read-site changes (the migration is
    a strict substitution; no behavior change for English users)
  - Whisper model loading, cpal capture, Tauri event emission, any frontend
    UI component other than `ConfigContext.tsx`

### Audio Pipeline Risk (for planner awareness)
- `.planning/codebase/CONCERNS.md` notes the audio pipeline is a hot zone
  (concern #6 "Audio pipeline panics on VAD init failure")
- The 4 call-site migrations at
  `whisper_engine/commands.rs:396`, `parallel_processor.rs:344`,
  `audio/transcription/worker.rs:449,526` are in the transcription path
- Migration is a **read-only substitution** — replacing one getter call with
  another of the same shape (`Option<String>` returning `transcription_language`).
  No new async boundaries, no new allocations, no new error paths in the
  recording loop itself. The risk is entirely in the shape of
  `preferences::read()` — if that function is not O(1) lock acquisition +
  clone, it will degrade the recording hot path. D-04's `tokio::sync::RwLock`
  is fine because read contention on a rarely-written value is cheap.

</code_context>

<specifics>
## Specific Ideas

- The rollout posture is a **clean cut-over, not staged**. This is the
  user's explicit choice against the spec's stale §11 P0 "feature flag" wording.
  When the planner sees "feature flag" anywhere in the spec or research docs,
  ignore it for this phase.
- The `.backup` file deletion is a **dedicated commit** with a specific
  commit message (`chore: remove unused recording_commands.rs.backup`). It is
  not bundled into the migration commit. This is for git-archaeology purposes
  — a future maintainer searching for why that file disappeared should find
  the commit quickly.
- The Parakeet-ban **direction-of-change rule** (D-10) is the conceptual
  heart of the invariant. It's worth a code comment above the invariant
  function explaining that the patch fields determine user intent, not the
  merged state.
- Phase 1's targeted tests (D-22) are deliberately narrow. They are NOT a
  substitute for Phase 6 QA-01. They exist so that the migration commit can
  be merged with confidence. Phase 6 builds the full desync regression suite
  on top of this foundation.

</specifics>

<deferred>
## Deferred Ideas

- **Encrypting the `user_preferences` row** — raised implicitly by
  CONCERNS.md entry #4 (plaintext API keys in SQLite). `user_preferences`
  holds no secrets (just `ui_locale`, `summary_language`,
  `transcription_language`), so there's no urgency, but if a future phase
  takes on the CONCERNS #4 encryption work, `user_preferences` should be
  included in the same envelope for consistency. Not in scope for Phase 1.
- **Telemetry / analytics on preference changes** — not mentioned in spec
  or requirements, but `analytics::commands::track_settings_changed` already
  exists in `lib.rs:514`. A future phase may want to emit an analytics event
  on locale switch. Deferred indefinitely until product raises it.
- **Multi-user support** — `id = '1'` singleton row convention is fine for
  a single-user desktop app. If Meetily ever grows multi-user, the schema
  needs a `user_id` column and RLS semantics. Explicitly out of scope.
- **Hot-swap mid-session locale switching** — rejected in PROJECT.md Key
  Decisions (§12.4, "triggers full reload instead"). Phase 1 design
  deliberately does not support runtime propagation of locale changes to
  running Rust subsystems (tray, notifications) except via a full restart.
  Phase 6 UI-07 hydrates Rust strings at startup and re-hydrates on
  preference change via Tauri event, but that is Phase 6's problem, not
  Phase 1's.
- **Internationalized error messages** for `PreferencesError` variants —
  log and error messages stay in English per PROJECT.md Out of Scope.

</deferred>

---

*Phase: 01-preferences-foundation*
*Context gathered: 2026-04-07*
*Discussion mode: discuss (standard)*
*Gray areas discussed: GA-1 (.backup file), GA-2 (Parakeet ban semantic), GA-3 (setter shape), GA-4 (rollout posture)*
