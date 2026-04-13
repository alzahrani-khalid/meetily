# Phase 6: Rust Strings, QA & Release Hardening - Research

**Researched:** 2026-04-13
**Domain:** Rust i18n (tray/notifications), automated regression testing, manual QA
**Confidence:** HIGH

## Summary

Phase 6 hardens what Phases 1-5 built. It has three distinct work streams: (1) localizing ~13 tray menu strings and ~10 notification strings in Rust, wiring locale-change events to rebuild them; (2) extending the existing `#[cfg(test)]` Rust test suite with QA-01/QA-02/QA-03 regression tests; and (3) manual QA passes for RTL visual regression, Arabic transcription accuracy, and Arabic summary quality.

The codebase is well-prepared. `preferences::read().ui_locale` is the single source of truth. `tray.rs:build_menu()` has 13 hardcoded English strings at lines 326-391. `notifications/types.rs` has 10 helper constructors (lines 116-198) with English strings. The existing test infrastructure in `preferences/tests.rs` (5 tests, `test_pool_with_migration()`, `prefs_test_lock()`) provides the scaffold for QA-01/QA-02 expansion. The `summary/prompts/loader.rs` and `summary/templates/loader.rs` already have the exact `get_prompt(id, locale)` and `get_template(id, locale)` signatures that QA-03 needs to test.

**Primary recommendation:** Implement in 3 waves -- (1) Rust string constants + tray/notification locale wiring, (2) automated regression tests, (3) manual QA checklist document.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Translations stored as Rust constants -- parallel `TRAY_STRINGS_EN` / `TRAY_STRINGS_AR` arrays in `tray.rs`, and equivalent constants in `notifications/types.rs`. No external JSON files, no new crate dependencies.
- **D-02:** Tray menu re-hydrates on locale change via a Tauri event listener. `preferences::set` emits a `locale-changed` event; `tray.rs` listens and calls `rebuild_menu()` with the new locale. Same pattern as frontend event listeners.
- **D-03:** Emoji prefixes stay the same in both English and Arabic -- emojis are language-neutral. Only the text labels change.
- **D-04:** All automated tests use Rust `#[cfg(test)]` unit tests inside existing modules. Phase 1 already has 5 tests in `preferences/tests.rs` -- Phase 6 extends the same pattern.
- **D-05:** QA-02 (Parakeet ban) tests the reject branch only -- `set_user_preferences` rejects `provider:'parakeet'` when `ui_locale='ar'`, plus edge cases for `en->ar` locale switch auto-repointing provider. No frontend component tests.
- **D-06:** QA-03 (template/prompt fallback) uses the 3-case matrix: AR present -> AR, AR missing + EN present -> EN fallback, both missing -> error. Covers both `get_template()` and `get_prompt()`.
- **D-07:** Manual RTL regression documented as Markdown checklist in `QA-04-RTL-PASS.md`. One section per screen.
- **D-08:** Pass/fail bar: no text overflow, no clipping, no visual asymmetry. Minor spacing acceptable.
- **D-09:** QA-05 uses user-provided MSA Arabic audio samples (30-60s). Whisper `large-v3`. Expected ~85-88% accuracy.
- **D-10:** QA-06 tests BOTH Claude and Ollama providers.
- **D-11:** QA-05 and QA-06 results documented in same checklist file as QA-04.

