---
phase: 01-preferences-foundation
reviewed: 2026-04-08T00:00:00Z
depth: standard
files_reviewed: 11
files_reviewed_list:
  - frontend/src-tauri/migrations/20260407000000_add_user_preferences.sql
  - frontend/src-tauri/src/preferences/mod.rs
  - frontend/src-tauri/src/preferences/repository.rs
  - frontend/src-tauri/src/preferences/commands.rs
  - frontend/src-tauri/src/preferences/tests.rs
  - frontend/src-tauri/src/lib.rs
  - frontend/src-tauri/src/whisper_engine/commands.rs
  - frontend/src-tauri/src/whisper_engine/parallel_processor.rs
  - frontend/src-tauri/src/audio/transcription/worker.rs
  - frontend/src/services/preferencesService.ts
  - frontend/src/contexts/ConfigContext.tsx
findings:
  critical: 0
  warning: 3
  info: 5
  total: 8
status: issues_found
---

# Phase 01: Preferences Foundation — Code Review Report

**Reviewed:** 2026-04-08
**Depth:** standard
**Files Reviewed:** 11
**Status:** issues_found (3 warnings, 5 info)

## Summary

Phase 01 delivers a clean, well-documented single-source-of-truth preferences module. The architectural intent in the phase context is correctly implemented:

- Reject branch (`invariant::check_reject_branch`) fires BEFORE `pool.begin().await` in `repository::apply_patch_atomic` — T-1-03 mitigation verified.
- Cache update in `commands::set_user_preferences` is executed strictly AFTER `apply_patch_atomic` returns `Ok` — T-1-02 mitigation verified.
- `PREFS_CACHE` uses `std::sync::RwLock`; no guard is held across any `.await` point in commands, hydrate, or tests.
- `preferences::read()` is a pure sync clone returning an owned value — safe for the audio hot path (used in 4 migrated call sites).
- Legacy `LANGUAGE_PREFERENCE`, `get_language_preference_internal`, and `set_language_preference` symbols are fully deleted from `src-tauri/src/` (grep over the entire tree returned zero matches).
- All four recording-path call sites (`whisper_engine/commands.rs:396`, `whisper_engine/parallel_processor.rs:344`, `audio/transcription/worker.rs:449`, `audio/transcription/worker.rs:527`) now read from `crate::preferences::read().transcription_language`.
- Tauri commands `get_user_preferences` and `set_user_preferences` are correctly registered in `lib.rs::invoke_handler` (lines 654-655).
- `hydrate_from_db` is called via `tauri::async_runtime::block_on` inside `setup()` AFTER database init and BEFORE command registration completes, matching RESEARCH Pitfall 4 guidance.
- Test isolation via `prefs_test_lock()` (tokio `Mutex` in a `OnceLock`) is correct — every test that touches `PREFS_CACHE` acquires the guard at its first line, and the guard is held for the full test body via RAII. `tokio::sync::Mutex` is the right choice because the guard must be `Send` across `.await` points.
- `ConfigContext.tsx` introduces NO physical-direction Tailwind classes (`ml-*`, `pl-*`, `text-left`, `text-right`, etc.) — the RTL discipline from CLAUDE.md is respected. The file is pure state/effects logic.

No critical security or correctness defects were found. The issues below are all code-quality concerns or minor hardening opportunities; none block merge.

## Warnings

### WR-01: Double-write to `transcript_settings` when `should_auto_repoint` and `patch.provider` are both non-None

**File:** `frontend/src-tauri/src/preferences/repository.rs:84-108`
**Issue:** Steps F and G are not strictly mutually exclusive in the *code*, only in the *documented intent*. If a caller sends `{ ui_locale: 'ar', provider: 'localWhisper' }` while `current.ui_locale == 'en'`, both branches fire:

1. Step F writes `provider='localWhisper', model='large-v3'` (auto-repoint).
2. Step G then writes `provider='localWhisper'` again.

This is functionally idempotent today, but with a patch like `{ ui_locale: 'ar', provider: 'whisperKit' }` — legal per the current `PreferencesError` rules (reject branch only blocks `parakeet`+`ar`) — step G would *overwrite* step F's repoint, silently clobbering the Arabic-safe default (`large-v3` model stays but provider becomes `whisperKit`). Whether that is desired or a latent footgun depends on product intent not captured in the doc comments.

