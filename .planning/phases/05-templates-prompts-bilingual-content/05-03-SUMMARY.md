---
phase: 05-templates-prompts-bilingual-content
plan: 03
subsystem: summary
tags: [rust, locale, pipeline-wiring, prompts, templates, i18n]

requires:
  - phase: 05-01
    provides: "Locale-aware template loader with get_template(id, locale)"
  - phase: 05-02
    provides: "Prompts module with get_prompt(id, locale) and 10 externalized .txt files"
provides:
  - "Fully locale-aware summary pipeline: commands -> service -> processor -> prompts + templates"
  - "Summary language read from PREFS_CACHE.summary_language (independent from UI locale per SUMM-01)"
  - "Named placeholders {transcript_chunk}, {summaries}, {section_instructions}, {template_markdown} replacing bare {}"
affects: [05-04-blocknote-arabic]

tech-stack:
  added: []
  patterns: ["locale threading through async pipeline", "preferences-driven locale resolution at command entry point"]

key-files:
  created: []
  modified:
    - frontend/src-tauri/src/summary/processor.rs
    - frontend/src-tauri/src/summary/service.rs
    - frontend/src-tauri/src/summary/commands.rs
    - frontend/src-tauri/src/summary/template_commands.rs

key-decisions:
  - "template_commands.rs uses ui_locale (not summary_language) for template display names -- template listing is a UI concern"
  - "No changes to mod.rs re-exports needed -- function signature change is transparent through pub use"

patterns-established:
  - "Summary locale flows: commands.rs reads preferences::read().summary_language -> service.rs passes as String -> processor.rs uses as &str for get_prompt/get_template"
  - "UI template display uses preferences::read().ui_locale separately from summary generation locale"

requirements-completed: [SUMM-01, TPL-04]

duration: 3min
completed: 2026-04-13
---

# Phase 5 Plan 3: Pipeline Wiring Summary

**Locale-aware summary pipeline wired from PREFS_CACHE.summary_language through commands/service/processor to externalized prompts and templates with named placeholders**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-13T05:43:57Z
- **Completed:** 2026-04-13T05:46:54Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Replaced all 5 inline prompt strings in processor.rs with prompts::get_prompt(id, locale) calls
- Replaced all bare {} placeholders with named placeholders: {transcript_chunk}, {summaries}, {section_instructions}, {template_markdown}
- Added locale parameter to generate_meeting_summary and process_transcript_background signatures
- Wired summary_language from PREFS_CACHE through commands.rs -> service.rs -> processor.rs
- Updated template_commands.rs to use ui_locale for template display (separate from summary locale)
- Full cargo check passes, all 41 summary module tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Rewire processor.rs to use externalized prompts with locale** - `af3da94` (feat)
2. **Task 2: Thread locale through service.rs and commands.rs from PREFS_CACHE** - `dc1678a` (feat)

## Files Modified
- `frontend/src-tauri/src/summary/processor.rs` - Locale-aware summary generation using externalized prompts with named placeholders
- `frontend/src-tauri/src/summary/service.rs` - Added locale parameter to process_transcript_background, passes to processor
- `frontend/src-tauri/src/summary/commands.rs` - Reads preferences::read().summary_language and passes as summary_locale
- `frontend/src-tauri/src/summary/template_commands.rs` - Uses preferences::read().ui_locale for template display

## Decisions Made
- template_commands.rs uses ui_locale for template listing/details display, keeping UI concerns separate from summary generation locale (SUMM-01 compliance)
- No mod.rs changes needed since re-exports are transparent to signature changes

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full locale-aware pipeline operational for both EN and AR summaries
- Plan 05-04 (BlockNote Arabic display) can proceed -- summaries will arrive in the correct locale

## Self-Check: PASSED

- All 4 modified files verified present on disk
- Commit af3da94 (Task 1): FOUND
- Commit dc1678a (Task 2): FOUND
- cargo check: 0 errors
- cargo test summary: 41/41 passed

---
*Phase: 05-templates-prompts-bilingual-content*
*Completed: 2026-04-13*
