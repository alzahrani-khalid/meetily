# Phase 6: Rust Strings, QA & Release Hardening - Pattern Map

**Mapped:** 2026-04-15
**Files analyzed:** 5 files to modify
**Analogs found:** 5 / 5

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `frontend/src-tauri/src/tray.rs` | utility | event-driven | `frontend/src-tauri/src/tray.rs` (self) | exact — modify in place |
| `frontend/src-tauri/src/notifications/types.rs` | utility | request-response | `frontend/src-tauri/src/notifications/types.rs` (self) | exact — modify in place |
| `frontend/src-tauri/src/preferences/commands.rs` | controller | request-response | `frontend/src-tauri/src/preferences/commands.rs` (self) | exact — modify in place |
| `frontend/src-tauri/src/preferences/tests.rs` | test | CRUD | `frontend/src-tauri/src/preferences/tests.rs` (self, T1–T5) | exact — extend in place |
| `frontend/src-tauri/src/summary/prompts/loader.rs` | utility | request-response | `frontend/src-tauri/src/summary/prompts/loader.rs` (self, 6 existing tests) | exact — extend in place |

---

## Pattern Assignments

### `frontend/src-tauri/src/tray.rs` — Add bilingual string constants + locale-aware `build_menu()`

**Analog:** `frontend/src-tauri/src/tray.rs` (existing file)

**Imports pattern** (lines 1–6) — copy these, add `Listener` if needed for `listen()`:
```rust
use tauri::{
    Emitter,
    menu::{MenuBuilder, MenuItemBuilder, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, Runtime,
};
```

**Core constant struct pattern** — define above `build_menu()`, modelled on D-01:
```rust
pub struct TrayStrings {
    pub downloading_model: &'static str,
    pub start_recording: &'static str,
    pub starting_recording: &'static str,
    pub pause_recording: &'static str,
    pub stop_recording: &'static str,
    pub pausing: &'static str,
    pub resume_recording: &'static str,
    pub resuming: &'static str,
    pub stopping: &'static str,
    pub open_main_window: &'static str,
    pub settings: &'static str,
    pub check_for_updates: &'static str,
    pub quit: &'static str,
}

pub const TRAY_EN: TrayStrings = TrayStrings {
    downloading_model: "Downloading transcription model...",
    start_recording: "Start Recording",
    // ... fill all 13 fields
};

pub const TRAY_AR: TrayStrings = TrayStrings {
    downloading_model: "جارٍ تنزيل نموذج النسخ...",
    start_recording: "بدء التسجيل",
    // ... fill all 13 fields
};

pub fn tray_strings(locale: &str) -> &'static TrayStrings {
    match locale {
        "ar" => &TRAY_AR,
        _ => &TRAY_EN,
    }
}
```

**Core build_menu() usage pattern** (lines 316–392) — replace hardcoded strings with struct fields:
```rust
// Before (line 326):
&MenuItemBuilder::new("⏳ Downloading transcription model...")

// After — emoji prefix preserved (D-03), only text label swapped:
let s = tray_strings(&preferences::read().ui_locale);
&MenuItemBuilder::new(format!("⏳ {}", s.downloading_model))

// Before (line 334):
&MenuItemBuilder::with_id("toggle_recording", "Start Recording")

// After:
&MenuItemBuilder::with_id("toggle_recording", s.start_recording)
```

**update_tray_menu() function** — new public fn, called from `lib.rs` after hydration and from locale-changed listener:
```rust
pub fn update_tray_menu<R: Runtime>(app: &AppHandle<R>) {
    // Read current recording state + can_record flag (same as existing update_tray_state)
    // then call build_menu() with current locale strings
    // Pattern mirrors existing update_tray_state() in same file
}
```

**Locale-changed listener registration** — add in `lib.rs` after line 477 (post-hydration):
```rust
// After preferences::hydrate_from_db(...):
let app_handle_for_tray = _app.handle().clone();
_app.handle().listen("locale-changed", move |_event| {
    tray::update_tray_menu(&app_handle_for_tray);
});
// Then immediately rebuild with correct locale now that cache is populated:
tray::update_tray_menu(_app.handle());
```

---

### `frontend/src-tauri/src/notifications/types.rs` — Add bilingual string constants + locale param to helpers

