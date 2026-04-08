-- Migration: Add user_preferences singleton row for UI locale, summary language,
-- and transcription language. Single source of truth replaces lib.rs:68
-- LANGUAGE_PREFERENCE static and localStorage.primaryLanguage.
-- Requirement: PREFS-01
-- Decisions: D-01 (schema), D-02 (filename convention)

CREATE TABLE IF NOT EXISTS user_preferences (
    id                      TEXT PRIMARY KEY DEFAULT '1',
    ui_locale               TEXT NOT NULL DEFAULT 'en',
    summary_language        TEXT NOT NULL DEFAULT 'en',
    transcription_language  TEXT NOT NULL DEFAULT 'auto',
    updated_at              INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

INSERT OR IGNORE INTO user_preferences (id) VALUES ('1');