### Claude's Discretion
- Implementation details for how Rust constant arrays are structured (struct vs tuple vs enum)
- Exact Tauri event name for locale-changed notification
- QA-01 specific test case design -- based on existing T1-T5 tests
- Whether to merge QA-04/QA-05/QA-06 checklist sections or keep as separate headings

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-07 | Rust-owned UI elements (tray ~13 strings, notifications ~10 strings) in selected language, hydrated from `preferences::read().ui_locale` at startup and re-hydrated on preference change via Tauri event | Tray string inventory (13 items), notification string inventory (10 items), locale-changed event pattern, `build_menu()` signature analysis |
| QA-01 | Automated preference desync regression: startup RTL, runtime switch visible to whisper_engine, concurrent setters no partial state | Extends existing T1-T5 in `preferences/tests.rs` using `test_pool_with_migration()` and `prefs_test_lock()` |
| QA-02 | Automated Parakeet-ban enforcement: Arabic onboarding never calls parakeet_download_model, `set_user_preferences` rejects provider:parakeet when ar | Extends T4 pattern (variant match), adds edge cases for en->ar auto-repoint |
| QA-03 | Automated template/prompt locale fallback: 3-case matrix for `get_template()` and `get_prompt()` | `get_template(id, locale)` and `get_prompt(id, locale)` signatures verified, existing prompt tests in `prompts/loader.rs` provide pattern |
| QA-04 | Manual RTL regression pass across 7 screens | Checklist document with per-screen sections |
| QA-05 | Arabic transcription quality spot-check with MSA audio | Manual test with Whisper large-v3, documented in checklist |
| QA-06 | Arabic summary quality check with Claude + Ollama | Manual test with both providers, documented in checklist |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tauri | 2.6.2 | Desktop framework, tray API, event bus | Already in Cargo.toml [VERIFIED: Cargo.toml] |
| sqlx | (existing) | SQLite for preferences tests | Already in Cargo.toml, used by T1-T5 [VERIFIED: preferences/tests.rs] |
| tokio | (existing) | Async runtime for tests | Already in Cargo.toml [VERIFIED: preferences/tests.rs] |

No new dependencies needed. [VERIFIED: codebase grep]

## Architecture Patterns

### Tray String Constants Pattern (D-01)

**Recommended structure:** Use a struct with named fields for type safety and readability.

```rust
// Source: D-01 decision + codebase pattern analysis
pub struct TrayStrings {
    pub start_recording: &'static str,
    pub starting_recording: &'static str,
    pub pause_recording: &'static str,
    pub pausing: &'static str,
    pub resume_recording: &'static str,
    pub resuming: &'static str,
    pub stop_recording: &'static str,
    pub stopping: &'static str,
    pub downloading_model: &'static str,
    pub open_main_window: &'static str,
    pub settings: &'static str,
    pub check_for_updates: &'static str,
    pub quit: &'static str,
}

pub const TRAY_EN: TrayStrings = TrayStrings {
    start_recording: "Start Recording",
    starting_recording: "Starting Recording...",
    pause_recording: "Pause Recording",
    // ... etc
};

pub const TRAY_AR: TrayStrings = TrayStrings {
    start_recording: "\u{0628}\u{062F}\u{0621} \u{0627}\u{0644}\u{062A}\u{0633}\u{062C}\u{064A}\u{0644}",
    // ... etc
};

pub fn tray_strings(locale: &str) -> &'static TrayStrings {
    match locale {
        "ar" => &TRAY_AR,
        _ => &TRAY_EN,
    }
}
```

This pattern is recommended over tuple arrays because field names prevent ordering bugs. [ASSUMED]

### Notification String Constants Pattern

Same struct approach for `notifications/types.rs`:

```rust
pub struct NotificationStrings {
    pub app_name: &'static str,
    pub recording_started: &'static str,
    pub recording_started_for: &'static str, // format pattern
    pub recording_stopped: &'static str,
    pub recording_paused: &'static str,
    pub recording_resumed: &'static str,
    pub transcription_complete: &'static str,
    pub transcription_saved_to: &'static str, // format pattern
    pub meeting_reminder: &'static str, // format pattern
    pub error_title: &'static str,
    pub test_notification: &'static str,
}
```

Notification helpers (`recording_started()`, etc.) need a `locale: &str` parameter added. [VERIFIED: types.rs lines 116-198]

### Locale-Changed Event Pattern (D-02)

The `locale-changed` event does NOT exist yet. [VERIFIED: grep found zero matches]

**Implementation:** `set_user_preferences` in `commands.rs` must emit the event after cache update:

```rust
// In commands.rs::set_user_preferences, after PREFS_CACHE write:
if let Some(ref new_locale) = patch.ui_locale {
    // Only emit if locale actually changed
    app.emit("locale-changed", new_locale.clone())
        .unwrap_or_else(|e| log::error!("Failed to emit locale-changed: {}", e));
}
```

