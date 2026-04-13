---
phase: 05-templates-prompts-bilingual-content
plan: 02
subsystem: summary
tags: [prompts, llm, include_str, i18n, arabic, msa]

requires:
  - phase: 05-01
    provides: "Arabic template JSONs and locale-aware template loader pattern to mirror"
provides:
  - "10 externalized LLM prompt .txt files (5 EN + 5 AR) with named placeholders"
  - "summary/prompts/ Rust module with get_prompt(id, locale) and EN fallback"
  - "All prompts embedded via include_str! for offline builds"
affects: [05-03-pipeline-wiring, 05-04-blocknote-arabic]

tech-stack:
  added: []
  patterns: ["include_str! prompt embedding mirroring templates/defaults.rs", "locale-aware prompt loader with EN fallback"]

key-files:
  created:
    - "frontend/src-tauri/prompts/*.txt (10 files)"
    - "frontend/src-tauri/src/summary/prompts/defaults.rs"
    - "frontend/src-tauri/src/summary/prompts/loader.rs"
    - "frontend/src-tauri/src/summary/prompts/mod.rs"
  modified:
    - "frontend/src-tauri/src/summary/mod.rs"
    - "frontend/src-tauri/src/summary/templates/loader.rs"

key-decisions:
  - "Added Arabic punctuation instructions to AR user prompts (not just system prompts) to satisfy D-06 requirement that ALL 5 AR files contain Arabic comma character"
  - "Fixed pre-existing templates/loader.rs compilation error by passing 'en' default locale to get_builtin_template (Rule 3 blocking fix)"

patterns-established:
  - "Prompt module mirrors templates module: defaults.rs (include_str! embeds) + loader.rs (get_prompt with fallback) + mod.rs (re-exports)"
  - "Named placeholders in prompts: {transcript_chunk}, {summaries}, {section_instructions}, {template_markdown} -- never bare {}"

requirements-completed: [TPL-03, TPL-04, SUMM-03]

duration: 3min
completed: 2026-04-13
---

# Phase 5 Plan 2: Prompts Module Summary

**10 externalized LLM prompt .txt files (5 EN + 5 AR with D-06 punctuation) embedded via include_str! with locale-aware get_prompt(id, locale) loader**

## Performance

- **Duration:** 3 min
- **Started:** 2026-04-13T05:35:51Z
- **Completed:** 2026-04-13T05:39:05Z
- **Tasks:** 2
- **Files modified:** 15

## Accomplishments
- Extracted 5 inline LLM prompts from processor.rs into 10 .txt files (5 EN + 5 AR)
- All Arabic prompts include D-06 punctuation enforcement (Arabic comma, semicolon, question mark)
- Created summary/prompts/ Rust module mirroring the templates module pattern
- 13 comprehensive tests covering locale resolution, fallback, placeholders, and Arabic punctuation

## Task Commits

Each task was committed atomically:

1. **Task 1: Create 10 prompt .txt files (5 EN + 5 AR)** - `7893574` (feat)
2. **Task 2: Create summary/prompts/ Rust module with defaults.rs + loader.rs** - `872d6da` (feat)

## Files Created/Modified
- `frontend/src-tauri/prompts/chunk_summarizer_system.{en,ar}.txt` - System prompt for chunk summarization
- `frontend/src-tauri/prompts/chunk_summarizer_user.{en,ar}.txt` - User prompt with {transcript_chunk} placeholder
- `frontend/src-tauri/prompts/chunk_combiner_system.{en,ar}.txt` - System prompt for combining chunk summaries
- `frontend/src-tauri/prompts/chunk_combiner_user.{en,ar}.txt` - User prompt with {summaries} placeholder
- `frontend/src-tauri/prompts/final_report_system.{en,ar}.txt` - System prompt with {section_instructions} and {template_markdown} placeholders
- `frontend/src-tauri/src/summary/prompts/defaults.rs` - 10 include_str! embeds + get_builtin_prompt(id, locale)
- `frontend/src-tauri/src/summary/prompts/loader.rs` - get_prompt(id, locale) with EN fallback
- `frontend/src-tauri/src/summary/prompts/mod.rs` - Module re-exports
- `frontend/src-tauri/src/summary/mod.rs` - Added pub mod prompts
- `frontend/src-tauri/src/summary/templates/loader.rs` - Fixed get_builtin_template call to include locale arg

## Decisions Made
- Added Arabic punctuation instructions inline to AR user prompts (chunk_summarizer_user.ar.txt and chunk_combiner_user.ar.txt) so all 5 AR files satisfy D-06's requirement of containing the Arabic comma character
- Fixed pre-existing compilation error in templates/loader.rs where get_builtin_template was called with 1 arg after 05-01 changed it to require 2 args -- passed "en" as default locale

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added Arabic punctuation to AR user prompts**
- **Found during:** Task 1 (prompt file creation)
- **Issue:** Plan's AR user prompt templates did not include the literal Arabic comma character, but acceptance criteria requires all 5 AR files to contain it (D-06)
- **Fix:** Added inline punctuation instruction with Arabic comma, semicolon, question mark to chunk_summarizer_user.ar.txt and chunk_combiner_user.ar.txt
- **Files modified:** frontend/src-tauri/prompts/chunk_summarizer_user.ar.txt, frontend/src-tauri/prompts/chunk_combiner_user.ar.txt
- **Verification:** grep -l confirms all 5 AR files contain Arabic comma
- **Committed in:** 7893574 (Task 1 commit)

**2. [Rule 3 - Blocking] Fixed templates/loader.rs compilation error**
- **Found during:** Task 2 (cargo test)
- **Issue:** Plan 05-01 updated get_builtin_template to take (id, locale) but left the call site in templates/loader.rs with only 1 arg, causing E0061
- **Fix:** Added "en" as default locale argument to the call site
- **Files modified:** frontend/src-tauri/src/summary/templates/loader.rs
- **Verification:** cargo test summary::prompts passes (13/13 tests)
- **Committed in:** 872d6da (Task 2 commit)

---

**Total deviations:** 2 auto-fixed (1 missing critical, 1 blocking)
**Impact on plan:** Both auto-fixes necessary for correctness. No scope creep.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Prompts module ready for plan 05-03 (pipeline wiring) to replace inline prompts in processor.rs with prompts::get_prompt() calls
- Locale threading through commands/service/processor is the next step

## Self-Check: PASSED

- All 13 created files: FOUND
- Commit 7893574: FOUND
- Commit 872d6da: FOUND
- cargo test summary::prompts: 13/13 passed

---
*Phase: 05-templates-prompts-bilingual-content*
*Completed: 2026-04-13*