**Fix:** Make the mutual exclusion explicit and document the precedence in code, e.g.:

```rust
// Step F — Auto-repoint (only when step G will NOT run)
if invariant::should_auto_repoint(&patch, &current) && patch.provider.is_none() {
    sqlx::query(
        r#"UPDATE transcript_settings
           SET provider = 'localWhisper', model = 'large-v3'
           WHERE id = '1'"#,
    )
    .execute(&mut *tx).await.map_err(PreferencesError::Database)?;
}

// Step G — Explicit provider (user intent wins, but already invariant-checked)
if let Some(ref provider) = patch.provider {
    sqlx::query(r#"UPDATE transcript_settings SET provider = ? WHERE id = '1'"#)
        .bind(provider)
        .execute(&mut *tx).await.map_err(PreferencesError::Database)?;
}
```

Alternatively, if the product intent is "explicit provider always wins," add a test that pins that behavior and document it in the step comments.

---

### WR-02: Tests use `*PREFS_CACHE.write().unwrap() = merged` but commands path uses `map_err` — test divergence from production ordering

**File:** `frontend/src-tauri/src/preferences/tests.rs:152`
**Issue:** T2 (`atomic_write_auto_repoints_parakeet`) manually mirrors the commands-layer cache update with:

```rust
*PREFS_CACHE.write().expect("PREFS_CACHE poisoned") = merged.clone();
```

This is fine mechanically, but it means the test does NOT actually exercise `commands::set_user_preferences`. If a future refactor moves cache-update logic (e.g., adds event emission, notifies the frontend, or performs a secondary invariant check) the test will silently pass while production drifts. The comment at tests.rs:151 says "mirrors commands.rs ordering" but code drift is the whole risk.

**Fix:** Either (a) add an integration-style test that constructs a `tauri::test::mock_app` and calls the actual `commands::set_user_preferences`, or (b) extract the "apply + cache update" composition into a helper in `mod.rs` that both `commands.rs` and the test can call, guaranteeing a single source of truth for the ordering.

---

### WR-03: `set_user_preferences` silently swallows `PREFS_CACHE` write-lock poison on a committed tx, leaving DB and cache out of sync

**File:** `frontend/src-tauri/src/preferences/commands.rs:47-51`
**Issue:** After `apply_patch_atomic` has successfully committed the SQL transaction, if another thread previously panicked while holding `PREFS_CACHE.write()`, the `.write().map_err(...)` call here returns an error string to the frontend. The **database is now ahead of the cache**: the next `read()` / `get_user_preferences` call will also fail on the poisoned lock (or return stale data from `PREFS_CACHE` if any subsequent code calls `.read().ok()` with a fallback).

Lock poisoning is rare in practice, but the ordering of "commit first, then update cache" means any post-commit failure leaves invariants broken. This is the inverse risk of T-1-02.

**Fix:** Two mitigations worth considering:

1. Treat poison as recoverable by using `.write().unwrap_or_else(|e| e.into_inner())` — `PoisonError::into_inner()` returns the guard anyway, and a write-through to `*guard = merged` atomically restores a valid state. Add a `log::error!` so poisoning doesn't go silent:

   ```rust
   let mut guard = PREFS_CACHE.write().unwrap_or_else(|poisoned| {
       log::error!("PREFS_CACHE poisoned — recovering by overwriting");
       poisoned.into_inner()
   });
   *guard = merged.clone();
   ```

2. Alternatively, re-hydrate from DB on poison (`repository::load(pool).await` + overwrite), which gives a "last known good" state straight from SQLite.

The current `expect`/`map_err` strategy converts a committed-DB/stale-cache divergence into a persistent error state.

## Info

### IN-01: Tests strip `--` comments line-by-line but don't handle trailing-line comments after a statement

