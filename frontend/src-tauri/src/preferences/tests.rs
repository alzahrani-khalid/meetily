//! Targeted Phase 1 integration tests (T1..T5) for the preferences module.
//!
//! These are the Nyquist-sufficient minimum described in VALIDATION.md § Nyquist
//! Sufficiency. Phase 6 QA-01 will expand the regression suite.
//!
//! Anti-Sampling Rules (VALIDATION.md):
//! 1. Real in-memory SqlitePool, no mocks
//! 2. Exercise commands path ordering (post-commit cache update)
//! 3. Concurrent tests use try_join! with futures built before any .await
//! 4. Rollback forced via invalid SQL (DROP TABLE), not panic!
//! 5. Error assertions via variant match, not generic is_err()
//!
//! Test isolation: `PREFS_CACHE` is a process-global `Lazy<RwLock<_>>`, so
//! every test that calls `hydrate_from_db` or reads/writes the cache MUST
//! acquire `prefs_test_lock()` first. Without this, cargo test's default
//! parallelism races hydrate_from_db across pools and breaks cache
//! assertions. Using `tokio::sync::Mutex` (not `std::sync::Mutex`) so the
//! guard is `Send` across `.await` boundaries inside `#[tokio::test]`.

use super::*;
use sqlx::SqlitePool;
use std::sync::OnceLock;
use tokio::sync::{Mutex, MutexGuard};

/// Serializes access to the process-global `PREFS_CACHE` across tests.
/// Every test that hydrates or reads the cache acquires this at its start.
fn prefs_test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Acquire the cross-test PREFS_CACHE mutex. Hold the returned guard for the
/// full duration of the test body.
async fn lock_prefs_cache() -> MutexGuard<'static, ()> {
    prefs_test_lock().lock().await
}

/// Builds an in-memory SQLite pool with:
/// (1) The real Phase 1 migration executed via include_str! — no hand-written duplicate.
/// (2) A minimal `transcript_settings` table + singleton row, sufficient for T2/T3/T4
///     to exercise the cross-table transaction without loading the full initial schema.
///
/// Nyquist Anti-Sampling Rule #1: real SqlitePool, no mocks.
pub(crate) async fn test_pool_with_migration() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to open sqlite::memory:");

    // Real Phase 1 migration — loaded from disk at compile time
    let migration_sql = include_str!(
        "../../migrations/20260407000000_add_user_preferences.sql"
    );
    // Phase 2 migration — adds bootstrapped column
    let migration2_sql = include_str!(
        "../../migrations/20260409000000_add_preferences_bootstrapped_flag.sql"
    );
    // Strip line-level `--` comments before splitting on ';'. Comments at the
    // top of the file would otherwise be lumped into the first statement chunk
    // (causing the CREATE TABLE to be silently skipped by the `starts_with("--")`
    // guard). This keeps in-statement constructs like `strftime('%s','now')`
    // intact because we only strip leading-line comments.
    let stripped: String = migration_sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    // Split on ';' for multi-statement execution; sqlx::query accepts one stmt at a time
    for stmt in stripped.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("migration stmt failed: {} — err: {}", trimmed, e));
        }
    }

    // Run Phase 2 migration (bootstrapped column)
    let stripped2: String = migration2_sql
        .lines()
        .filter(|line| !line.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    for stmt in stripped2.split(';') {
        let trimmed = stmt.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(&pool)
                .await
                .unwrap_or_else(|e| panic!("migration2 stmt failed: {} — err: {}", trimmed, e));
        }
    }

    // Minimal transcript_settings shim for T2/T3/T4
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS transcript_settings (
            id TEXT PRIMARY KEY DEFAULT '1',
            provider TEXT NOT NULL DEFAULT 'localWhisper',
            model TEXT NOT NULL DEFAULT 'large-v3'
        )"#,
    )
    .execute(&pool)
    .await
    .expect("transcript_settings create failed");

    sqlx::query("INSERT OR IGNORE INTO transcript_settings (id) VALUES ('1')")
        .execute(&pool)
        .await
        .expect("transcript_settings seed failed");

    pool
}

// =============================================================================
// T1 — Hydration reflects seeded row (1-01-07, binds PREFS-01)
// =============================================================================

