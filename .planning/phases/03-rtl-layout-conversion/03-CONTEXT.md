---
phase: 3
slug: rtl-layout-conversion
status: ready-for-research
gathered: 2026-04-09
milestone: v1.0 — Arabic Bilingual Support
requirements: [UI-05, UI-06, QA-07]
upstream_artifacts:
  - .planning/phases/02-i18n-framework-locale-bootstrap/02-CONTEXT.md
  - .planning/phases/01-preferences-foundation/01-CONTEXT.md
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md §4, §7, §11, §12.3
---

# Phase 3: RTL Layout Conversion — Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Convert every visible screen to mirror correctly in Arabic mode. This phase delivers three outcomes:

1. **BlockNote RTL spike (decision gate)** — A time-boxed investigation answering 4 questions from spec §7 about BlockNote v0.36.0's RTL capabilities. The outcome locks SUMM-04's path (editable vs read-only) for Phase 5.
2. **ESLint guardrail (QA-07)** — A `no-restricted-syntax` rule that prevents physical-direction Tailwind classes from entering the codebase, enforced as an error in both CI and local dev.
3. **Full RTL conversion (UI-05, UI-06)** — Replace all physical-direction Tailwind classes with logical equivalents across ~47 `.tsx` files, plus sidebar collapse animation branching on `dir`.

**Requirements covered:** UI-05, UI-06, QA-07

**Fixed out-of-scope for this phase:**
- Migrating component strings into message catalogues (already done in Phase 2 for i18n-wired components; remaining strings are Phase 5/6 concern)
- Any Parakeet UI filter, onboarding fork, or transcription policy changes (Phase 4)
- Template/prompt locale resolution (Phase 5)
- Tray menu / system notification Arabic strings (Phase 6)
- BlockNote editing features beyond the spike (EDIT-01, EDIT-02 are v2)
- Manual RTL regression pass (QA-04 is Phase 6)

**Phase 2 surface this phase consumes:**
- `<html dir>` attribute switching (set by root layout based on `uiLocale`)
- `I18nProvider` wrapping the component tree
- Tajawal font loaded and active for Arabic locale
- `globals.css` already has `[dir="rtl"]` selectors for base direction

</domain>

<decisions>
## Implementation Decisions

### BlockNote RTL Spike
- **D-01:** The spike tests ALL 4 questions from spec §7: (1) Does BlockNote render RTL text correctly? (2) Does the slash menu work in RTL? (3) Can the `dictionary` prop localize block labels? (4) Is cursor behavior correct in RTL?
- **D-02:** The pass bar is strict — **all 4 questions must pass** for SUMM-04 to ship as editable. If ANY question fails, SUMM-04 falls back to read-only rendered markdown (spec §12.3 red path). Slash menu and dictionary failures are NOT deferred to v2 for the pass/fail decision — they block the editable path.
- **D-03:** Spike deliverable is a written decision doc + a minimal test page (kept in repo as evidence) + a screenshot showing the RTL render result. All three artifacts live in the phase directory.
- **D-04:** The spike is the FIRST plan in this phase. No other RTL conversion work begins until the spike decision is locked.

### Conversion Strategy & Ordering
- **D-05:** Claude's Discretion — Claude decides the optimal batching and ordering of file conversions. The important constraint is: ESLint rule lands BEFORE any conversion work (so regressions are caught during the conversion itself), and all files end up converted by phase end.
- **D-06:** Claude's Discretion — shadcn/ui primitives (dialog, dropdown-menu, sheet, accordion, select, alert-dialog, scroll-area, command, button-group, input-group) are vendored project code. Claude decides whether to convert them as a separate batch before hotspots, inline with hotspots, or in any other ordering that makes sense.
- **D-07:** Claude's Discretion — Claude decides whether to use a strict 1:1 mapping table (ml→ms, mr→me, pl→ps, pr→pe, text-left→text-start, text-right→text-end, border-l→border-s, border-r→border-e, rounded-l→rounded-s, rounded-r→rounded-e), contextual per-case evaluation, or a hybrid approach with an exception list.

### Sidebar Collapse Animation (UI-06)
- **D-08:** Claude's Discretion — Claude decides how the sidebar component reads the current direction (DOM `dir` attribute, locale context, or another approach) and how `translate-x` is branched to slide toward the correct edge in each direction.
- **D-09:** The sidebar must collapse toward the RIGHT edge in Arabic mode and toward the LEFT edge in English mode. This is the only place where logical properties don't apply — `translate-x` has no logical equivalent. Explicit testing in both directions is required.

