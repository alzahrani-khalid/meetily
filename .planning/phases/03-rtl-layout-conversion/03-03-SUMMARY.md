---
phase: 03-rtl-layout-conversion
plan: 03
status: complete
started: 2026-04-09T10:27:09Z
completed: 2026-04-09T10:36:34Z
dependency_graph:
  requires: [03-02]
  provides: [RTL-converted hotspot files, direction-aware sidebar collapse]
  affects: [frontend/src/components/Sidebar/index.tsx, frontend/src/components/ModelSettingsModal.tsx, frontend/src/components/ui/dropdown-menu.tsx, frontend/src/components/AnalyticsDataModal.tsx, frontend/src/components/AISummary/index.tsx, frontend/src/components/ChunkProgressDisplay.tsx, frontend/src/components/BuiltInModelManager.tsx, frontend/src/components/TranscriptRecovery/TranscriptRecovery.tsx, frontend/src/components/MeetingDetails/SummaryPanel.tsx, frontend/src/components/ImportAudio/ImportAudioDialog.tsx]
tech_stack:
  added: []
  patterns: [logical Tailwind properties, rtl:space-x-reverse companion, useLocale direction detection]
key_files:
  created: []
  modified:
    - frontend/src/components/Sidebar/index.tsx
    - frontend/src/components/ModelSettingsModal.tsx
    - frontend/src/components/ui/dropdown-menu.tsx
    - frontend/src/components/AnalyticsDataModal.tsx
    - frontend/src/components/AISummary/index.tsx
    - frontend/src/components/ChunkProgressDisplay.tsx
    - frontend/src/components/BuiltInModelManager.tsx
    - frontend/src/components/TranscriptRecovery/TranscriptRecovery.tsx
    - frontend/src/components/MeetingDetails/SummaryPanel.tsx
    - frontend/src/components/ImportAudio/ImportAudioDialog.tsx
decisions:
  - "D-08: Sidebar reads direction via useLocale() from I18nProvider, derives isRTL = locale === 'ar'"
  - "D-12: translate-x physical classes use inline style branching instead of className, avoiding ESLint exceptions"
metrics:
  duration: 9m 25s
  completed: 2026-04-09
  tasks: 2/2
  files: 10
---

# Phase 3 Plan 3: Hotspot RTL Conversion Summary

10 highest-hit files converted from physical-direction to logical Tailwind classes, plus direction-aware sidebar collapse animation using useLocale() for translate-x branching.

## What Was Built

### Task 1: Sidebar RTL Conversion with Direction-Aware Collapse (UI-05, UI-06)
- Converted all physical-direction classes in `Sidebar/index.tsx`: `left-0` to `start-0`, `-right-6` to `-end-6`, `border-r` to `border-e`, all `ml-`/`mr-` to `ms-`/`me-`
- Added `useLocale()` from `@/providers/I18nProvider` to detect direction
- Collapse button `translateX` uses inline style branching: `isRTL ? 'translateX(-50%)' : 'translateX(50%)'`
- Chevron icons swap based on `isRTL` and `isCollapsed` state:
  - Expanded + RTL: ChevronRightCircle (collapse toward right)
  - Collapsed + RTL: ChevronLeftCircle (expand from right)
  - Expanded + LTR: ChevronLeftCircle (collapse toward left)
  - Collapsed + LTR: ChevronRightCircle (expand from left)

### Task 2: Convert 9 Remaining Hotspot Files (UI-05)
- **ModelSettingsModal.tsx** (17 hits): `ml/mr/pl/pr/right/border-l` to logical equivalents, `space-x` + `rtl:space-x-reverse`
- **dropdown-menu.tsx** (11 hits): All `pl/left/ml` to `ps/start/ms` across SubTrigger, MenuItem, CheckboxItem, RadioItem, Label, Shortcut
- **AnalyticsDataModal.tsx** (6 hits): `ml-4` to `ms-4` across all list sections
- **AISummary/index.tsx** (9 hits): `mr/ml/text-left` to `me/ms/text-start`, `space-x` + `rtl:space-x-reverse`
- **ChunkProgressDisplay.tsx** (7 hits): `space-x` + `rtl:space-x-reverse` on 6 flex containers, `ml` to `ms`
- **BuiltInModelManager.tsx** (6 hits): `ml/mr` to `ms/me` on buttons and spacing
- **TranscriptRecovery.tsx** (5 hits): `text-left` to `text-start`, `mr` to `me` on action buttons
- **SummaryPanel.tsx** (5 hits): `left-0/right-0` to `start-0/end-0`, `pl` to `ps` for lists
- **ImportAudioDialog.tsx** (4 hits): `mr` to `me` on button icons

## Commits

| SHA | Message |
|-----|---------|
| 6426ed2 | feat(03-03): convert Sidebar to logical RTL classes with direction-aware collapse animation [UI-05, UI-06] |
| 102eb97 | feat(03-03): convert 9 hotspot files from physical to logical RTL classes [UI-05] |

## Requirements Satisfied

- **UI-05**: All 10 hotspot files converted to logical Tailwind properties -- zero physical-direction classes remain
- **UI-06**: Sidebar collapse animation slides right in Arabic mode, left in English mode; chevron icons swap correctly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] ModelSettingsModal right-3 positioning icons**
- **Found during:** Task 2
- **Issue:** Two `right-3` absolute positioning classes on validation icons (CheckCircle2, XCircle) were not in the grep hit list but are physical-direction classes
- **Fix:** Converted `right-3` to `end-3` on both validation state icons
- **Files modified:** frontend/src/components/ModelSettingsModal.tsx
- **Commit:** 102eb97

## Known Stubs

None -- all conversions are complete with no placeholder values.

## Threat Flags

None -- CSS class renaming only, no new endpoints or trust boundaries.

## Self-Check: PASSED

- All 10 modified files exist on disk
- Both commit hashes (6426ed2, 102eb97) verified in git log