**Analog:** `frontend/src-tauri/src/notifications/types.rs` lines 114–199

**Existing helper constructor pattern** (lines 116–125) — this is what gets modified:
```rust
pub fn recording_started(meeting_name: Option<String>) -> Self {
    let body = match meeting_name {
        Some(name) => format!("Recording started for meeting: {}", name),
        None => "Recording has started. Please inform others...".to_string(),
    };
    Notification::new("Meetily", body, NotificationType::RecordingStarted)
        .with_priority(NotificationPriority::High)
        .with_timeout(NotificationTimeout::Seconds(5))
}
```

**New pattern** — read locale internally from `preferences::read()` (per RESEARCH open question recommendation):
```rust
pub struct NotificationStrings {
    pub app_name: &'static str,
    pub app_name_error: &'static str,
    pub recording_started_named: &'static str,   // format: "{}" = meeting name
    pub recording_started_generic: &'static str,
    pub recording_stopped: &'static str,
    pub recording_paused: &'static str,
    pub recording_resumed: &'static str,
    pub transcription_complete_named: &'static str, // format: "{}" = path
    pub transcription_complete_generic: &'static str,
    pub meeting_reminder_named: &'static str,    // format: "{}" title, "{}" minutes
    pub meeting_reminder_generic: &'static str,  // format: "{}" minutes
    pub test_notification: &'static str,
}

pub const NOTIF_EN: NotificationStrings = NotificationStrings { /* English strings */ };
pub const NOTIF_AR: NotificationStrings = NotificationStrings { /* Arabic strings */ };

fn notif_strings(locale: &str) -> &'static NotificationStrings {
    match locale { "ar" => &NOTIF_AR, _ => &NOTIF_EN }
}

// Helper constructors become locale-aware without changing call sites:
pub fn recording_started(meeting_name: Option<String>) -> Self {
    let s = notif_strings(&crate::preferences::read().ui_locale);
    let body = match meeting_name {
        Some(name) => format!("{}", /* s.recording_started_named with name */),
        None => s.recording_started_generic.to_string(),
    };
    Notification::new(s.app_name, body, NotificationType::RecordingStarted)
        .with_priority(NotificationPriority::High)
        .with_timeout(NotificationTimeout::Seconds(5))
}
```

---

### `frontend/src-tauri/src/preferences/commands.rs` — Add `AppHandle` param + emit `locale-changed`

**Analog:** `frontend/src-tauri/src/preferences/commands.rs` lines 23–54

**Current signature** (lines 23–27) — this is the base to modify:
```rust
#[tauri::command]
pub async fn set_user_preferences(
    patch: UserPreferencesPatch,
    state: State<'_, AppState>,
) -> Result<UserPreferences, String> {
```

**New signature** — add `app: AppHandle<R>` as first parameter (Tauri commands accept both):
```rust
#[tauri::command]
pub async fn set_user_preferences<R: Runtime>(
    app: tauri::AppHandle<R>,
    patch: UserPreferencesPatch,
    state: State<'_, AppState>,
) -> Result<UserPreferences, String> {
```

**Locale-changed emission** — insert after PREFS_CACHE write block (after line 51), before `Ok(merged)`:
```rust
// Capture old locale BEFORE apply_patch_atomic (add at top of fn body):
let old_locale = super::read().ui_locale.clone();

// ... existing apply_patch_atomic + cache write ...

// Emit locale-changed only when locale actually changed:
if merged.ui_locale != old_locale {
    let _ = app.emit("locale-changed", &merged.ui_locale);
}

Ok(merged)
```

**Required import addition** — add to existing `use tauri::State;`:
```rust
use tauri::{Runtime, State};
```

---

### `frontend/src-tauri/src/preferences/tests.rs` — Extend with T6–T10 (QA-01, QA-02)

**Analog:** `frontend/src-tauri/src/preferences/tests.rs` — existing T1–T5 tests (lines 114–end)

**Test infrastructure to reuse** (lines 27–112) — do NOT duplicate, just call:
```rust
// Every new test acquires the lock first (mandatory for PREFS_CACHE isolation):
let _guard = lock_prefs_cache().await;
let pool = test_pool_with_migration().await;
hydrate_from_db(&pool).await.expect("hydrate");
```

