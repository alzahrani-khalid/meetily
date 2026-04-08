//! SQL adapters for `user_preferences` + cross-table atomic transaction.
//!
//! All writes to `user_preferences` and the conditional `transcript_settings`
//! repoint happen inside a single `sqlx::Transaction`. Callers update
//! `PREFS_CACHE` only after this function returns `Ok`. See D-11, D-12.
//!
//! This file does NOT touch `PREFS_CACHE` — cache updates are the caller's
//! responsibility (commands.rs). This separation exists so tests can call
//! `apply_patch_atomic` without going through Tauri state and still exercise
//! the real transaction.

use sqlx::SqlitePool;

use super::{invariant, PreferencesError, UserPreferences, UserPreferencesPatch};

/// Load the singleton `user_preferences` row.
pub async fn load(pool: &SqlitePool) -> Result<UserPreferences, sqlx::Error> {
    sqlx::query_as::<_, UserPreferences>(
        r#"SELECT id, ui_locale, summary_language, transcription_language, updated_at
           FROM user_preferences
           WHERE id = '1'
           LIMIT 1"#,
    )
    .fetch_one(pool)
    .await
}

/// Apply a `UserPreferencesPatch` atomically across `user_preferences` and
/// (conditionally) `transcript_settings`. Returns the fresh post-commit state.
///
/// Execution order (ANY deviation breaks T3 rollback invariance):
///
/// A. Pre-flight load (outside tx) → `current`.
/// B. Merge: `let merged = current.merge(&patch);`
/// C. Pre-flight invariant (BEFORE opening tx per D-09): `check_reject_branch`.
///    This is the T-1-03 mitigation — reject returns BEFORE `pool.begin().await`.
/// D. Open transaction.
/// E. UPDATE `user_preferences` with merged values + `updated_at = strftime('%s','now')`.
/// F. Conditional auto-repoint: if `should_auto_repoint(&patch, &current)` is true,
///    UPDATE `transcript_settings SET provider = 'localWhisper', model = 'large-v3'`.
///    Passes `&current` (pre-merge) — see `invariant::should_auto_repoint` doc.
/// G. Conditional explicit provider write (A1 Option B): if `patch.provider.is_some()`
///    and the reject branch did NOT fire (step C passed), UPDATE
///    `transcript_settings SET provider = ?`. Mutually exclusive with step F by
///    construction: step F requires `patch.ui_locale == Some("ar")` with
///    `current.ui_locale != "ar"`; step G with `patch.provider == Some("parakeet")`
///    while `merged.ui_locale == "ar"` is blocked by step C.
/// H. Commit.
/// I. Read-back (outside tx) for authoritative post-commit state with the
///    SQL-updated `updated_at`.
pub async fn apply_patch_atomic(
    pool: &SqlitePool,
    patch: UserPreferencesPatch,
) -> Result<UserPreferences, PreferencesError> {
    // Step A — Pre-flight load
    let current = load(pool).await.map_err(PreferencesError::Database)?;

    // Step B — Merge
    let merged = current.merge(&patch);

    // Step C — Pre-flight invariant (BEFORE opening tx)
    invariant::check_reject_branch(&patch, &merged)?;

    // Step D — Open transaction
    let mut tx: sqlx::Transaction<'_, sqlx::Sqlite> =
        pool.begin().await.map_err(PreferencesError::Database)?;

    // Step E — UPDATE user_preferences
    sqlx::query(
        r#"UPDATE user_preferences
           SET ui_locale = ?,
               summary_language = ?,
               transcription_language = ?,
               updated_at = strftime('%s','now')
           WHERE id = '1'"#,
    )
    .bind(&merged.ui_locale)
    .bind(&merged.summary_language)
    .bind(&merged.transcription_language)
    .execute(&mut *tx)
    .await
    .map_err(PreferencesError::Database)?;

    // Step F — Conditional auto-repoint (D-08). Pass `&current` (pre-merge).
    if invariant::should_auto_repoint(&patch, &current) {
        sqlx::query(
            r#"UPDATE transcript_settings
               SET provider = 'localWhisper',
                   model = 'large-v3'
               WHERE id = '1'"#,
        )
        .execute(&mut *tx)
        .await
        .map_err(PreferencesError::Database)?;
    }

    // Step G — Conditional explicit provider write (A1 Option B)
    if let Some(ref provider) = patch.provider {
        sqlx::query(
            r#"UPDATE transcript_settings
               SET provider = ?
               WHERE id = '1'"#,
        )
        .bind(provider)
        .execute(&mut *tx)
        .await
        .map_err(PreferencesError::Database)?;
    }

    // Step H — Commit
    tx.commit().await.map_err(PreferencesError::Database)?;

    // Step I — Read-back for authoritative post-commit state
    let fresh = load(pool).await.map_err(PreferencesError::Database)?;
    Ok(fresh)
}