### ESLint Guardrail (QA-07)
- **D-10:** The rule bans: `ml-*`, `mr-*`, `pl-*`, `pr-*`, `text-left`, `text-right`, `border-l-*`, `border-r-*`, `rounded-l-*`, `rounded-r-*` in all `frontend/src/**/*.tsx` files.
- **D-11:** The rule is an **error** in both CI and local development. Developers see red squiggles immediately; PRs are blocked.
- **D-12:** Claude's Discretion — Claude decides the exception mechanism for legitimate physical-direction usages (e.g., `translate-x` centering in dialogs, switch toggle animations). Options include inline `eslint-disable` comments or an allowlist in the ESLint config.
- **D-13:** The ESLint rule lands as its own plan, BEFORE the hotspot conversion plan starts. This ensures the guardrail catches any regression during the conversion itself.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Spec
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §4 — RTL layout conversion requirements, hotspot list, directional class inventory
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §7 — BlockNote RTL spike: 4 questions, success criteria, fallback decision
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §11 — Phase structure rationale (spike-as-first-plan)
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §12.3 — SUMM-04 fallback: read-only rendered markdown if spike fails

### Requirements
- `.planning/REQUIREMENTS.md` — UI-05 (layout mirroring), UI-06 (sidebar collapse animation), QA-07 (ESLint guardrail)

### Prior Phase Context
- `.planning/phases/02-i18n-framework-locale-bootstrap/02-CONTEXT.md` — `<html dir>` switching, I18nProvider integration, globals.css RTL selectors
- `.planning/phases/01-preferences-foundation/01-CONTEXT.md` — `preferences::read().ui_locale` source for direction

### CLAUDE.md RTL Rules
- `CLAUDE.md` §"MANDATORY: RTL-First Rules for React Native" — While written for React Native, the 5 immutable RTL rules and the mental model apply to the Tailwind conversion. Specifically: no `.reverse()`, no `textAlign: "right"` (maps to no `text-right` in Tailwind), directional icon conventions.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **BlockNote v0.36.0** already installed (`@blocknote/core`, `@blocknote/react`, `@blocknote/shadcn`) — spike can test immediately without dependency changes
- **`BlockNoteEditor/Editor.tsx`** and `BasicBlockNoteTest.tsx` exist — spike can extend or create a parallel test page
- **`globals.css`** already has `[dir="rtl"]` selectors from Phase 2 — RTL-specific overrides can be added here
- **ESLint config** (`frontend/eslint.config.mjs`) uses flat config with `@eslint/eslintrc` compat layer — `no-restricted-syntax` rule can be added directly

### Established Patterns
- **shadcn/ui components** are vendored in `frontend/src/components/ui/` — they're project code, safe to edit directly
- **Tailwind CSS** is the styling approach — no CSS modules, no styled-components
- **`SettingsModal.tsx`** already uses `rtl:` Tailwind variant for the toggle switch — shows the team is aware of RTL utilities

### Integration Points
- **`frontend/src/components/Sidebar/index.tsx`** — sidebar collapse animation (UI-06 target)
- **`frontend/eslint.config.mjs`** — ESLint rule addition point
- **47 `.tsx` files** with physical-direction classes need conversion (full list in codebase scan)

### Files by Category
- **10 Hotspot files** (~51% of hits): Sidebar, ModelSettingsModal, AnalyticsDataModal, AISummary/index.tsx, WhisperModelManager, ChunkProgressDisplay, SettingsModal, dropdown-menu, SummaryPanel, ImportAudioDialog
- **~12 shadcn/ui primitives**: dialog, dropdown-menu, sheet, accordion, select, alert-dialog, scroll-area, command, button-group, input-group, alert, switch
- **~25 remaining application files**: page.tsx, StatusOverlays, About, AudioLevelMeter, BetaSettings, RecordingControls, onboarding steps, etc.

</code_context>

<specifics>
## Specific Ideas

- The spike screenshot should show Arabic content in BlockNote with visible RTL alignment — saved in phase directory as evidence for the decision
- The spike test page should be kept in the repo (not throwaway) so future developers can verify the decision
- ESLint rule should be strict enough that any new physical-direction class is immediately caught in the editor

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-rtl-layout-conversion*
*Context gathered: 2026-04-09*