**T1 pattern** (lines 118–139) — template for T6 (startup locale):
```rust
#[tokio::test]
async fn hydration_reflects_seeded_row() {
    let _guard = lock_prefs_cache().await;
    let pool = test_pool_with_migration().await;
    sqlx::query("UPDATE user_preferences SET ui_locale = 'ar' WHERE id = '1'")
        .execute(&pool).await.expect("seed update failed");
    hydrate_from_db(&pool).await.expect("hydrate_from_db failed");
    let prefs = read();
    assert_eq!(prefs.ui_locale, "ar", "hydration did not reflect seeded row");
}
```

**T2 pattern** (lines 146–188) — template for T9 (en→ar parakeet auto-repoint). Key elements to copy:
```rust
// Setup: seed parakeet provider
sqlx::query("UPDATE transcript_settings SET provider = 'parakeet', model = 'parakeet-tdt-0.6b' WHERE id = '1'")
    .execute(&pool).await.unwrap();

// Act: apply patch via repository
let patch = UserPreferencesPatch { ui_locale: Some("ar".to_string()), ..Default::default() };
let merged = repository::apply_patch_atomic(&pool, patch).await
    .expect("apply_patch_atomic should succeed for auto-repoint");

// Post-commit cache update (mirrors commands.rs ordering):
*PREFS_CACHE.write().expect("PREFS_CACHE poisoned") = merged.clone();

// Assert with variant match (Anti-Sampling Rule #5):
assert_eq!(merged.ui_locale, "ar");
```

**T4 pattern** (use for T10 — direct parakeet rejection):
```rust
// T4 tests InvalidCombination variant match — copy this error assertion style:
let result = repository::apply_patch_atomic(&pool, patch).await;
assert!(matches!(result, Err(PreferencesError::InvalidCombination { .. })));
```

**New tests to add** (append after existing T5):
- **T6** (`qa01_startup_ar_locale_hydrates_correctly`): Seed `ui_locale='ar'`, hydrate, assert `read().ui_locale == "ar"`. Strengthens T1.
- **T7** (`qa01_runtime_locale_switch_visible_in_cache`): Hydrate with default, apply patch `ui_locale='ar'`, write cache, assert `read()` returns `"ar"`. Tests runtime switch path.
- **T8** (`qa01_concurrent_locale_setters_no_partial_state`): Two concurrent `apply_patch_atomic` calls with different locales; final state must equal one of the inputs. Mirrors T5 structure.
- **T9** (`qa02_en_to_ar_switch_repoints_parakeet`): Copy T2 exactly. Rename + comment to bind QA-02.
- **T10** (`qa02_direct_parakeet_ar_patch_rejected`): Apply `{ provider: "parakeet", ui_locale: "ar" }` in one patch; assert `InvalidCombination` variant.

---

### `frontend/src-tauri/src/summary/prompts/loader.rs` — Extend with QA-03 3-case matrix (also apply to `templates/loader.rs`)

**Analog:** `frontend/src-tauri/src/summary/prompts/loader.rs` — existing 6 tests (lines 20–78)

**Core function signature** (line 14) — QA-03 tests call this directly:
```rust
pub fn get_prompt(prompt_id: &str, locale: &str) -> Result<&'static str, String>
```

**Existing 3-case coverage** — already covers 2 of 3 QA-03 cases:
```rust
// Case 1: AR locale present → AR returned (line 31):
fn test_get_prompt_ar() {
    let result = get_prompt("chunk_summarizer_system", "ar");
    assert!(result.is_ok());
    assert!(result.unwrap().contains('\u{060C}')); // Arabic comma
}

// Case 2: AR missing (synthetic "fr" locale) + EN present → EN fallback (line 39):
fn test_get_prompt_fallback_to_en() {
    let result = get_prompt("chunk_summarizer_system", "fr");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("expert meeting summarizer"));
    assert!(!result.unwrap().contains('\u{060C}'));
}

// Case 3: Both missing → error (line 48):
fn test_get_prompt_unknown_id() {
    let result = get_prompt("nonexistent_prompt", "en");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}
```