#[tokio::test]
async fn hydration_reflects_seeded_row() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;

    // Seed arabic state BEFORE hydration — the test simulates "user previously
    // set 'ar', now the process starts up and must reflect it"
    sqlx::query("UPDATE user_preferences SET ui_locale = 'ar' WHERE id = '1'")
        .execute(&pool)
        .await
        .expect("seed update failed");

    // Exercise the real hydrate_from_db path
    hydrate_from_db(&pool)
        .await
        .expect("hydrate_from_db failed");

    // Exercise the real read() path — Anti-Sampling Rule #2: go through the
    // public API, not direct PREFS_CACHE inspection
    let prefs = read();
    assert_eq!(prefs.ui_locale, "ar", "hydration did not reflect seeded row");
}

// =============================================================================
// T2 — Atomic auto-repoint on ui_locale→ar flips parakeet → localWhisper+large-v3
//      (1-01-08, binds PREFS-02 + T-1-02)
// =============================================================================

#[tokio::test]
async fn atomic_write_auto_repoints_parakeet() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Setup: transcript_settings currently has parakeet
    sqlx::query(
        "UPDATE transcript_settings SET provider = 'parakeet', model = 'parakeet-tdt-0.6b' WHERE id = '1'",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Act: user flips ui_locale to 'ar' — should auto-repoint transcript_settings
    let patch = UserPreferencesPatch {
        ui_locale: Some("ar".to_string()),
        summary_language: None,
        transcription_language: None,
        provider: None,
        ..Default::default()
    };
    let merged = repository::apply_patch_atomic(&pool, patch)
        .await
        .expect("apply_patch_atomic should succeed for auto-repoint");

    // Post-commit cache update (mirrors commands.rs ordering — AFTER apply_patch_atomic Ok)
    *PREFS_CACHE.write().expect("PREFS_CACHE poisoned") = merged.clone();

    // Assert both rows updated atomically
    assert_eq!(merged.ui_locale, "ar");

    let row: (String, String) =
        sqlx::query_as("SELECT provider, model FROM transcript_settings WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "localWhisper", "auto-repoint did not set provider");
    assert_eq!(row.1, "large-v3", "auto-repoint did not set model");

    // Cache reflects committed state
    assert_eq!(read().ui_locale, "ar");
}

// =============================================================================
// T3 — Rollback invariance: forced tx failure leaves cache + row unchanged
//      (1-01-09, binds PREFS-02 + T-1-02)
// =============================================================================

#[tokio::test]
async fn rollback_leaves_cache_and_row_unchanged() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Initial state: en
    assert_eq!(read().ui_locale, "en");

    // Force failure: drop transcript_settings so the auto-repoint UPDATE inside
    // the tx fails with "no such table". This exercises the REAL error path —
    // no panic!, no mock — per Anti-Sampling Rule #4.
    sqlx::query("DROP TABLE transcript_settings")
        .execute(&pool)
        .await
        .unwrap();

    // Act: request ui_locale='ar' → will trigger auto-repoint branch → will try
    // to UPDATE transcript_settings (now missing) → tx should rollback
    let patch = UserPreferencesPatch {
        ui_locale: Some("ar".to_string()),
        summary_language: None,
        transcription_language: None,
        provider: None,
        ..Default::default()
    };
    let result = repository::apply_patch_atomic(&pool, patch).await;

    assert!(
        matches!(result, Err(PreferencesError::Database(_))),
        "expected Database error from dropped table, got: {:?}",
        result
    );

    // Assert: user_preferences row unchanged (rollback)
    let (ui_locale,): (String,) =
        sqlx::query_as("SELECT ui_locale FROM user_preferences WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(ui_locale, "en", "rollback did not preserve user_preferences row");

    // Assert: PREFS_CACHE unchanged — because the commands.rs layer updates the
    // cache ONLY after apply_patch_atomic Ok, and here it returned Err. In this
    // test the caller (us) mirrors that discipline: we do NOT update the cache
    // on Err.
    assert_eq!(read().ui_locale, "en", "cache mutated despite rollback");
}

// =============================================================================
// T4 — REAL reject branch: {provider:'parakeet'} while ar → InvalidCombination
//      (1-01-10, binds PREFS-02 + T-1-03, A1 Option B)
// =============================================================================

#[tokio::test]
async fn reject_parakeet_while_arabic() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;

    // Seed arabic state
    sqlx::query("UPDATE user_preferences SET ui_locale = 'ar' WHERE id = '1'")
        .execute(&pool)
        .await
        .unwrap();
    hydrate_from_db(&pool).await.expect("hydrate");

    // Snapshot pre-call transcript_settings to prove no tx side effects
    let (provider_before,): (String,) =
        sqlx::query_as("SELECT provider FROM transcript_settings WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();

    // Act: patch directly requests parakeet
    let patch = UserPreferencesPatch {
        ui_locale: None,
        summary_language: None,
        transcription_language: None,
        provider: Some("parakeet".to_string()),
        ..Default::default()
    };
    let result = repository::apply_patch_atomic(&pool, patch).await;

    // VARIANT MATCH (Anti-Sampling Rule #5) — NOT generic is_err()
    assert!(
        matches!(result, Err(PreferencesError::InvalidCombination { .. })),
        "expected InvalidCombination variant, got: {:?}",
        result
    );

    // Prove tx never opened: transcript_settings.provider unchanged
    let (provider_after,): (String,) =
        sqlx::query_as("SELECT provider FROM transcript_settings WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(provider_before, provider_after, "reject branch had side effects");
    assert_ne!(provider_after, "parakeet", "reject branch let parakeet through");
}

// =============================================================================
// T5 — Concurrent setters: try_join! two writers, no partial state
//      (1-01-11, binds PREFS-02 + T-1-04)
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_setters_serialize() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let patch1 = UserPreferencesPatch {
        ui_locale: None,
        summary_language: None,
        transcription_language: Some("en".to_string()),
        provider: None,
        ..Default::default()
    };
    let patch2 = UserPreferencesPatch {
        ui_locale: None,
        summary_language: None,
        transcription_language: Some("fr".to_string()),
        provider: None,
        ..Default::default()
    };

    // CRITICAL (Anti-Sampling Rule #3): both futures built before any .await.
    // Move pools into the futures so they have 'static lifetimes.
    let fut1 = async move { repository::apply_patch_atomic(&pool1, patch1).await };
    let fut2 = async move { repository::apply_patch_atomic(&pool2, patch2).await };

    // Bound: deadlock → test hangs → panic via timeout wrapper
    let joined = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        async { tokio::try_join!(fut1, fut2) },
    )
    .await
    .expect("concurrent setters deadlocked (>2s)");

    let (r1, r2) = joined.expect("one or both setters errored");

    // Both setters succeeded; final state must be one of the two inputs, never a hybrid
    let (final_lang,): (String,) = sqlx::query_as(
        "SELECT transcription_language FROM user_preferences WHERE id = '1'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        final_lang == "en" || final_lang == "fr",
        "final state is a hybrid/default: got {}, r1.transcription_language={}, r2.transcription_language={}",
        final_lang, r1.transcription_language, r2.transcription_language
    );
    assert_ne!(
        final_lang, "auto",
        "final state is the pre-call default — neither write landed"
    );
}

