//! Tauri command surface for user preferences.
//!
//! `set_user_preferences` delegates to `repository::apply_patch_atomic` for
//! the cross-table transaction, then updates `PREFS_CACHE` ONLY after the
//! repository returns `Ok` (i.e., `tx.commit()` already succeeded). This
//! ordering is the T-1-02 mitigation — inverting it breaks T3 rollback
//! invariance.

use tauri::State;

use crate::state::AppState;

use super::{PreferencesError, UserPreferences, UserPreferencesPatch, PREFS_CACHE};

#[tauri::command]
pub async fn get_user_preferences() -> Result<UserPreferences, String> {
    Ok(PREFS_CACHE
        .read()
        .map_err(|_| "PREFS_CACHE poisoned".to_string())?
        .clone())
}

#[tauri::command]
pub async fn set_user_preferences(
    patch: UserPreferencesPatch,
    state: State<'_, AppState>,
) -> Result<UserPreferences, String> {
    let pool = state.db_manager.pool();

    // Runs apply_patch_atomic: pre-flight load → merge → invariant pre-flight
    // → begin → UPDATE(s) → commit.
    let merged = super::repository::apply_patch_atomic(pool, patch)
        .await
        .map_err(|e| match e {
            PreferencesError::InvalidCombination { reason } => reason,
            PreferencesError::Database(err) => format!("Database error: {}", err),
        })?;

    // T-1-02 mitigation: update cache ONLY after apply_patch_atomic returned
    // Ok, which means tx.commit() already succeeded. If this order is
    // inverted, T3 (rollback invariance) will fail because on tx failure the
    // cache must be untouched.
    //
    // Uses std::sync::RwLock (not tokio) per D-04 — the guard is acquired
    // and dropped inline, NEVER held across an .await boundary.
    {
        let mut guard = PREFS_CACHE
            .write()
            .map_err(|_| "PREFS_CACHE poisoned".to_string())?;
        *guard = merged.clone();
    }

    Ok(merged)
}
