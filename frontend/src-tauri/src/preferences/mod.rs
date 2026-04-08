//! Single source of truth for user preferences.
//!
//! Replaces the legacy 4-way split between React state / localStorage /
//! `lib.rs::LANGUAGE_PREFERENCE` process-global / SQLite `settings` table.
//!
//! Architecture:
//! - `PREFS_CACHE` is a process-global `Lazy<RwLock<UserPreferences>>` hydrated
//!   from SQLite at startup by `hydrate_from_db` (called in `lib.rs::run` setup).
//! - `read()` returns an owned clone for the sync audio hot path.
//! - Writes go through `commands::set_user_preferences` which delegates to
//!   `repository::apply_patch_atomic` (cross-table tx) and updates the cache
//!   ONLY after `tx.commit()` succeeds. See D-07/D-11/T-1-02.
//!
//! Invariant: `{provider: 'parakeet'}` while `ui_locale == 'ar'` is rejected
//! BEFORE `pool.begin().await` (see `invariant::check_reject_branch`).

pub mod commands;
pub mod repository;

#[cfg(test)]
mod tests;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// User preferences row (singleton, `id = '1'`).
///
/// Serialized to/from the frontend in camelCase via Tauri IPC. The `id` field
/// is never sent from the frontend.
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

impl UserPreferences {
    /// Non-`None` fields from `patch` override fields in `self`. Returns a
    /// new owned `UserPreferences`. Note: `patch.provider` is NOT merged here —
    /// it lives in `transcript_settings`, not `user_preferences`.
    pub fn merge(&self, patch: &UserPreferencesPatch) -> Self {
        let mut out = self.clone();
        if let Some(ref v) = patch.ui_locale {
            out.ui_locale = v.clone();
        }
        if let Some(ref v) = patch.summary_language {
            out.summary_language = v.clone();
        }
        if let Some(ref v) = patch.transcription_language {
            out.transcription_language = v.clone();
        }
        out
    }
}

/// Partial patch for `set_user_preferences`. All fields optional.
///
/// **A1 Option B (2026-04-07):** Includes `provider` which writes to
/// `transcript_settings.provider` (NOT `user_preferences`). The reject branch
/// uses this field to detect `{provider: 'parakeet'}` while Arabic.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferencesPatch {
    #[serde(default)]
    pub ui_locale: Option<String>,
    #[serde(default)]
    pub summary_language: Option<String>,
    #[serde(default)]
    pub transcription_language: Option<String>,
    /// A1 Option B: this writes to `transcript_settings.provider` inside the
    /// same sqlx transaction as the `user_preferences` UPDATE. The reject
    /// branch (see `invariant::check_reject_branch`) reads this field.
    #[serde(default)]
    pub provider: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PreferencesError {
    #[error("invalid combination: {reason}")]
    InvalidCombination { reason: String },
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Process-global cache of the current preferences. Populated by
/// `hydrate_from_db` at startup, updated by `commands::set_user_preferences`
/// ONLY after `tx.commit()` succeeds (T-1-02 mitigation).
pub(crate) static PREFS_CACHE: Lazy<RwLock<UserPreferences>> = Lazy::new(|| {
    RwLock::new(UserPreferences {
        id: "1".to_string(),
        ui_locale: "en".to_string(),
        summary_language: "en".to_string(),
        transcription_language: "auto".to_string(),
        updated_at: 0,
    })
});

/// Load `user_preferences` row from SQLite and populate `PREFS_CACHE`.
///
/// Called once at startup inside `lib.rs::run` setup closure (via
/// `tauri::async_runtime::block_on`) BEFORE command registration completes,
/// so the first `get_user_preferences` invocation always resolves against a
/// populated cache. Uses `block_on` (not `spawn`) per RESEARCH Pitfall 4.
pub async fn hydrate_from_db(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let loaded = repository::load(pool).await?;
    {
        let mut guard = PREFS_CACHE.write().await;
        *guard = loaded;
    }
    let prefs = PREFS_CACHE.read().await;
    log::info!(
        "preferences hydrated from db: ui_locale={}, summary_language={}, transcription_language={}",
        prefs.ui_locale,
        prefs.summary_language,
        prefs.transcription_language
    );
    Ok(())
}

/// Synchronous reader for the audio hot-path. Returns an owned clone;
/// callers MUST NOT hold any guard across `.await` points.
///
/// Uses `blocking_read()` which is safe here because:
/// 1. Writes happen only from `set_user_preferences` (a Tauri command handler,
///    not the audio hot path), so contention is rare.
/// 2. Returning an owned clone means the read guard is dropped before return.
pub fn read() -> UserPreferences {
    PREFS_CACHE.blocking_read().clone()
}

/// Invariant checks for the Parakeet-ban hybrid rule.
///
/// D-10 direction-of-change rule: reads the PATCH (user intent), not merely
/// the merged state. The reject branch fires when the user DIRECTLY asks for
/// Parakeet while Arabic is active. The auto-repoint branch fires when the
/// user is FLIPPING ui_locale to 'ar' while Parakeet is currently selected.
pub(crate) mod invariant {
    use super::{PreferencesError, UserPreferences, UserPreferencesPatch};

    /// Reject branch (D-09): `{provider: 'parakeet'}` while `merged.ui_locale == 'ar'`.
    ///
    /// Reads the patch (user intent), not merely the merged state. The reject
    /// branch fires when the user is DIRECTLY asking for Parakeet while
    /// Arabic is active. Called BEFORE `pool.begin().await` so the transaction
    /// never opens on reject (T-1-03 mitigation).
    pub fn check_reject_branch(
        patch: &UserPreferencesPatch,
        merged: &UserPreferences,
    ) -> Result<(), PreferencesError> {
        if patch.provider.as_deref() == Some("parakeet") && merged.ui_locale == "ar" {
            return Err(PreferencesError::InvalidCombination {
                reason: "Parakeet provider is not supported when ui_locale == 'ar'; use localWhisper + large-v3"
                    .to_string(),
            });
        }
        Ok(())
    }

    /// D-08 auto-repoint branch: fires ONLY when the user's patch is FLIPPING
    /// ui_locale from non-'ar' to 'ar'. A no-op re-set (patch.ui_locale ==
    /// Some('ar') while already ar) does NOT trigger the repoint — the branch
    /// is guarded against vacuous firings. The actual SQL UPDATE to
    /// transcript_settings.provider is idempotent if the provider is already
    /// localWhisper, but we still avoid the unnecessary write.
    ///
    /// Takes `current` (pre-merge), NOT `merged` (post-merge). Reason: D-08
    /// says the branch fires when the merge result *flips* ui_locale to 'ar'.
    /// Using `merged.ui_locale == "ar"` is vacuously true whenever
    /// `patch.ui_locale == Some("ar")` (because merge already applied the
    /// patch), so the branch would fire on every re-set.
    pub fn should_auto_repoint(patch: &UserPreferencesPatch, current: &UserPreferences) -> bool {
        patch.ui_locale.as_deref() == Some("ar") && current.ui_locale != "ar"
    }
}