// =============================================================================
// T6 — QA-01: Startup with Arabic locale hydrates correctly
//      (6-02, binds QA-01)
// =============================================================================

#[tokio::test]
async fn qa01_startup_ar_locale_hydrates_correctly() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;

    // Seed Arabic locale in DB before hydration — simulates returning user
    sqlx::query("UPDATE user_preferences SET ui_locale = 'ar' WHERE id = '1'")
        .execute(&pool)
        .await
        .expect("seed ar locale");

    hydrate_from_db(&pool).await.expect("hydrate");
    let prefs = read();
    assert_eq!(prefs.ui_locale, "ar", "startup hydration must reflect seeded ar locale");
}

// =============================================================================
// T7 — QA-01: Runtime locale switch visible in cache
//      (6-02, binds QA-01)
// =============================================================================

#[tokio::test]
async fn qa01_runtime_locale_switch_visible_in_cache() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Default is "en", switch to "ar"
    let patch = UserPreferencesPatch {
        ui_locale: Some("ar".to_string()),
        ..Default::default()
    };
    let merged = repository::apply_patch_atomic(&pool, patch)
        .await
        .expect("apply_patch_atomic");

    // Mirror commands.rs cache update ordering
    *PREFS_CACHE.write().expect("poisoned") = merged.clone();
    let prefs = read();
    assert_eq!(prefs.ui_locale, "ar", "runtime switch must be visible via read()");
}