**File:** `frontend/src-tauri/src/preferences/tests.rs:58-72`
**Issue:** The comment-stripping logic keeps any line that does not *start* with `--` (after trim). This is correct for the current migration, but a future statement like `CREATE TABLE foo (x INTEGER); -- inline trailing comment` would leave the trailing comment attached to the next split chunk, potentially breaking the loop if the next chunk happens to contain only whitespace + a comment. It's benign today because the migration has no inline trailing comments, but the shim is subtly fragile.

**Fix:** Either add a note in the comment block noting the constraint ("no inline `--` comments after a statement") or use `sqlx::raw_sql` / `Executor::execute` on the full script once sqlx supports it, rather than hand-rolled splitting.

---

### IN-02: `UserPreferences::id` is serialized with `#[serde(skip_deserializing)]` but still serialized to the frontend

**File:** `frontend/src-tauri/src/preferences/mod.rs:36-37`
**Issue:** The `id` field (always `"1"`) is sent to the frontend on every `get_user_preferences` call. The `UserPreferences` TypeScript interface in `preferencesService.ts:15-19` correctly omits it, but the wire payload still contains `{"id":"1",...}`. This is harmless but wastes bytes and leaks an implementation detail.

**Fix:** Add `#[serde(skip_serializing)]` alongside `skip_deserializing`, or use `#[serde(skip)]`:

```rust
#[serde(skip)]
pub id: String,
```

Since `sqlx::FromRow` needs the field populated from the DB, `#[serde(skip)]` is compatible (serde attributes don't affect sqlx).

---

### IN-03: `hydrate_from_db` logs ui/summary/transcription languages but not `updated_at`

**File:** `frontend/src-tauri/src/preferences/mod.rs:131-136`
**Issue:** The startup log line is useful for field debugging but omits `updated_at`, which is the single most diagnostic value for "did the user ever change this, or is it a fresh default?" Would help post-mortem analysis when users report lost-settings bugs.

**Fix:** Include `updated_at` in the log line:

```rust
log::info!(
    "preferences hydrated: ui_locale={}, summary_language={}, transcription_language={}, updated_at={}",
    ui_locale, summary_language, transcription_language, guard.updated_at,
);
```

(Move the log inside the scoped block or capture `updated_at` into the tuple like the other fields.)

---

### IN-04: ConfigContext.tsx still reads legacy `localStorage` keys for `showConfidenceIndicator` and `isAutoSummary`

**File:** `frontend/src/contexts/ConfigContext.tsx:147-162`
**Issue:** Per D-18 and phase context, the "4-way preferences desync" is being removed. `showConfidenceIndicator` and `isAutoSummary` are still sourced from `localStorage` with no corresponding SQLite-backed preference. This is almost certainly intentional for Phase 01 (those two flags are not in `user_preferences`), but the file mixes legacy + new patterns, which invites future confusion.

**Fix:** Add a comment at the top of those two `useState` initializers noting that `showConfidenceIndicator` and `isAutoSummary` are intentionally local-only and scheduled for a future phase (or explicitly out-of-scope per PROJECT.md). Mention the REQ-ID if one exists.

---

### IN-05: `handleSetSelectedLanguage` swallows error and leaves React state stale

**File:** `frontend/src/contexts/ConfigContext.tsx:482-489`
**Issue:** When `setUserPreferences` throws (e.g., InvalidCombination rejection from the backend, or IPC failure), the catch branch only logs — `selectedLanguage` in React state is unchanged, but the caller (a UI component) has no signal that the write failed. The user's click appears to succeed visually (if the UI optimistically updates) or silently no-ops.

**Fix:** Re-throw so callers can surface a toast/error, or set an error state that the UI can render:

```ts
const handleSetSelectedLanguage = useCallback(async (lang: string) => {
  try {
    const updated = await setUserPreferences({ transcriptionLanguage: lang });
    setSelectedLanguage(updated.transcriptionLanguage);
  } catch (err) {
    console.error('[ConfigContext] Failed to save transcription language:', err);
    throw err; // let caller surface the failure
  }
}, []);
```

Note: the `ConfigContextType.setSelectedLanguage` signature is declared as `(lang: string) => void`, so re-throwing would also require widening that type to `Promise<void>` or adding a separate error field to the context.

---

_Reviewed: 2026-04-08_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
