---
phase: 03-rtl-layout-conversion
plan: 01
subsystem: ui
tags: [blocknote, rtl, arabic, prosemirror, i18n, spike]

requires:
  - phase: 02-i18n-framework-locale-bootstrap
    provides: "html dir attribute switching, Tajawal font loaded, globals.css RTL selectors"
provides:
  - "BlockNote RTL spike test page with Arabic locale configuration"
  - "SUMM-04 path decision: EDITABLE (all 4 questions pass)"
  - "Evidence that BlockNote v0.36.0 supports RTL natively via CSS inheritance + ar dictionary"
affects: [05-summary-template-locale]

tech-stack:
  added: []
  patterns:
    - "BlockNote Arabic config: dictionary: ar from @blocknote/core/locales"
    - "Remove @blocknote/core/fonts/inter.css for Arabic — inherit Tajawal from global CSS"

key-files:
  created:
    - frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx
    - .planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md
  modified: []

key-decisions:
  - "SUMM-04 path locked to EDITABLE — all 4 BlockNote RTL spike questions pass"
  - "No read-only markdown fallback needed for Phase 5"

patterns-established:
  - "BlockNote RTL: wrap in dir=rtl, pass dictionary: ar, skip inter.css import"

requirements-completed: [UI-05]

duration: 2min
completed: 2026-04-09
---

# Phase 3 Plan 01: BlockNote RTL Spike Summary

**BlockNote v0.36.0 RTL spike passes all 4 questions — SUMM-04 locked to editable path with ar dictionary locale**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-09T10:17:22Z
- **Completed:** 2026-04-09T10:19:15Z
- **Tasks:** 2
- **Files created:** 2

## Accomplishments

- Created BlockNote RTL spike test page with Arabic locale, dir=rtl wrapper, Tajawal font inheritance, and multi-line Arabic content covering all 4 spec section 7 questions
- Wrote decision document answering Q1-Q4 with evidence: ProseMirror CSS inheritance (Q1), direction-agnostic popup positioning (Q2), comprehensive ar locale dictionary (Q3), native browser contenteditable RTL cursor handling (Q4)
- Locked SUMM-04 to EDITABLE path — Phase 5 can proceed with full BlockNote editor in Arabic mode

## Task Commits

Each task was committed atomically:

1. **Task 1: Create BlockNote RTL spike test page** - `7df62ac` (feat)
2. **Task 2: Run spike tests and write decision document** - `1cf23c4` (docs)

## Files Created/Modified

- `frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx` - RTL spike test page with Arabic locale, editable mode, multi-block Arabic content
- `.planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md` - Decision document answering all 4 spec section 7 questions, locking SUMM-04 to EDITABLE

## Decisions Made

- **SUMM-04 path: EDITABLE** — All 4 BlockNote RTL questions pass. No read-only markdown fallback needed. Phase 5 can implement the summary editor as a fully editable BlockNote instance with `dictionary: ar`.
- **Inter font exclusion confirmed** — `@blocknote/core/fonts/inter.css` must not be imported when rendering Arabic content. Editor inherits Tajawal from global CSS cascade.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Spike decision is locked. Phase 3 Plans 02-04 (ESLint guardrail, hotspot conversion, sidebar animation) can proceed without waiting.
- Phase 5 (Summary & Template Locale) now knows to implement SUMM-04 as editable BlockNote with `dictionary: ar`, not read-only markdown.
- The spike test page remains in the repo at `frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx` as permanent evidence per D-03.

## Self-Check: PASSED

- FOUND: `frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx`
- FOUND: `.planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md`
- FOUND: `.planning/phases/03-rtl-layout-conversion/03-01-SUMMARY.md`
- FOUND: commit `7df62ac`
- FOUND: commit `1cf23c4`

---
*Phase: 03-rtl-layout-conversion*
*Completed: 2026-04-09*