**QA-03 additions for prompts** — add explicit QA-03-labelled tests that name all 3 cases:
```rust
#[test]
fn qa03_ar_locale_returns_ar_content() {
    // Case 1: AR file present → AR returned
    let result = get_prompt("chunk_summarizer_system", "ar");
    assert!(result.is_ok());
    assert!(result.unwrap().contains('\u{060C}'), "Expected Arabic content");
}

#[test]
fn qa03_unknown_locale_falls_back_to_en() {
    // Case 2: AR missing (use "fr") + EN present → EN fallback
    let result = get_prompt("chunk_summarizer_system", "fr");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("expert meeting summarizer"));
}

#[test]
fn qa03_unknown_id_returns_error() {
    // Case 3: Both missing → error with "not found"
    let result = get_prompt("nonexistent_prompt_qa03", "ar");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}
```

**Apply identical 3-case matrix to `summary/templates/loader.rs`** — same structure, substitute `get_template(id, locale)` for `get_prompt(id, locale)` and use a template-specific Arabic content marker.

---

## Shared Patterns

### Rust Constant Struct for Bilingual Strings
**Source:** D-01 decision + RESEARCH.md Architecture Patterns section
**Apply to:** `tray.rs` (TrayStrings) and `notifications/types.rs` (NotificationStrings)
```rust
// Pattern: parallel EN/AR const instances of a named-field struct
pub struct XxxStrings { pub field: &'static str, /* ... */ }
pub const XXX_EN: XxxStrings = XxxStrings { field: "English text", /* ... */ };
pub const XXX_AR: XxxStrings = XxxStrings { field: "النص العربي", /* ... */ };
pub fn xxx_strings(locale: &str) -> &'static XxxStrings {
    match locale { "ar" => &XXX_AR, _ => &XXX_EN }
}
```

### Tauri Event Emission (Rust → Rust via AppHandle)
**Source:** `frontend/src-tauri/src/preferences/commands.rs` (pattern to add) + `frontend/src-tauri/src/tray.rs` line 2 (`use tauri::Emitter`)
**Apply to:** `preferences/commands.rs` (emit) and `lib.rs` (listen)
```rust
// Emit (in commands.rs after cache write):
let _ = app.emit("locale-changed", &merged.ui_locale);

// Listen (in lib.rs setup closure after hydration at line 477):
let handle = _app.handle().clone();
_app.handle().listen("locale-changed", move |_event| {
    tray::update_tray_menu(&handle);
});
```

### Test Isolation Lock
**Source:** `frontend/src-tauri/src/preferences/tests.rs` lines 27–36
**Apply to:** All new tokio tests in `preferences/tests.rs`
```rust
// MANDATORY first two lines of every PREFS_CACHE-touching test:
let _guard = lock_prefs_cache().await;
let pool = test_pool_with_migration().await;
```

### Error Variant Match Assertion
**Source:** `frontend/src-tauri/src/preferences/tests.rs` T4 pattern
**Apply to:** T10 (QA-02 direct parakeet rejection test)
```rust
// Use variant match, NOT generic is_err():
assert!(matches!(result, Err(PreferencesError::InvalidCombination { .. })));
```

### Post-Commit Cache Update Ordering
**Source:** `frontend/src-tauri/src/preferences/commands.rs` lines 46–51
**Apply to:** Any test that exercises `apply_patch_atomic` and then checks `read()` — mirrors commands.rs ordering to test the real code path:
```rust
// Write cache ONLY after apply_patch_atomic returned Ok:
*PREFS_CACHE.write().expect("PREFS_CACHE poisoned") = merged.clone();
```

---

## No Analog Found

No files in this phase are greenfield — all files are modifications to existing Rust source. The manual QA checklist (`QA-04-RTL-PASS.md`) is a new Markdown document with no code analog needed; structure it per D-07/D-11 (one section per screen: Sidebar, Settings, Transcript, Summary, Onboarding, Tray, Meeting Details; plus QA-05 and QA-06 sections inline).

---

## Metadata

**Analog search scope:** `frontend/src-tauri/src/` (tray.rs, notifications/types.rs, preferences/commands.rs, preferences/tests.rs, summary/prompts/loader.rs, lib.rs)
**Files scanned:** 6 source files read directly
**Pattern extraction date:** 2026-04-15