// =============================================================================
// T8 — QA-01: Concurrent locale setters produce no partial state
//      (6-02, binds QA-01)
// =============================================================================

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn qa01_concurrent_locale_setters_no_partial_state() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Build two futures before any .await (Anti-Sampling Rule #3)
    let pool_a = pool.clone();
    let pool_b = pool.clone();
    let fut_a = async move {
        let patch = UserPreferencesPatch {
            ui_locale: Some("ar".to_string()),
            ..Default::default()
        };
        repository::apply_patch_atomic(&pool_a, patch).await
    };
    let fut_b = async move {
        let patch = UserPreferencesPatch {
            ui_locale: Some("en".to_string()),
            ..Default::default()
        };
        repository::apply_patch_atomic(&pool_b, patch).await
    };

    let joined = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        async { tokio::try_join!(fut_a, fut_b) },
    )
    .await
    .expect("concurrent locale setters deadlocked (>2s)");

    let (_res_a, _res_b) = joined.expect("one or both setters errored");

    // Final DB state must be one of the two inputs, never a hybrid
    let row: (String,) =
        sqlx::query_as("SELECT ui_locale FROM user_preferences WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .expect("fetch");
    assert!(
        row.0 == "ar" || row.0 == "en",
        "final state must be ar or en, got: {}",
        row.0
    );
}

// =============================================================================
// T9 — QA-02: en->ar switch auto-repoints parakeet provider
//      (6-02, binds QA-02)
// =============================================================================

#[tokio::test]
async fn qa02_en_to_ar_switch_repoints_parakeet() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Seed parakeet provider while locale is still "en"
    sqlx::query(
        "UPDATE transcript_settings SET provider = 'parakeet', model = 'parakeet-tdt-0.6b' WHERE id = '1'",
    )
    .execute(&pool)
    .await
    .expect("seed parakeet");

    // Switch locale to "ar" — should auto-repoint
    let patch = UserPreferencesPatch {
        ui_locale: Some("ar".to_string()),
        ..Default::default()
    };
    let merged = repository::apply_patch_atomic(&pool, patch)
        .await
        .expect("should succeed with auto-repoint");

    *PREFS_CACHE.write().expect("poisoned") = merged.clone();

    // Assert variant match (Anti-Sampling Rule #5)
    assert_eq!(merged.ui_locale, "ar");

    // Provider must NOT be parakeet after auto-repoint
    let row: (String,) =
        sqlx::query_as("SELECT provider FROM transcript_settings WHERE id = '1'")
            .fetch_one(&pool)
            .await
            .expect("fetch");
    assert_ne!(row.0, "parakeet", "parakeet must be auto-repointed on ar switch");
}

// =============================================================================
// T10 — QA-02: Direct parakeet+ar patch rejected
//       (6-02, binds QA-02)
// =============================================================================

#[tokio::test]
async fn qa02_direct_parakeet_ar_patch_rejected() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    hydrate_from_db(&pool).await.expect("hydrate");

    // Set locale to ar first
    let setup = UserPreferencesPatch {
        ui_locale: Some("ar".to_string()),
        ..Default::default()
    };
    let merged = repository::apply_patch_atomic(&pool, setup)
        .await
        .expect("setup");
    *PREFS_CACHE.write().expect("poisoned") = merged;

    // Now try to set provider to parakeet while ar
    let patch = UserPreferencesPatch {
        provider: Some("parakeet".to_string()),
        ..Default::default()
    };
    let result = repository::apply_patch_atomic(&pool, patch).await;

    // VARIANT MATCH (Anti-Sampling Rule #5) — NOT generic is_err()
    assert!(
        matches!(result, Err(PreferencesError::InvalidCombination { .. })),
        "parakeet + ar must be rejected as InvalidCombination, got: {:?}",
        result
    );
}