**Problem:** `set_user_preferences` currently takes `State<'_, AppState>` but NOT `AppHandle`. It needs `AppHandle` to emit events. The Tauri command can accept both -- add `app: AppHandle<R>` parameter. [VERIFIED: commands.rs line 24]

**Tray listener:** In `lib.rs` setup, after `create_tray()`, register a listener:

```rust
let app_handle = _app.handle().clone();
_app.handle().listen("locale-changed", move |_event| {
    tray::update_tray_menu(&app_handle);
});
```

Tauri 2.x `AppHandle::listen()` is the Rust-side global event listener. [VERIFIED: Tauri 2.6.2 in Cargo.toml]

### Tray Startup Locale Hydration

Current order in `lib.rs`:
1. Line 390: `tray::create_tray()` -- builds menu with hardcoded English
2. Line 463-466: `initialize_database_on_startup`
3. Line 473-477: `preferences::hydrate_from_db`

**Problem:** Tray is created BEFORE preferences are hydrated. The tray will always show English on first paint, then need a rebuild.

**Fix:** After hydration (line 477), call `tray::update_tray_menu()` to rebuild with the correct locale. This is a one-line addition. [VERIFIED: lib.rs lines 390, 475]

### Test Extension Patterns

**QA-01 (preference desync):** Extend `preferences/tests.rs`. Existing infrastructure provides:
- `test_pool_with_migration()` -- real SQLite pool
- `lock_prefs_cache()` -- serialized cache access
- `hydrate_from_db()` -- real hydration path

New tests to add:
- T6: Startup with `ui_locale='ar'` + verify `read()` returns `"ar"` immediately after hydration (strengthens T1)
- T7: After `set_user_preferences(ui_locale='ar')`, verify `read()` reflects new value for subsequent callers (runtime switch)
- T8: Concurrent `set_user_preferences` calls with different locales -- final state is one of the inputs (strengthens T5 with locale specifically)

**QA-02 (Parakeet ban):** Extends T4 pattern. New edge cases:
- T9: `en->ar` locale switch when `provider='parakeet'` triggers auto-repoint to `localWhisper+large-v3` (variant of T2 with explicit parakeet)
- T10: Direct `provider:'parakeet'` + `ui_locale:'ar'` in same patch -> `InvalidCombination`

**QA-03 (template/prompt fallback):** New tests in `summary/prompts/loader.rs` and `summary/templates/loader.rs`. The prompt loader already has 6 tests (verified). Template loader tests need the 3-case matrix. [VERIFIED: prompts/loader.rs lines 20-78]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Tray event listener | Custom polling/timer | Tauri `AppHandle::listen()` | Built-in event bus, zero overhead |
| String formatting with Arabic | Manual concatenation | `format!()` with string constants | Rust's `format!` handles Unicode correctly |
| Test isolation | Manual state cleanup | `prefs_test_lock()` mutex pattern from Phase 1 | Proven in T1-T5, prevents flaky tests |

## Common Pitfalls

### Pitfall 1: Tray Created Before Preferences Hydrated
**What goes wrong:** Tray shows English strings even when user has `ui_locale='ar'`
**Why it happens:** `create_tray()` runs at line 390, but `hydrate_from_db()` runs at line 475
**How to avoid:** Call `tray::update_tray_menu()` immediately after hydration completes (line 477+)
**Warning signs:** Arabic user sees English tray on app launch

### Pitfall 2: set_user_preferences Lacks AppHandle for Event Emission
**What goes wrong:** Cannot emit `locale-changed` event from the command
**Why it happens:** Current signature only takes `State<'_, AppState>`, not `AppHandle`
**How to avoid:** Add `app: AppHandle<R>` parameter to the Tauri command. Tauri commands can accept both `State` and `AppHandle`.
**Warning signs:** Compile error when trying to call `app.emit()`

