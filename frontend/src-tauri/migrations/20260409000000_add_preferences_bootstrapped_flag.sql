-- Migration: Add `bootstrapped` flag to user_preferences so the Phase 2
-- first-run navigator.language detector runs exactly once per install.
-- Requirement: UI-01 (first-run detection with en fallback)
-- Decision: D-01 (column, not re-detect heuristic) + D-03 (lives in Phase 1 preferences module)

ALTER TABLE user_preferences
    ADD COLUMN bootstrapped INTEGER NOT NULL DEFAULT 0;
