---
phase: 06-rust-strings-qa-release-hardening
plan: 01
subsystem: rust-i18n
tags: [tray, notifications, bilingual, locale-event, UI-07]
dependency_graph:
  requires: [preferences-foundation, i18n-framework]
  provides: [bilingual-tray-menu, bilingual-notifications, locale-changed-event]
  affects: [tray.rs, notifications/types.rs, preferences/commands.rs, lib.rs]
tech_stack:
  added: []
  patterns: [bilingual-string-constants, locale-changed-event-bus, post-hydration-rebuild]
key_files:
  created: []
  modified:
    - frontend/src-tauri/src/tray.rs
    - frontend/src-tauri/src/notifications/types.rs
    - frontend/src-tauri/src/preferences/commands.rs
    - frontend/src-tauri/src/lib.rs
decisions:
  - "Used struct with named fields for TrayStrings/NotificationStrings (type safety over tuple arrays)"
  - "Notification helpers read locale internally via preferences::read() -- no caller signature changes"
  - "Locale-changed event guarded by old!=new comparison (T-06-03 mitigation)"
metrics:
  duration_seconds: 273
  completed: 2026-04-15T16:42:15Z
  tasks_completed: 2
  tasks_total: 2
  files_modified: 4
---

# Phase 06 Plan 01: Tray + Notification Bilingual String Constants + Locale-Changed Event Wiring Summary

Bilingual EN/AR Rust string constants for 13 tray menu items and 12 notification strings, with locale-changed event pipeline from preferences write through tray rebuild on language switch.

## Task Results

| Task | Name | Commit | Key Changes |
|------|------|--------|-------------|
| 1 | Add bilingual string constants to tray.rs and notifications/types.rs | 19ef0bd | TrayStrings/NotificationStrings structs + EN/AR constants + locale-aware build_menu and notification helpers |
| 2 | Wire locale-changed event from preferences to tray rebuild | ec03439 | AppHandle param on set_user_preferences + locale-changed emission + lib.rs listener + post-hydration rebuild |

## Implementation Details

### Tray Menu (tray.rs)
- Added `TrayStrings` struct with 13 named `&'static str` fields
- Added `TRAY_EN` and `TRAY_AR` constants with exact strings from UI-SPEC copywriting table
- Added `tray_strings(locale)` resolver function
- Modified `build_menu()` to read `preferences::read().ui_locale` and use resolved strings
- Emoji prefixes (D-03) remain hardcoded as language-neutral

### Notifications (notifications/types.rs)
- Added `NotificationStrings` struct with 12 named `&'static str` fields
- Added `NOTIF_EN` and `NOTIF_AR` constants
- Added `notif_strings(locale)` resolver (crate-private)
- All 8 helper constructors now read locale internally via `crate::preferences::read().ui_locale`
- No caller signature changes required -- backwards compatible

### Locale-Changed Event (preferences/commands.rs + lib.rs)
- `set_user_preferences` now accepts `AppHandle<R>` (Tauri infers generic automatically)
- Captures `old_locale` before `apply_patch_atomic`, emits `locale-changed` only when changed
- `lib.rs` registers `listen("locale-changed")` handler that calls `tray::update_tray_menu`
- Post-hydration `tray::update_tray_menu` call ensures correct locale on first startup

## Decisions Made

1. **Struct vs tuple for string constants**: Used named-field structs (`TrayStrings`, `NotificationStrings`) for type safety and readability over tuple arrays.
2. **Notification locale source**: Helpers read `preferences::read().ui_locale` internally rather than accepting a locale parameter -- fewer caller changes, single source of truth.
3. **Event guard**: `locale-changed` only fires when `merged.ui_locale != old_locale` (T-06-03 DoS mitigation).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Missing sidecar binary placeholder**
- **Found during:** Task 1 verification
- **Issue:** `cargo check` failed with `resource path binaries/llama-helper-aarch64-apple-darwin doesn't exist` (pre-existing build issue)
- **Fix:** Created placeholder binary file to unblock cargo check
- **Files modified:** frontend/src-tauri/binaries/llama-helper-aarch64-apple-darwin (placeholder, not committed)

## Known Stubs

None -- all string constants are fully populated with production Arabic and English text from UI-SPEC.

## Self-Check: PASSED

- All 4 modified files exist on disk
- Commit 19ef0bd (Task 1) verified in git log
- Commit ec03439 (Task 2) verified in git log
- All acceptance criteria grep checks return expected counts
- cargo check passes with zero errors