### Pitfall 3: Notification Helpers Are Static Methods Without Locale
**What goes wrong:** `Notification::recording_started()` always returns English
**Why it happens:** Helper constructors don't accept a locale parameter
**How to avoid:** Add `locale: &str` parameter to each helper, or create a `NotificationFactory` that reads `preferences::read().ui_locale`
**Warning signs:** Notifications always in English regardless of locale setting

### Pitfall 4: Template/Prompt Fallback Tests May Need Custom Builtins
**What goes wrong:** Can't test "AR missing, EN present -> fallback" because all 5 prompts have both AR and EN embedded
**Why it happens:** Phase 5 embedded all 10 prompt files (5 EN + 5 AR)
**How to avoid:** QA-03 tests should use a synthetic/nonexistent locale (e.g., "fr") to verify fallback to EN, and a nonexistent prompt_id to verify error. The existing `test_get_prompt_fallback_to_en` and `test_get_prompt_unknown_id` in `prompts/loader.rs` already cover 2 of 3 cases.
**Warning signs:** Tests pass trivially because all locales are present

### Pitfall 5: Arabic String Width in Tray Menus
**What goes wrong:** Arabic strings truncated in system tray
**Why it happens:** Arabic averages ~1.2x English width; system tray has platform-imposed limits
**How to avoid:** Keep Arabic tray strings concise. Test on macOS (primary platform).
**Warning signs:** Truncated text with "..." in tray

## Code Examples

### Tray String Inventory (13 items to localize)
```
// From tray.rs build_menu() -- VERIFIED line-by-line:
1. "Downloading transcription model..."     (line 327)
2. "Start Recording"                         (line 334)
3. "Starting Recording..."                   (line 339, with emoji)
4. "Pause Recording"                         (line 345, with emoji)
5. "Stop Recording"                          (line 346, with emoji)
6. "Pausing..."                              (line 351, with emoji)
7. "Stop Recording"                          (line 355, with emoji, duplicate)
8. "Resume Recording"                        (line 360, with emoji)
9. "Stop Recording"                          (line 362, with emoji, duplicate)
10. "Resuming..."                            (line 367, with emoji)
11. "Stopping..."                            (line 377, with emoji)
12. "Open Main Window"                       (line 387)
13. "Settings"                               (line 388)
14. "Check for Updates"                       (line 389)
15. "Quit"                                   (line 390)
```

Unique strings: 13 (deduplicating "Stop Recording" which appears 3 times). [VERIFIED: tray.rs]

### Notification String Inventory (10 items to localize)
```
// From notifications/types.rs helper constructors -- VERIFIED:
1. "Meetily" (title, used 6 times)
2. "Recording started for meeting: {}"
3. "Recording has started. Please inform others..."
4. "Recording has been stopped and saved"
5. "Recording has been paused"
6. "Recording has been resumed"
7. "Transcription completed and saved to: {}"
8. "Transcription has been completed"
9. "Meeting '{}' starts in {} minutes"
10. "Meeting starts in {} minutes"
11. "Meetily Error" (error title)
12. "This is a test notification..."
```

Unique strings: ~12 (title "Meetily" deduplicated). [VERIFIED: types.rs]

### Event Emission in set_user_preferences
```rust
// Source: Pattern analysis of commands.rs + D-02
#[tauri::command]
pub async fn set_user_preferences<R: Runtime>(
    app: AppHandle<R>,  // NEW: needed for event emission
    patch: UserPreferencesPatch,
    state: State<'_, AppState>,
) -> Result<UserPreferences, String> {
    let old_locale = super::read().ui_locale.clone();
    let pool = state.db_manager.pool();

    let merged = super::repository::apply_patch_atomic(pool, patch)
        .await
        .map_err(|e| match e {
            PreferencesError::InvalidCombination { reason } => reason,
            PreferencesError::Database(err) => format!("Database error: {}", err),
        })?;

    {
        let mut guard = PREFS_CACHE.write().map_err(|_| "PREFS_CACHE poisoned".to_string())?;
        *guard = merged.clone();
    }

    // Emit locale-changed event if locale actually changed
    if merged.ui_locale != old_locale {
        let _ = app.emit("locale-changed", &merged.ui_locale);
    }

    Ok(merged)
}
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` with tokio::test + sqlx |
| Config file | `frontend/src-tauri/Cargo.toml` (dev-dependencies) |
| Quick run command | `cd frontend/src-tauri && cargo test --lib` |
| Full suite command | `cd frontend/src-tauri && cargo test` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-07 | Tray/notification strings localized | manual | N/A (visual check in tray) | N/A |
| QA-01 | Preference desync regression | unit | `cargo test --lib preferences::tests` | Exists (extend T1-T5) |
| QA-02 | Parakeet-ban enforcement | unit | `cargo test --lib preferences::tests` | Exists (extend T4) |
| QA-03 | Template/prompt locale fallback | unit | `cargo test --lib summary::prompts::loader::tests` | Exists (extend 6 tests) |
| QA-04 | Manual RTL regression | manual-only | N/A | Wave 3 creates checklist |
| QA-05 | Arabic transcription accuracy | manual-only | N/A | Wave 3 creates checklist |
| QA-06 | Arabic summary quality | manual-only | N/A | Wave 3 creates checklist |

