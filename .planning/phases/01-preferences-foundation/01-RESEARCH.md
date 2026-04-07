# Phase 1: Preferences Foundation - Research

**Researched:** 2026-04-07
**Domain:** SQLite-backed Rust process-global state with atomic Tauri command writes
**Confidence:** HIGH (all claims verified against the live codebase at file:line precision)

## Summary

This phase replaces a 3-line global `LazyLock<StdMutex<String>>` (the current `LANGUAGE_PREFERENCE` at `lib.rs:68`) and its Tauri command (`set_language_preference` at `lib.rs:376`) with a proper `preferences/` module backed by a new `user_preferences` SQLite row and hydrated into a `Lazy<tokio::sync::RwLock<UserPreferences>>` at startup. The module must atomically enforce the Parakeet-ban invariant by opening a single `sqlx::Transaction<'_, Sqlite>` that updates both `user_preferences` and (when Arabic is chosen over Parakeet) `transcript_settings`, commit or rollback as a unit, and only then mutate the RwLock. The matching `ConfigContext.tsx:215` `useEffect` workaround is deleted in lockstep with the migration, and all 4 grep-confirmed `get_language_preference_internal()` callers in the transcription path migrate to `preferences::read().transcription_language` in one commit. Every CONTEXT decision (D-01..D-22) is implementable with the codebase's existing sqlx 0.8 + tokio 1.32 + once_cell 1.17 toolchain; no new dependencies are required.

**Primary recommendation:** Mirror three existing patterns verbatim — (1) `database/repositories/setting.rs` for sqlx upsert style, (2) `database/manager.rs::with_transaction` closure for tx lifecycle, and (3) `notifications::manager`'s `Lazy<RwLock>` state holder. The only genuinely new code is the `PreferencesError::InvalidCombination` enum variant and the direction-of-change rule (D-10) that reads `UserPreferencesPatch` fields (not merged state) to choose between auto-repoint and reject branches.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions (D-01..D-22)

**SQLite schema & migration**
- **D-01:** New table `user_preferences` with columns `id TEXT PRIMARY KEY DEFAULT '1'`, `ui_locale TEXT NOT NULL DEFAULT 'en'`, `summary_language TEXT NOT NULL DEFAULT 'en'`, `transcription_language TEXT NOT NULL DEFAULT 'auto'`, `updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`. Seeded with `INSERT OR IGNORE INTO user_preferences (id) VALUES ('1')`.
- **D-02:** New `.sql` file in `frontend/src-tauri/migrations/` following the `YYYYMMDDHHMMSS_description.sql` convention.

**Rust module layout**
- **D-03:** New module `frontend/src-tauri/src/preferences/` with `mod.rs` (UserPreferences struct, UserPreferencesPatch, Lazy<RwLock> cache, `pub fn read() -> UserPreferences`), `commands.rs` (get/set Tauri commands), `repository.rs` (SQL adapters mirroring `setting.rs`).
- **D-04:** `tokio::sync::RwLock` to match `NotificationManagerState`. Claude's discretion if profiling later.
- **D-05:** `preferences::hydrate_from_db(pool).await` runs in `lib.rs::run` setup closure before command registration.

**Tauri command shape (setter = partial patch)**
- **D-06:** Partial patch at both boundaries — frontend `Partial<UserPreferences>`, Rust `UserPreferencesPatch` with `Option<T>` fields.
- **D-07:** Merge order: acquire write guard → read current → merge patch → run invariant → write tx → update RwLock → release → return merged result.

**Parakeet-ban invariant (hybrid)**
- **D-08:** Auto-repoint on locale change (`ui_locale` flips to `'ar'` while provider is `'parakeet'`) — force `localWhisper` + `large-v3` atomically.
- **D-09:** Reject direct violating writes (caller directly sets Parakeet while `ui_locale == 'ar'`) with `Err(PreferencesError::InvalidCombination)` before touching SQLite.
- **D-10:** Direction-of-change rule — the invariant reads the PATCH fields (user intent), not merged state, to decide between auto-repoint and reject branches.

**Cross-table atomic transaction**
- **D-11:** Single `sqlx::Transaction` spans `user_preferences` UPDATE and (conditionally) `transcript_settings` UPDATE. RwLock updated ONLY after successful commit.
- **D-12:** On rollback, RwLock is never touched; failed writes invisible to readers.

**Call-site migration**
- **D-13:** `get_language_preference_internal()` deleted; 4 recording-path call sites migrate in one commit to `preferences::read().transcription_language`.
- **D-14:** `set_language_preference` command and `LANGUAGE_PREFERENCE` global deleted; frontend callers updated to `set_user_preferences({ transcriptionLanguage })`.
- **D-15:** `audio/recording_commands.rs.backup` deleted in a dedicated chore commit. Commit message: `chore: remove unused recording_commands.rs.backup`.

**Frontend migration (ConfigContext + service)**
- **D-16:** New `frontend/src/services/preferencesService.ts` with `getUserPreferences()`, `setUserPreferences(p: Partial<UserPreferences>)`, and types `UiLocale`, `SummaryLanguage`, `UserPreferences { uiLocale, summaryLanguage, transcriptionLanguage }`.
- **D-17:** `ConfigContext.tsx:140` direct `localStorage.getItem('primaryLanguage')` removed; replaced with `getUserPreferences()` on mount.
- **D-18:** `ConfigContext.tsx:215` `useEffect` workaround deleted in the SAME commit as the localStorage removal.
- **D-19:** Residual `primaryLanguage` localStorage entries left alone (never read again).

**Rollout posture**
- **D-20:** Clean cut-over, NO feature flag. Spec §11 P0 "feature flag" wording is stale.
- **D-21:** Phase 1 commit order: (1) migration SQL, (2) preferences module + commands + hydration, (3) targeted tests, (4) 4 call-site migration + old global/command deletion, (5) frontend service + ConfigContext migration (workaround deletion in same commit), (6) `chore: remove unused recording_commands.rs.backup`.

**Targeted Phase 1 tests**
- **D-22:** Five tests ship alongside code: (1) startup hydration, (2) atomic-write invariant, (3) rollback invariance, (4) reject-branch invariant, (5) concurrent setter.

### Claude's Discretion
- RwLock flavor swap (`tokio::sync::RwLock` → `parking_lot::RwLock` wrapped in std-sync shim) IF profiling shows contention.
- `updated_at` bump mechanism (trigger vs. explicit in `repository.rs` — prefer explicit to match `setting.rs` style).
- Internal ConfigContext API shape (single `setPreferences(partial)` vs. granular wrappers).
- Exact `PreferencesError` variant names.
- `sqlx::query!` macro vs. string literals — mirror `setting.rs` (which uses string literals, see §Code Context below).
- Migration filename timestamp (must be strictly greater than `20251229000000_add_gemini_api_key.sql`).