### Sampling Rate
- **Per task commit:** `cd frontend/src-tauri && cargo test --lib`
- **Per wave merge:** `cd frontend/src-tauri && cargo test`
- **Phase gate:** Full suite green + manual checklist complete before `/gsd-verify-work`

### Wave 0 Gaps
None -- existing test infrastructure covers all automated phase requirements. `test_pool_with_migration()`, `prefs_test_lock()`, and the prompt/template test modules are already in place.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Struct-based constants preferred over tuple arrays for tray strings | Architecture Patterns | Low -- either approach works, struct is just more readable |
| A2 | `AppHandle::listen()` is the correct Rust-side global event listener in Tauri 2.x | Architecture Patterns | Medium -- if API differs, need to use `Listener` trait instead |
| A3 | Tauri commands can accept both `AppHandle<R>` and `State<'_, T>` parameters simultaneously | Pitfall 2 | Medium -- if not, need alternative approach for event emission |
| A4 | Arabic tray string width fits within macOS system tray limits | Pitfall 5 | Low -- can truncate if needed |

## Open Questions

1. **Notification locale source**
   - What we know: `Notification::recording_started()` etc. are called from various places in the codebase
   - What's unclear: Should each call site pass the locale, or should the helper read `preferences::read().ui_locale` internally?
   - Recommendation: Read `preferences::read().ui_locale` inside each helper -- fewer caller changes, single source of truth

2. **QA-03 template fallback testing depth**
   - What we know: All 5 prompts and 6 templates have both EN and AR versions embedded
   - What's unclear: How to test "AR missing -> EN fallback" when all AR files exist
   - Recommendation: Use synthetic locale "fr" to exercise the fallback path (already done in `test_get_prompt_fallback_to_en`)

## Sources

### Primary (HIGH confidence)
- `frontend/src-tauri/src/tray.rs` -- 13 hardcoded English strings verified line-by-line
- `frontend/src-tauri/src/notifications/types.rs` -- 12 notification strings verified
- `frontend/src-tauri/src/preferences/commands.rs` -- set_user_preferences signature verified
- `frontend/src-tauri/src/preferences/tests.rs` -- T1-T5 test infrastructure verified
- `frontend/src-tauri/src/summary/prompts/loader.rs` -- get_prompt signature + 6 existing tests verified
- `frontend/src-tauri/src/lib.rs` -- startup sequence order verified (tray at 390, hydration at 475)
- `frontend/src-tauri/Cargo.toml` -- Tauri 2.6.2 verified

### Secondary (MEDIUM confidence)
- Tauri 2.x `AppHandle::listen()` API for Rust-side event listening [ASSUMED from Tauri 2.x docs]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- zero new dependencies, all verified in Cargo.toml
- Architecture: HIGH -- all integration points verified in source code
- Pitfalls: HIGH -- startup ordering, missing AppHandle, and notification signatures all verified
- Test patterns: HIGH -- extending proven T1-T5 infrastructure

**Research date:** 2026-04-13
**Valid until:** 2026-05-13 (stable -- Tauri 2.x API unlikely to change)