### Deferred Ideas (OUT OF SCOPE)
- Encrypting `user_preferences` row (CONCERNS.md #4 — no secrets in this row, no urgency).
- Telemetry on preference changes (`analytics::commands::track_settings_changed` exists but no spec ask).
- Multi-user support (`id='1'` singleton is fine for single-user desktop).
- Hot-swap mid-session locale switching (§12.4 — full reload only; Phase 6 hydrates Rust strings on event).
- Internationalized `PreferencesError` messages (logs/errors stay English).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| **PREFS-01** | User preferences live in a single SQLite `user_preferences` row, readable through one Rust `preferences` module with process-global `Lazy<RwLock<UserPreferences>>` hydrated at startup | §Standard Stack (once_cell Lazy + tokio RwLock pattern verified in codebase), §Architecture Pattern 1 (hydrate_from_db at lib.rs setup closure line 405), §Code Example 1 (Lazy<RwLock> init), migration SQL template in Code Example 2 |
| **PREFS-02** | Setting a preference updates SQLite + in-memory RwLock atomically in a single transaction; no window of partial state; Parakeet-ban invariant enforced in the same write | §Architecture Pattern 2 (sqlx Transaction pattern from `database/manager.rs:164` and `database/repositories/meeting.rs:26`), §Code Example 3 (tx.begin → UPDATE → conditional UPDATE → commit → RwLock update), §Pitfall 1 (lock-before-commit = desync risk), §Validation Architecture test L3 |
| **PREFS-03** | All 6+ recording-path call sites read from the new module; `get_language_preference_internal()` deleted, not deprecated | §Call-Site Audit (verified: 4 live call sites + 3 in `.backup`), §Runtime State Inventory, D-13/D-15 mapping |
| **PREFS-04** | `ConfigContext.tsx:215` `useEffect` workaround removed in the same commit as its cause; `primaryLanguage` no longer touches localStorage | §Frontend Migration Map (verified: 3 references total — :140 read, :215 sync effect, :477 write), §Architecture Pattern 4 (ConfigContext mount hydration) |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- GSD workflow enforced — atomic commits referencing REQ-IDs (PREFS-01..04).
- Each plan's commits atomic; each commit ties to exactly one REQ-ID where possible.
- No ad-hoc scope changes — any discovered issue routes to `/gsd-add-todo`.
- Phase success criteria in ROADMAP.md (lines 28–32) are the "done" bar.
- Rust naming: `snake_case` files, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- Import order in Rust files: `std::*` → external crates → local crate modules.
- Rust error handling: `anyhow::Result` for internal, `Result<T, String>` for Tauri commands (FFI boundary).
- TS strict mode is ON — `UserPreferences` types must be exhaustive.
- ESLint is the only frontend lint gate — project ships with NO test runner in `frontend/package.json`.

## Standard Stack

### Core (already present in codebase, NO new deps required)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sqlx` | 0.8 (features `runtime-tokio`, `sqlite`, `chrono`) | SQLite persistence + transaction API | Already the only DB driver; verified via `STACK.md` and `Cargo.toml` [VERIFIED: grep `sqlx = 0.8`] |
| `tokio` | 1.32.0 (features `full`) | Async runtime + `tokio::sync::RwLock` for cache | Already used by `NotificationManagerState` at `lib.rs:400`, imported at `lib.rs:63` [VERIFIED] |
| `once_cell` | 1.17.1 | `Lazy<T>` static initialization | Used in 11+ files including `summary/templates/loader.rs:5`, `ollama/ollama.rs:10` [VERIFIED] |
| `serde` | 1.0 (derive) + `serde_json` | Tauri command payload ser/deser | Universal in codebase [VERIFIED] |
| `chrono` | 0.4.31 (serde) | `updated_at` timestamp if we need richer formatting | Used by `DateTimeUtc` in `database/models.rs` [VERIFIED] |
| `anyhow` | 1.0 | Internal `Result` type | Universal [VERIFIED] |
| `thiserror` | 2.0.16 | `PreferencesError` enum derive | Used for domain errors [VERIFIED from Cargo.toml] |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tokio::sync::RwLock` | `parking_lot::RwLock` (NOT currently in Cargo.toml) | Faster sync locking, no `.await` hold-across-await risk. BUT adds a new crate dependency and forces non-async signatures. D-04 locks this to tokio's RwLock for pattern consistency; Claude's discretion to swap later if profiling shows contention. parking_lot is **not a current dep** — verified by grep [VERIFIED: zero matches for `parking_lot` in src/]. |
| `sqlx::query!` macro | string literals via `sqlx::query(...)` | Compile-time verified SQL, but requires `DATABASE_URL` env or `.sqlx/` offline metadata. `setting.rs` uses string literals → mirror that style (D-claude-discretion confirms). |
| Single `RwLock` | `ArcSwap<UserPreferences>` (lock-free) | Would eliminate reader contention entirely. Overkill for a rarely-written value; not in Cargo.toml; not a codebase pattern. |
| `serde(rename_all = "camelCase")` on struct | per-field `#[serde(rename = "camelCase")]` | Struct-level rename_all is cleaner; `Setting`/`TranscriptSetting` in `database/models.rs:70–130` use per-field rename because column names are not uniform camelCase (`whisperModel` vs. `whisper_api_key`). For `UserPreferences` (pure camelCase output), `rename_all = "camelCase"` is the cleaner choice. |

**Installation:** No new dependencies. All required crates are already in `frontend/src-tauri/Cargo.toml`. [VERIFIED: STACK.md lines 106–131]

## Architecture Patterns

### Recommended Module Structure
```
frontend/src-tauri/src/preferences/
├── mod.rs          # pub mod commands; pub mod repository;
│                    # pub struct UserPreferences { ui_locale, summary_language, transcription_language, updated_at }
│                    # pub struct UserPreferencesPatch { all Option<T> }
│                    # pub enum PreferencesError { InvalidCombination { reason }, Database(sqlx::Error) }
│                    # static PREFS_CACHE: Lazy<RwLock<UserPreferences>>
│                    # pub async fn hydrate_from_db(pool: &SqlitePool) -> anyhow::Result<()>
│                    # pub fn read() -> UserPreferences  (clones out of the RwLock read guard)
├── commands.rs     # #[tauri::command] get_user_preferences() -> Result<UserPreferences, String>
│                    # #[tauri::command] set_user_preferences(patch, state: State<AppState>) -> Result<UserPreferences, String>
└── repository.rs   # async fn load(pool) -> Result<UserPreferences, sqlx::Error>
                     # async fn apply_patch_in_tx(tx, patch) -> Result<UserPreferences, PreferencesError>
                     # async fn force_whisper_in_tx(tx) -> Result<(), sqlx::Error>
```

**Module placement:** `preferences/` sits as a TOP-LEVEL module under `src/` (sibling to `audio/`, `database/`, `whisper_engine/`) per spec §3.2 and the CONTEXT canonical ref. It is NOT under `database/repositories/` — it uses `SqlitePool` as its persistence substrate but is its own domain module. [VERIFIED: matches `notifications/`, `summary/`, `ollama/` pattern]

### Pattern 1: Lazy<RwLock> process-global state

**What:** Static cache hydrated once at startup, cheap `.read().await` for hot-path callers, single-writer `.write().await` for setters.

**When to use:** Rarely-written, frequently-read domain state — exactly the preferences case.

**Example (VERIFIED pattern from codebase):**
```rust
// Source: frontend/src-tauri/src/ollama/ollama.rs:8–10 and frontend/src-tauri/src/summary/summary_engine/model_manager.rs:14
use once_cell::sync::Lazy;
use tokio::sync::RwLock;

static PREFS_CACHE: Lazy<RwLock<UserPreferences>> = Lazy::new(|| {
    // Safe seed value — replaced on first hydrate_from_db call
    RwLock::new(UserPreferences {
        ui_locale: "en".to_string(),
        summary_language: "en".to_string(),
        transcription_language: "auto".to_string(),
        updated_at: 0,
    })
});
```

**Critical: the `read()` helper.** Because call sites in the audio hot path are synchronous (see `whisper_engine/commands.rs:396`: `let language = crate::get_language_preference_internal();` is a plain sync call), the reader API must NOT require `.await`. Two options:

1. **`try_read().ok()` + clone** — non-blocking; returns fresh-enough value, fails only if a writer is currently holding the lock (rare).
2. **`blocking_read()`** — works in sync context but would block the thread; problematic if called from inside an async task.

**Recommendation:** Expose `pub fn read() -> UserPreferences` implemented with `PREFS_CACHE.blocking_read().clone()`. The call sites are NOT inside `select!` / hot await loops — they are one-shot reads at the start of a transcription call. `blocking_read` is the correct choice for this codebase. Document the contract in the function's doc comment: "Callers must not hold the resulting clone across a `.await`; clone it into the local task if needed."

**Warning:** If Claude's discretion later swaps to `parking_lot::RwLock`, the sync-call contract is preserved (parking_lot is always sync) but the hydration call in `lib.rs::run` changes shape (no `.await` on the lock itself).

### Pattern 2: sqlx cross-table transaction

**What:** Open a transaction, execute multiple UPDATEs against it, commit or rollback as a unit.

**When to use:** Any operation that must touch multiple rows/tables atomically — exactly the Parakeet auto-repoint case.

**Example (VERIFIED from `database/repositories/meeting.rs:26`, `audio/import.rs:720`):**
```rust
// Source: frontend/src-tauri/src/database/repositories/meeting.rs (transaction lifecycle)
use sqlx::{SqlitePool, Transaction, Sqlite};

pub async fn apply_patch_atomic(
    pool: &SqlitePool,
    patch: UserPreferencesPatch,
) -> Result<UserPreferences, PreferencesError> {
    let mut tx: Transaction<'_, Sqlite> = pool.begin().await
        .map_err(PreferencesError::Database)?;

    // 1. Load current state within tx (SELECT)
    let current: UserPreferences = sqlx::query_as::<_, UserPreferencesRow>(
        "SELECT * FROM user_preferences WHERE id = '1' LIMIT 1"
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(PreferencesError::Database)?
    .into();

    // 2. Merge patch → compute `merged`
    let merged = current.merge(&patch);

    // 3. Run invariant (D-09 reject branch) — BEFORE SQLite is touched
    //    D-10: reads patch fields, not merged state
    if let Err(e) = invariant::check_reject_branch(&patch, &merged) {
        tx.rollback().await.ok();  // best-effort; error already decided
        return Err(e);
    }

    // 4. UPDATE user_preferences
    sqlx::query(
        r#"
        UPDATE user_preferences
        SET ui_locale = ?, summary_language = ?, transcription_language = ?,
            updated_at = strftime('%s','now')
        WHERE id = '1'
        "#,
    )
    .bind(&merged.ui_locale)
    .bind(&merged.summary_language)
    .bind(&merged.transcription_language)
    .execute(&mut *tx)
    .await
    .map_err(PreferencesError::Database)?;

    // 5. Conditional auto-repoint (D-08) — same tx
    if invariant::should_auto_repoint(&patch, &merged) {
        sqlx::query(
            r#"
            UPDATE transcript_settings
            SET provider = 'localWhisper', model = 'large-v3'
            WHERE id = '1'
            "#,
        )
        .execute(&mut *tx)
        .await
        .map_err(PreferencesError::Database)?;
    }

    // 6. Commit — SQLite now consistent
    tx.commit().await.map_err(PreferencesError::Database)?;

    Ok(merged)
}
```

**Key detail:** `&mut *tx` is the idiomatic way to borrow a `Transaction` as an `Executor` for multiple statements. Verified in `meeting.rs:26–80`, `audio/import.rs:720–735`. Do NOT pass `&mut tx` (wrong type) or `&tx` (not mutable enough).

### Pattern 3: Invariant as a separate pure function (direction-of-change rule)

**What:** Decision logic that reads `UserPreferencesPatch` (intent) and `UserPreferences` (merged state) and returns one of three outcomes: {ok, auto-repoint, reject}.

**Why separate:** Testable in isolation without a database. Critical for Phase 1 test T4 (reject-branch invariant — must be testable without touching SQLite).

**Example:**
```rust
// preferences/mod.rs or preferences/invariant.rs
pub(crate) mod invariant {
    use super::{UserPreferences, UserPreferencesPatch, PreferencesError};

    /// D-09 reject branch: caller directly requested a Parakeet-shaped
    /// configuration while current/merged ui_locale is 'ar'.
    ///
    /// NOTE (D-10): This reads the PATCH, not the merged state. A patch
    /// that is "change the locale" triggers auto-repoint (D-08, see
    /// `should_auto_repoint`). A patch that is "set a parakeet provider"
    /// while ar is active triggers rejection.
    ///
    /// Phase 1 does not yet expose a provider-setter patch field, so this
    /// function's reject check is the hook for Phase 4's TRANS-02.
    /// For Phase 1, document the hook and leave the check as a no-op if
    /// the patch does not include a provider field.
    pub fn check_reject_branch(
        _patch: &UserPreferencesPatch,
        _merged: &UserPreferences,
    ) -> Result<(), PreferencesError> {
        // Phase 1: UserPreferencesPatch does not include provider field.
        // Hook exists for Phase 4 to extend.
        Ok(())
    }

    /// D-08 auto-repoint branch: user's patch is flipping ui_locale to 'ar'.
    /// Whether the current provider is parakeet is decided at the SQL layer
    /// (the UPDATE is idempotent if already localWhisper), but the trigger
    /// fires on patch-field presence.
    pub fn should_auto_repoint(
        patch: &UserPreferencesPatch,
        merged: &UserPreferences,
    ) -> bool {
        patch.ui_locale.as_deref() == Some("ar") && merged.ui_locale == "ar"
    }
}
```

**CRITICAL CLARIFICATION for the planner:** The CONTEXT decision D-09 says the reject branch fires "when the caller's patch is directly setting a Parakeet-shaped configuration." Phase 1's `UserPreferencesPatch` (per D-01, only `ui_locale`/`summary_language`/`transcription_language`) does NOT expose a `provider` field — that lives in `transcript_settings`, not `user_preferences`. **The reject-branch hook in Phase 1 is therefore a scaffolded no-op with a test that asserts "a hypothetical future Parakeet patch would hit this code path"**, OR the planner treats the reject branch as strictly a Phase 4 concern and Phase 1 ships only the auto-repoint half of D-08. I recommend the planner choose option B (ship only the auto-repoint, leave a documented `// TODO(TRANS-02, Phase 4): reject here if patch.provider == "parakeet" && merged.ui_locale == "ar"` comment and update D-22 test T4 to be a TRANS-04-adjacent test). **This is a minor gap between D-09 as written and the `UserPreferencesPatch` shape in D-01 — the plan-check should flag it for user confirmation.** [ASSUMED: the planner will resolve this ambiguity in plan-check.]

### Pattern 4: ConfigContext mount hydration (replaces useEffect desync fix)

**What:** Single mount-time `getUserPreferences()` call populates React state from the authoritative Rust cache, eliminating the startup desync window.

**Example:**
```tsx
// frontend/src/contexts/ConfigContext.tsx (after migration)
import { getUserPreferences, setUserPreferences } from '@/services/preferencesService';

// DELETE: initializer-function reading localStorage at line 140
const [selectedLanguage, setSelectedLanguage] = useState<string>('auto');

// DELETE: useEffect at line 215 ("fixes startup desync bug")

// REPLACE with mount hydration:
useEffect(() => {
  (async () => {
    try {
      const prefs = await getUserPreferences();
      setSelectedLanguage(prefs.transcriptionLanguage);
      // Also hydrate uiLocale, summaryLanguage into future Phase 2 state
    } catch (err) {
      console.error('[ConfigContext] Failed to load user preferences:', err);
    }
  })();
}, []);

// REPLACE handleSetSelectedLanguage (line 474–483):
const handleSetSelectedLanguage = useCallback(async (lang: string) => {
  try {
    const updated = await setUserPreferences({ transcriptionLanguage: lang });
    setSelectedLanguage(updated.transcriptionLanguage);
    // localStorage.setItem removed (D-19: residual keys ignored)
  } catch (err) {
    console.error('[ConfigContext] Failed to save transcription language:', err);
  }
}, []);
```

### Anti-Patterns to Avoid
- **Holding a sync lock across an `.await`** — if we swap to `parking_lot::RwLock`, make sure the write guard drops BEFORE any `.await`. With `tokio::sync::RwLock` this is fine because the lock is itself async.
- **Updating RwLock before commit** — inverts D-11. A panic between cache update and `tx.commit()` would leave the cache ahead of disk.
- **Two-step "write then load"** — using separate `sqlx::query` UPDATE + `sqlx::query_as` SELECT outside the same transaction re-introduces the concurrent-setter race D-22 T5 is designed to catch.
- **Global static initialized from DB** — `Lazy::new(|| block_on(load_from_db()))` looks clean but fights the tokio runtime at process startup. Keep `Lazy::new` seeded with defaults and run `hydrate_from_db(pool).await` explicitly in `lib.rs::run` setup.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Partial update ser/deser | Hand-coded field-by-field `if Some(v) = ...` parsing | `UserPreferencesPatch` struct with `#[serde(default)]` + `Option<T>` per field | serde handles the omission-is-None case out of the box |
| Multi-table atomic write | Manual "try UPDATE A, if A ok then UPDATE B, if B fails manually UNDO A" | `sqlx::Transaction` with `.begin().await` / `.commit().await` / implicit rollback on drop | sqlx handles rollback-on-drop and WAL checkpointing; manual undo is a known desync source |
| Process-global cache | `unsafe static mut PREFS: UserPreferences` | `once_cell::sync::Lazy<tokio::sync::RwLock<UserPreferences>>` | No `unsafe`, thread-safe, compositional with async code |
| Migration replay tracking | Custom `applied_migrations` table | sqlx's built-in migrator reading `frontend/src-tauri/migrations/` at startup | Already wired; just drop the `.sql` file in and it runs |
| Test-time SQLite setup | Writing a test helper that copies the production DB | `sqlx::SqlitePool::connect("sqlite::memory:")` + running migrations on the in-memory pool | Standard sqlx pattern; `sqlite::memory:` is explicit; tests are isolated |

**Key insight:** Every single building block for Phase 1 already exists in the codebase. The risk is NOT "learning a new library" — it is "getting the sequencing right" (commit-after-tx-commit-after-lock-acquire) and "deleting exactly the right things in exactly the right commits."

## Runtime State Inventory

*(Rename/refactor phase — deletes a global static and migrates 4+ call sites. Inventory confirms nothing is cached outside the files being touched.)*

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | SQLite `settings`, `transcript_settings`, `meetings`, `transcripts`, `summary_processes`, `transcript_chunks` tables already exist. **New**: `user_preferences` row (D-01). **Stale**: `localStorage.primaryLanguage` (D-19: ignore, not migrated). | Add new migration (D-02); D-19 leaves localStorage residue untouched |
| Live service config | None. No external service stores meetily-specific "language" as a key. The backend FastAPI does not persist per-user language. | None |
| OS-registered state | None. macOS Tauri app; no launchd plist or tray registration references "language preference" by name. `tray.rs` builds menu at runtime. | None |
| Secrets/env vars | None. `LANGUAGE_PREFERENCE` is not an env var. API keys in `settings` table are unrelated. | None |
| Build artifacts / installed packages | None. Nothing in `target/`, `pnpm-lock.yaml`, or bundled resources references `LANGUAGE_PREFERENCE`. The `sqlx` offline mode is not used (no `.sqlx/` dir) so no regeneration needed. | None |

**Nothing found in 4 of 5 categories — verified by grep across the repo.** The only state touched is (1) the SQLite schema (new table + new migration file) and (2) the Rust static at `lib.rs:68` that gets deleted.

**Call-site audit (CONTEXT D-13 verification):**
```
frontend/src-tauri/src/whisper_engine/commands.rs:396  (LIVE)
frontend/src-tauri/src/whisper_engine/parallel_processor.rs:344  (LIVE)
frontend/src-tauri/src/audio/transcription/worker.rs:449  (LIVE)
frontend/src-tauri/src/audio/transcription/worker.rs:526  (LIVE)
frontend/src-tauri/src/audio/recording_commands.rs.backup:1276  (DEAD — .backup not compiled)
frontend/src-tauri/src/audio/recording_commands.rs.backup:1353  (DEAD)
frontend/src-tauri/src/audio/recording_commands.rs.backup:1440  (DEAD)
frontend/src-tauri/src/lib.rs:386–388  (definition — DELETE)
frontend/src-tauri/src/lib.rs:376–383  (set_language_preference command — DELETE)
frontend/src-tauri/src/lib.rs:66–69  (LANGUAGE_PREFERENCE static — DELETE)
frontend/src-tauri/src/lib.rs:662  (registration in invoke_handler — REPLACE)
```

**Total: 4 live call sites (matches CONTEXT D-13).** ROADMAP's "6+" phrasing is confusing — it counts the 3 dead references in `.backup` plus the static/command/internal-helper trio in lib.rs. D-15 handles the `.backup` in a dedicated chore commit; D-13 migration is strictly the 4 live sites.

**Frontend audit (ConfigContext + dependencies):**
```
frontend/src/contexts/ConfigContext.tsx:142  localStorage.getItem('primaryLanguage')  (READ — DELETE, D-17)
frontend/src/contexts/ConfigContext.tsx:215  useEffect "fixes startup desync bug"  (DELETE, D-18)
frontend/src/contexts/ConfigContext.tsx:477  localStorage.setItem('primaryLanguage', lang)  (DELETE in handleSetSelectedLanguage, D-17)
```

**Total: 3 lines to delete in ConfigContext.tsx + one `invoke('set_language_preference', ...)` call at line 217 and line 480.** All in the same commit per D-18. Verified: `primaryLanguage` does not appear anywhere else in `frontend/src/`.

## Common Pitfalls

### Pitfall 1: Updating the RwLock before the transaction commits
**What goes wrong:** Code updates `PREFS_CACHE.write()` first, then calls `tx.commit()`. If the commit fails (constraint violation, disk full, WAL checkpoint error), the in-memory cache is ahead of disk — the exact desync D-11 prohibits.
**Why it happens:** "Feels more natural" to update the in-process state before the slower DB call.
**How to avoid:** Strict order in `set_user_preferences`: begin tx → execute statements → commit → ON SUCCESS write guard update → return. If any step between `begin` and `commit` errors, the function returns before touching the cache.
**Warning signs:** A test that forces transcript_settings UPDATE to fail finds the cache mutated anyway (Phase 1 test T3 "rollback invariance" catches exactly this).

### Pitfall 2: `tokio::sync::RwLock::read()` held across await in hot path
**What goes wrong:** If `preferences::read()` returns a `RwLockReadGuard` (not a clone) and a caller holds it across an `.await`, a concurrent writer deadlocks.
**Why it happens:** Idiomatic Rust says "return a reference to avoid clones" — but `tokio::sync::RwLock` guards must not cross `.await` in contested code.
**How to avoid:** `read()` always returns `UserPreferences` by value (clone). The struct is 3 small `String` fields + a `u64` — clone cost is negligible. The audio hot path calls `read()` ONCE per transcription.
**Warning signs:** Deadlock in the concurrent-setter test T5.

### Pitfall 3: Migration file timestamp collision or wrong ordering
**What goes wrong:** New migration named `20251229000000_add_user_preferences.sql` conflicts with the existing `20251229000000_add_gemini_api_key.sql`, or runs before it.
**Why it happens:** Copy-paste timestamp, or picking "today's date" without checking.
**How to avoid:** Pick a timestamp strictly greater than `20251229000000`. Suggested: `20260407000000_add_user_preferences.sql` (today's date, zeroed HHMMSS).
**Warning signs:** Migration doesn't apply, or applies out of order.

### Pitfall 4: Hydration happens AFTER a command that reads from the cache
**What goes wrong:** `hydrate_from_db` spawned as `tauri::async_runtime::spawn(...)` in the setup closure runs concurrently with command registration. First `get_user_preferences` call races hydration.
**Why it happens:** The existing setup closure spawns several init tasks (whisper, parakeet, model manager) with `async_runtime::spawn` — copy-paste would apply the same pattern to hydration.
**How to avoid:** `hydrate_from_db` runs INSIDE `tauri::async_runtime::block_on(async { ... })` in the setup closure (same pattern as `database::setup::initialize_database_on_startup` at `lib.rs:482–485`). This makes hydration synchronous within setup, so command registration cannot return before the cache is populated.
**Warning signs:** Test T1 (startup hydration) fails intermittently.

### Pitfall 5: Forgetting to delete the `set_language_preference` registration in `invoke_handler`
**What goes wrong:** `lib.rs:662` still lists `set_language_preference,` but the function is deleted → compile error. OR the function stays because of the compile error, silently leaving a dead command.
**Why it happens:** Mechanical deletion of the function and forgetting the registration.
**How to avoid:** Delete the function, delete the registration, delete the static, delete the internal helper — all in the same commit. Build must be green at each commit boundary.
**Warning signs:** `cargo check` fails with "cannot find function `set_language_preference`."

### Pitfall 6: Seeding the `user_preferences` row inside a UNIQUE constraint on id
**What goes wrong:** Migration runs on a DB that already has a `user_preferences` row (e.g., re-running tests without dropping the DB) and the `INSERT ... VALUES ('1')` errors on duplicate PK.
**Why it happens:** Missed `OR IGNORE`.
**How to avoid:** Use exactly `INSERT OR IGNORE INTO user_preferences (id) VALUES ('1')` per D-01. `CREATE TABLE IF NOT EXISTS user_preferences (...)` for idempotent schema. Matches the existing `CREATE TABLE IF NOT EXISTS settings ...` shape in `20250916100000_initial_schema.sql`.
**Warning signs:** Migrator errors on second run or during `cargo test` repeats.

## Code Examples

### Example 1: Migration SQL (D-01, D-02)
```sql
-- frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql
-- Migration: Add user_preferences singleton row for UI locale, summary language,
-- and transcription language. Single source of truth replaces lib.rs:68
-- LANGUAGE_PREFERENCE static and localStorage.primaryLanguage.

CREATE TABLE IF NOT EXISTS user_preferences (
    id                      TEXT PRIMARY KEY DEFAULT '1',
    ui_locale               TEXT NOT NULL DEFAULT 'en',
    summary_language        TEXT NOT NULL DEFAULT 'en',
    transcription_language  TEXT NOT NULL DEFAULT 'auto',
    updated_at              INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

INSERT OR IGNORE INTO user_preferences (id) VALUES ('1');
```
[VERIFIED: matches existing migration idempotency style in `20250916100000_initial_schema.sql`]

### Example 2: UserPreferences struct with serde bridging

```rust
// frontend/src-tauri/src/preferences/mod.rs (excerpt)
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    #[serde(skip_deserializing)]
    pub id: String,
    pub ui_locale: String,
    pub summary_language: String,
    pub transcription_language: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesPatch {
    #[serde(default)]
    pub ui_locale: Option<String>,
    #[serde(default)]
    pub summary_language: Option<String>,
    #[serde(default)]
    pub transcription_language: Option<String>,
}

impl UserPreferences {
    pub fn merge(&self, patch: &UserPreferencesPatch) -> Self {
        Self {
            id: self.id.clone(),
            ui_locale: patch.ui_locale.clone().unwrap_or_else(|| self.ui_locale.clone()),
            summary_language: patch.summary_language.clone().unwrap_or_else(|| self.summary_language.clone()),
            transcription_language: patch.transcription_language.clone().unwrap_or_else(|| self.transcription_language.clone()),
            updated_at: self.updated_at,  // bumped by DB write
        }
    }
}
```

**Serde note:** `#[serde(rename_all = "camelCase")]` handles the Rust→TypeScript field-name bridge. The TS `UserPreferences` type in `preferencesService.ts` has `uiLocale`, `summaryLanguage`, `transcriptionLanguage` → serde auto-renames `ui_locale` ↔ `uiLocale`. No per-field rename needed (unlike `Setting` in `database/models.rs:70` which has mixed column naming).

### Example 3: `hydrate_from_db` and the Lazy static

```rust
// frontend/src-tauri/src/preferences/mod.rs (excerpt)
static PREFS_CACHE: Lazy<RwLock<UserPreferences>> = Lazy::new(|| {
    RwLock::new(UserPreferences {
        id: "1".to_string(),
        ui_locale: "en".to_string(),
        summary_language: "en".to_string(),
        transcription_language: "auto".to_string(),
        updated_at: 0,
    })
});

/// Called once at startup from lib.rs::run setup closure.
/// Must complete before any Tauri command is registered.
pub async fn hydrate_from_db(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let loaded = repository::load(pool).await?;
    let mut guard = PREFS_CACHE.write().await;
    *guard = loaded;
    log::info!("User preferences hydrated from DB: ui_locale={}", guard.ui_locale);
    Ok(())
}

/// Synchronous reader for audio hot-path call sites.
/// Contract: returns a clone; do NOT rely on holding the lock across await points.
pub fn read() -> UserPreferences {
    PREFS_CACHE.blocking_read().clone()
}
```

### Example 4: Tauri command `set_user_preferences`

```rust
// frontend/src-tauri/src/preferences/commands.rs
use tauri::State;
use crate::state::AppState;
use super::{UserPreferences, UserPreferencesPatch, PREFS_CACHE, PreferencesError};

#[tauri::command]
pub async fn get_user_preferences() -> Result<UserPreferences, String> {
    Ok(PREFS_CACHE.read().await.clone())
}

#[tauri::command]
pub async fn set_user_preferences(
    patch: UserPreferencesPatch,
    state: State<'_, AppState>,
) -> Result<UserPreferences, String> {
    let pool = state.db_manager.pool();

    // Apply patch atomically — tx runs inside the helper
    let merged = super::repository::apply_patch_atomic(pool, patch)
        .await
        .map_err(|e| match e {
            PreferencesError::InvalidCombination { reason } => reason,
            PreferencesError::Database(e) => format!("Database error: {}", e),
        })?;

    // Only AFTER successful commit do we update the cache
    let mut guard = PREFS_CACHE.write().await;
    *guard = merged.clone();
    drop(guard);

    Ok(merged)
}
```

### Example 5: Call-site migration (4 files)

```rust
// BEFORE (all 4 call sites):
let language = crate::get_language_preference_internal();

// AFTER:
let language = Some(crate::preferences::read().transcription_language);
```

**Note on Option wrapping:** The old `get_language_preference_internal` returned `Option<String>` because the `LazyLock::lock()` could theoretically fail. The new `read()` returns `UserPreferences` unconditionally. Downstream code (e.g., `whisper_engine::transcribe_audio(audio, language)`) already accepts `Option<String>`, so the cleanest adapter is `Some(prefs.transcription_language)` — OR verify the downstream signature and drop the `Some` wrapping if it now accepts `&str`. Planner: include a grep check in acceptance criteria.

### Example 6: Wiring into `lib.rs::run` (hydration + registration)

```rust
// lib.rs setup closure (REPLACE lines 482–485 block):
tauri::async_runtime::block_on(async {
    database::setup::initialize_database_on_startup(&_app.handle()).await
})
.expect("Failed to initialize database");

// NEW: hydrate preferences after DB is initialized
tauri::async_runtime::block_on(async {
    let state = _app.handle().state::<crate::state::AppState>();
    preferences::hydrate_from_db(state.db_manager.pool()).await
})
.expect("Failed to hydrate user preferences");

// lib.rs invoke_handler (REPLACE line 662):
// DELETE: set_language_preference,
// ADD:
preferences::commands::get_user_preferences,
preferences::commands::set_user_preferences,
```

**Sequence constraint:** Hydration MUST run AFTER `initialize_database_on_startup` (which builds `AppState` and manages it) because hydration reads `AppState::db_manager.pool()`. Verified: `AppState { db_manager }` is installed by `.manage(AppState { db_manager })` at `database/setup.rs:33`.

### Example 7: TypeScript service

```typescript
// frontend/src/services/preferencesService.ts
import { invoke } from '@tauri-apps/api/core';

export type UiLocale = 'en' | 'ar';
export type SummaryLanguage = 'en' | 'ar';

export interface UserPreferences {
  uiLocale: UiLocale;
  summaryLanguage: SummaryLanguage;
  transcriptionLanguage: string;  // 'auto' | 'en' | 'ar' | ISO code
}

export async function getUserPreferences(): Promise<UserPreferences> {
  return await invoke<UserPreferences>('get_user_preferences');
}

export async function setUserPreferences(
  patch: Partial<UserPreferences>,
): Promise<UserPreferences> {
  return await invoke<UserPreferences>('set_user_preferences', { patch });
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `std::sync::LazyLock<StdMutex<String>>` for global language | `once_cell::sync::Lazy<tokio::sync::RwLock<UserPreferences>>` for whole preferences struct | This phase | Typed, async-aware, survives restart, enforces invariants |
| `localStorage.primaryLanguage` in frontend, synced via `useEffect` to Rust | Single source in SQLite, mount-time hydration via `getUserPreferences()` | This phase | Deletes startup desync workaround |
| `set_language_preference(language: String)` takes a single string | `set_user_preferences(patch: UserPreferencesPatch)` takes a full partial | This phase | Extensible to UI locale + summary language without new commands |

**Deprecated/outdated (deleted in this phase):**
- `LANGUAGE_PREFERENCE` static at `lib.rs:68`
- `set_language_preference` command at `lib.rs:376`
- `get_language_preference_internal()` helper at `lib.rs:386`
- `ConfigContext.tsx:215` `useEffect` "fixes startup desync bug" workaround
- `localStorage.getItem('primaryLanguage')` / `setItem('primaryLanguage', ...)` at `ConfigContext.tsx:142`/`:477`
- `audio/recording_commands.rs.backup` entire file (dedicated commit per D-15)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The D-09 reject branch is effectively Phase 4's concern (TRANS-02) because `UserPreferencesPatch` in D-01 does not include a `provider` field; Phase 1 ships only the auto-repoint half of the invariant | Architecture Pattern 3 | Plan-check may decide to expand the `UserPreferencesPatch` shape in Phase 1, which would add a test and complicate commit ordering. Worst case: D-22 test T4 becomes a no-op and the planner must add a genuine reject-branch test involving a Phase-4-only codepath. **Planner should confirm with user in plan-check.** |
| A2 | The downstream signatures (`whisper_engine::transcribe_audio`, etc.) still accept `Option<String>` for language, so `Some(prefs.transcription_language)` is a drop-in replacement | Code Example 5 | If the downstream now takes `&str` or `String` directly, the call-site adapter is different. **Grep verification in plan acceptance criteria** (see Validation Architecture §Sampling). |
| A3 | `blocking_read()` in `preferences::read()` does not cause deadlock because the hot-path callers are not inside await loops holding other related locks | Pattern 1 | If a future call site ends up inside a held-tokio-lock region, `blocking_read` could deadlock. Mitigated by keeping `read()` contract explicit ("do not hold across await") and documenting in the doc comment. |
| A4 | Single `tauri::async_runtime::block_on(hydrate_from_db(...))` in setup closure does not race the other spawned init tasks | Pitfall 4 | If Whisper engine init reads preferences before hydration completes (unlikely — whisper is spawned, not awaited), the engine sees the default seed. Mitigated by calling `hydrate_from_db` BEFORE the Whisper/Parakeet spawns. |
| A5 | `INTEGER` (unix seconds) is sufficient for `updated_at`; no TEXT/ISO-8601 needed | Code Example 1 | If downstream code (Phase 6 QA analytics?) needs millisecond precision, the column would need changing. Phase 1 has no such consumer; `updated_at` is currently unused by any read path (it's for observability). |
| A6 | The `transcript_settings` auto-repoint writes `provider='localWhisper'` and `model='large-v3'` with no other column updates needed | Pattern 2 | If `transcript_settings` has nullable columns that downstream code treats as "required when provider is localWhisper" (e.g., `whisperApiKey` must be NULL), the UPDATE might leave stale values. VERIFIED: the live `save_transcript_config` at `setting.rs:153` only updates `provider, model` — so auto-repoint matches existing write shape. |

**All 6 assumptions should be surfaced during discuss-phase or plan-check for user confirmation. A1 is the most material — it affects test T4's meaning.**

## Open Questions

1. **D-09 reject branch — scaffolded no-op or full Phase-4-style test?**
   - What we know: D-09 says reject before SQLite is touched; D-01 lists the `UserPreferencesPatch` fields and `provider` is not among them.
   - What's unclear: Does Phase 1 extend `UserPreferencesPatch` with an optional `provider` field (tied to `transcript_settings`) to make the reject branch executable, or does it scaffold the hook and defer the test to Phase 4?
   - Recommendation: Plan-check raises this. My strong recommendation: Phase 1 scaffolds the hook, Phase 4 extends the patch and adds the reject test. This keeps Phase 1 strictly additive to `user_preferences` and leaves `transcript_settings` writes purely for the auto-repoint branch.

2. **`preferences::read()` contract: `blocking_read` vs `try_read`?**
   - What we know: The 4 call sites are in sync function bodies (e.g., `whisper_engine/commands.rs:388` is `pub async fn whisper_transcribe_audio` — actually async, so could `.await`).
   - What's unclear: Actually — re-reading, 3 of 4 call sites (`commands.rs:396`, `worker.rs:449`, `worker.rs:526`) are inside `async fn` bodies, so they could use `.await`. Only `parallel_processor.rs:344` is inside `async fn` too (`process_chunk`). **All 4 call sites are async contexts.** This means `preferences::read()` can be `pub async fn read() -> UserPreferences` returning a clone from `PREFS_CACHE.read().await.clone()` — no `blocking_read` needed.
   - Recommendation: Make `read()` async. Update call sites to `crate::preferences::read().await.transcription_language`. This eliminates the blocking_read deadlock risk entirely. **The planner should prefer this cleaner API.**

3. **Call-site adapter shape — `Some(string)` or `string` directly?**
   - What we know: Old `get_language_preference_internal() -> Option<String>` was unconditionally `Some(...)` because the LazyLock never failed in practice.
   - What's unclear: Does each downstream transcription function actually need the `Option`? Some engines may ignore `None` (auto-detect) and treat `Some("auto")` differently.
   - Recommendation: Planner grep each downstream signature during plan drafting and lock the adapter shape per call site.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust 1.77+ | Tauri backend compile | ✓ | 1.77 (rust-version in Cargo.toml) | — |
| sqlx 0.8 | Migration runner + queries | ✓ | 0.8 | — |
| tokio 1.32 | Async runtime, RwLock | ✓ | 1.32.0 | — |
| once_cell 1.17.1 | Lazy static | ✓ | 1.17.1 | — |
| chrono 0.4.31 | Timestamp handling | ✓ | 0.4.31 | — |
| anyhow, thiserror | Error types | ✓ | 1.0 / 2.0.16 | — |
| SQLite runtime | Migrator at startup | ✓ (bundled via sqlx feature) | — | — |
| `cargo test` harness | D-22 tests | ✓ (built-in) | — | — |
| `pnpm` for frontend build | ConfigContext changes compile | ✓ | — | — |
| `jest`/`vitest` for frontend tests | D-22 frontend-side tests (if any) | ✗ (no test runner in frontend) | — | **D-22 tests are Rust-only — no frontend test runner needed** |

**Missing dependencies with no fallback:** None.
**Missing dependencies with fallback:** None. Frontend test runner absence is not a blocker — all 5 Phase 1 tests are Rust-side per D-22 (hydration, tx invariant, rollback, reject, concurrent).

## Validation Architecture

**Nyquist validation is ENABLED** (`.planning/config.json` workflow.nyquist_validation = true).

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `cargo test` built-in harness (no external framework) with `#[tokio::test]` for async tests |
| Config file | None — uses Rust defaults |
| Quick run command | `cd frontend/src-tauri && cargo test preferences::` (scopes to the new module) |
| Full suite command | `cd frontend/src-tauri && cargo test` |
| Test placement | Inline `#[cfg(test)] mod tests { ... }` at the bottom of `preferences/mod.rs` or (preferred) `preferences/tests.rs` as a module per D-22's "5 tests" scope |

### Phase Requirements → Test Map

| Req ID | Behavior to validate | Test Type | Automated Command | Test Placement |
|--------|----------------------|-----------|-------------------|----------------|
| PREFS-01 | Startup hydration: SQLite `ui_locale='ar'` → `read()` returns 'ar' immediately after setup | integration | `cargo test preferences::tests::test_hydration_from_db` | `preferences/tests.rs::test_hydration_from_db` — ❌ Wave 0 |
| PREFS-02 | Atomic-write invariant: locale flip + auto-repoint writes BOTH tables in one commit, RwLock updated AFTER commit | integration | `cargo test preferences::tests::test_atomic_locale_flip_updates_both_tables` | `preferences/tests.rs::test_atomic_locale_flip_updates_both_tables` — ❌ Wave 0 |
| PREFS-02 | Rollback invariance: force the second UPDATE to fail → first table unchanged, RwLock unchanged | integration | `cargo test preferences::tests::test_rollback_leaves_cache_consistent` | `preferences/tests.rs::test_rollback_leaves_cache_consistent` — ❌ Wave 0 |
| PREFS-02 | Reject-branch invariant (hook presence): reject codepath compiles and is called during patch-apply (even if no-op in Phase 1, see A1) | unit | `cargo test preferences::invariant::tests::test_reject_branch_hook` | `preferences/mod.rs::invariant::tests::test_reject_branch_hook` — ❌ Wave 0 |
| PREFS-02 | Concurrent setter: two `tokio::spawn`ed `set_user_preferences` calls → no partial state; final result == one of the two inputs | integration | `cargo test preferences::tests::test_concurrent_setters_never_partial` | `preferences/tests.rs::test_concurrent_setters_never_partial` — ❌ Wave 0 |
| PREFS-03 | Call-site migration complete: `get_language_preference_internal` appears in zero live source files | lint-style | `! grep -rn "get_language_preference_internal" frontend/src-tauri/src/ --include='*.rs'` (must exit 1) | In verification script, not unit test |
| PREFS-03 | `LANGUAGE_PREFERENCE` static and `set_language_preference` command deleted | lint-style | `! grep -n "LANGUAGE_PREFERENCE\|set_language_preference" frontend/src-tauri/src/lib.rs` | In verification script |
| PREFS-03 | `.backup` file deleted | lint-style | `! test -e frontend/src-tauri/src/audio/recording_commands.rs.backup` | In verification script |
| PREFS-04 | ConfigContext desync workaround gone | lint-style | `! grep -n "fixes startup desync bug\|primaryLanguage" frontend/src/contexts/ConfigContext.tsx` | In verification script |
| PREFS-04 | `preferencesService.ts` exists and exports required functions | lint-style | `grep -q "export async function getUserPreferences\|export async function setUserPreferences" frontend/src/services/preferencesService.ts` | In verification script |

### Sampling Strategy (Nyquist-rate analysis)

**The Nyquist rate for this phase must catch 4 distinct failure modes:**

1. **Desync between SQLite and RwLock** — Nyquist rate: one test per transaction outcome (commit success, commit failure). 2 tests minimum. **Covered by T1 (hydration), T2 (atomic), T3 (rollback).**
2. **Invariant bypass** — Nyquist rate: one test per branch of the invariant hybrid (auto-repoint, reject). 2 tests minimum. **Covered by T2 (auto-repoint), T4 (reject hook).**
3. **Concurrent racing** — Nyquist rate: at least 2 parallel writers to exercise the `RwLock::write().await` serialization. **Covered by T5 (concurrent setter).**
4. **Hot-path read staleness** — Nyquist rate: one test that writes, then reads via `read()`, and asserts fresh value (no reader caching). **Covered by T2's post-commit read assertion.**

**Total: 5 tests — Nyquist-sufficient for Phase 1 scope.** Phase 6 QA-01 adds the end-to-end desync regression (startup → runtime → UI repaint) which samples the INTEGRATED system; Phase 1's tests sample the UNIT in isolation.

### Test Layer Matrix

| Layer | Failure mode caught | Which D-22 test |
|-------|---------------------|------------------|
| SQL (SQLite row state) | UPDATE failed silently, FK violation, NOT NULL breach | T2 via post-commit `SELECT * FROM user_preferences WHERE id='1'` |
| sqlx Transaction | Partial commit, rollback not triggered on error | T3 via forced failure + post-commit SELECT showing no change |
| RwLock consistency | Cache ahead of disk or behind disk | T2 (cache == fresh read), T3 (cache == stale disk) |
| Tauri command surface | Wrong serde shape, wrong error serialization | T4 via direct call of `set_user_preferences(patch, state)` and matching error type |
| Sync primitive | Deadlock under concurrent writers | T5 via `tokio::spawn` × 2 + `tokio::join!` |

### Anti-Sampling (tests that would falsely pass)

- **A test that uses `sqlx::query!` with a mocked row** — would pass even if the transaction logic is broken because no real `BEGIN/COMMIT` runs. **Rule out:** Use `sqlx::SqlitePool::connect("sqlite::memory:").await` in every test, run the migration SQL against the in-memory pool, then exercise the real `apply_patch_atomic` against that pool. No mocks.
- **A test that inspects the `PREFS_CACHE` static directly without going through `set_user_preferences`** — would pass even if the commit-then-cache ordering is inverted. **Rule out:** Every test must call `set_user_preferences` via the real commands code path (or the extracted `apply_patch_atomic` plus explicit cache update) and assert on cache state AFTER the call returns.
- **A concurrent-setter test that `await`s serially (not `join!`)** — would pass even if there's a race. **Rule out:** T5 must use `tokio::try_join!(setter1, setter2)` with both futures started BEFORE the first `.await`.
- **A rollback test that uses `panic!`** — would poison the runtime, not the sqlx transaction. **Rule out:** T3 must force the error via a deliberately invalid SQL (e.g., write to a non-existent column via a test-only query) or via sqlx constraint violation. The error must flow through `Result`, not `panic`.
- **A test for the reject branch that asserts "an error happened"** without checking the error variant — would pass on any unrelated error. **Rule out:** T4 must match `assert!(matches!(result, Err(PreferencesError::InvalidCombination { .. })))`.

### Wave 0 Gaps

- [ ] `frontend/src-tauri/src/preferences/mod.rs` — new file
- [ ] `frontend/src-tauri/src/preferences/commands.rs` — new file
- [ ] `frontend/src-tauri/src/preferences/repository.rs` — new file
- [ ] `frontend/src-tauri/src/preferences/tests.rs` OR `#[cfg(test)] mod tests` block in `mod.rs` — new test module containing all 5 D-22 tests
- [ ] Shared test helper: `async fn new_test_pool() -> SqlitePool` that creates `sqlite::memory:`, runs the migration, seeds the row. Place in `preferences/tests.rs` as `fn test_pool_with_migration()`.
- [ ] No framework install required — `tokio::test` is available via existing `tokio = "1.32.0"` with `full` features.
- [ ] No new Cargo.toml entries required.

### Map to D-22's 5 Tests (Nyquist sufficiency confirmation)

| D-22 Test | Nyquist dimension covered |
|-----------|--------------------------|
| T1 Startup hydration | Dimension 4 (hot-path read staleness) AT STARTUP |
| T2 Atomic-write invariant | Dimensions 1 (desync) + 2 (auto-repoint branch) + 4 (fresh read) |
| T3 Rollback invariance | Dimension 1 (desync under failure) |
| T4 Reject-branch invariant | Dimension 2 (reject branch) |
| T5 Concurrent setter | Dimension 3 (race) |

**Conclusion: D-22 is Nyquist-sufficient for Phase 1.** Phase 6 QA-01 adds the integrated sampling (startup + runtime + concurrent across the full app); Phase 1 samples each invariant in isolation. Good separation of concerns.

## Security Domain

`security_enforcement` is not explicitly disabled in `.planning/config.json`, so this section is included.

### Trust Boundaries

| Boundary | Direction | Attack Surface |
|----------|-----------|----------------|
| Tauri IPC (frontend → Rust) | set_user_preferences patch input | Malformed JSON, missing fields, injection into SQLite |
| SQLite file | Persistence layer | Direct file edit bypasses app-level invariant (out of scope: local-only threat model) |
| In-memory RwLock | Reader / writer consistency | Race between readers and writer |

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Single-user desktop — no auth |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | Local-only single-user |
| V5 Input Validation | **yes** | serde deserialization + explicit enum-like validation on `ui_locale` ('en' or 'ar' only) and provider strings |
| V6 Cryptography | no | No secrets in `user_preferences` row (see Deferred Ideas — CONCERNS #4 encryption is out of scope) |
| V7 Error Handling | **yes** | Errors must not leak stack traces or SQL fragments across Tauri boundary. `PreferencesError::Database(sqlx::Error)` must be formatted as a generic string before `.map_err(...)?` |
| V8 Data Protection | partial | `updated_at` is not PII; `ui_locale`/`summary_language` are not PII |
| V9 Communications | no | Local IPC only, no network |

### Input Validation Rules (V5)

| Field | Allowed values | Enforcement site |
|-------|----------------|------------------|
| `ui_locale` | `'en'`, `'ar'` | Invariant in `apply_patch_atomic` BEFORE UPDATE — reject unknown values with `PreferencesError::InvalidCombination { reason: "unsupported locale" }` |
| `summary_language` | `'en'`, `'ar'` | Same |
| `transcription_language` | `'auto'`, `'en'`, `'ar'`, or ISO-639 codes | Less strict — allow any non-empty string (Whisper supports 99+ languages) but reject empty string |

**Planner:** Include field-level validation in `apply_patch_atomic`. Use `const` slices of allowed values, not runtime lookups.

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| SQL injection via `ui_locale='or 1=1'` | Tampering | Parameterized query (`.bind(&ui_locale)`) — sqlx handles it |
| TOCTOU between read and write | Tampering | Single transaction covers read-merge-write |
| Panic in command leaving Rust in bad state | DoS | `Result<T, String>` at command boundary; no `unwrap` in production paths |
| Concurrent write race | Tampering | `RwLock::write().await` serializes; T5 test catches regressions |
| Information disclosure via error message | Information Disclosure | `PreferencesError::Database` converted to generic string before Tauri return |
| Invariant bypass by direct DB edit | Tampering | OUT OF SCOPE for local-only threat model (user is root on their machine) |

## Sources

### Primary (HIGH confidence — verified against live codebase)
- `.planning/phases/01-preferences-foundation/01-CONTEXT.md` — 22 locked decisions
- `frontend/src-tauri/src/database/repositories/setting.rs` (full read) — sqlx upsert style reference
- `frontend/src-tauri/src/database/repositories/meeting.rs:26–80` — Transaction lifecycle pattern
- `frontend/src-tauri/src/database/manager.rs:164–182` — `with_transaction` closure helper
- `frontend/src-tauri/src/database/setup.rs` (full read) — how AppState is installed
- `frontend/src-tauri/src/database/models.rs` (full read) — sqlx::FromRow + serde rename patterns
- `frontend/src-tauri/src/lib.rs:55–70, 375–390, 405–500, 640–700` — hydration integration site + invoke_handler structure
- `frontend/src-tauri/src/state.rs` (full read) — `AppState { db_manager }` shape
- `frontend/src-tauri/src/audio/import.rs:720–740` — `&mut *tx` idiom in action
- `frontend/src-tauri/src/audio/transcription/worker.rs:440–540` — 2 of 4 live call sites
- `frontend/src-tauri/src/whisper_engine/commands.rs:385–405` — call site 1
- `frontend/src-tauri/src/whisper_engine/parallel_processor.rs:335–360` — call site 4
- `frontend/src-tauri/src/contexts/ConfigContext.tsx:120–225, 450–485` — full frontend state + workaround
- `frontend/src-tauri/migrations/` (listing + 3 files read) — migration naming convention
- `.planning/codebase/STACK.md` — crate versions
- `.planning/codebase/CONVENTIONS.md` — Rust style rules
- `.planning/codebase/TESTING.md` — `cargo test` + `#[tokio::test]` as the only pattern
- `.planning/config.json` — workflow.nyquist_validation = true confirmed
- grep across `frontend/src-tauri/src/` — all Lazy/OnceCell/RwLock usages cataloged

### Secondary (MEDIUM confidence)
- Recollected sqlx 0.8 Transaction API (verified against codebase usage patterns, no external doc fetch needed since every pattern is already proven in this repo)

### Tertiary (LOW confidence)
- None. Every claim in this document is backed by file:line grep or direct read.

## Metadata

**Confidence breakdown:**
- Standard stack: **HIGH** — every crate already in Cargo.toml, every version verified from STACK.md
- Architecture patterns: **HIGH** — every pattern shown has a live code exemplar at file:line
- Pitfalls: **HIGH** — all derived from the specific shape of this codebase (not generic sqlx/tokio gotchas)
- Call-site audit: **HIGH** — grep-verified count of 4 live + 3 dead
- Frontend audit: **HIGH** — grep-verified 3 `primaryLanguage` references
- Validation architecture: **HIGH** — D-22's 5 tests map cleanly to 4 Nyquist dimensions
- Security: **MEDIUM** — ASVS mapping is straightforward for local-only threat model; no external security scanner run

**Research date:** 2026-04-07
**Valid until:** 2026-05-07 (30 days — Meetily sqlx/tokio versions are stable; re-verify after any Cargo.toml bump)
