# Phase 3: RTL Layout Conversion - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 03-rtl-layout-conversion
**Areas discussed:** BlockNote RTL spike scope, Conversion strategy & ordering, Sidebar collapse animation, ESLint rule design

---

## BlockNote RTL Spike Scope

### Q1: Spike question scope

| Option | Description | Selected |
|--------|-------------|----------|
| Keep all 4 as-is | Spike tests all 4 questions from spec §7. If any fail, fallback is read-only rendered markdown. | ✓ |
| Reduce — only test render + cursor | Slash menu and dictionary are v2 concerns. For v1, only confirm RTL render and cursor. | |
| You decide | Claude picks the scope that makes sense given v1 vs v2 requirements. | |

**User's choice:** Keep all 4 as-is
**Notes:** Full spike scope preserved from spec §7.

### Q2: Spike deliverable format

| Option | Description | Selected |
|--------|-------------|----------|
| Written decision doc only | Short markdown answering each question with PASS/FAIL and locked SUMM-04 path. | |
| Decision doc + minimal test page | Doc plus a test page loading BlockNote with Arabic content — kept in repo as evidence. | |
| Decision doc + test page + screenshot | All above plus a screenshot saved in phase directory showing RTL render result. | ✓ |

**User's choice:** Decision doc + test page + screenshot
**Notes:** Maximum evidence for the decision.

### Q3: Pass bar definition

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — render + cursor = PASS | Render + cursor pass = editable. Slash menu and dictionary are v2 stretch goals. | |
| No — all 4 must pass | Any failure → read-only. Want fully usable Arabic editing or nothing. | ✓ |
| Partial — render must pass, cursor is soft | RTL render is hard. Cursor issues noted but don't block if minor. Slash menu/dictionary info-only. | |

**User's choice:** No — all 4 must pass
**Notes:** Strict bar. All 4 questions must pass for editable SUMM-04 path.

---

## Conversion Strategy & Ordering

### Q1: shadcn/ui primitive handling

| Option | Description | Selected |
|--------|-------------|----------|
| Convert in-place | Edit shadcn/ui files directly. They're vendored project code. | |
| Separate batch before hotspots | Convert all shadcn/ui primitives first, then hotspot files, then sweep. | |
| You decide the ordering | Claude picks optimal batching — all files end up converted. | ✓ |

**User's choice:** You decide the ordering
**Notes:** Delegated to Claude's discretion.

### Q2: Class mapping approach

| Option | Description | Selected |
|--------|-------------|----------|
| Strict mapping table | Mechanical 1:1 mapping (ml→ms, mr→me, etc.) applied uniformly. | |
| Contextual per-case | Some physical classes may be intentional. Evaluate each usage. | |
| Strict mapping + exception list | Mechanical default with explicit exception list for intentional physical classes. | |

**User's choice:** "you decide" (free text)
**Notes:** Delegated to Claude's discretion.

---

## Sidebar Collapse Animation

### Q1: Direction source

| Option | Description | Selected |
|--------|-------------|----------|
| Read from HTML dir attribute | Use document.documentElement.dir or useDirection() hook. Phase 2 already sets <html dir>. | |
| Read from locale context | Use locale from I18nProvider/ConfigContext and derive direction. | |
| You decide | Claude picks cleanest approach given codebase patterns. | ✓ |

**User's choice:** You decide
**Notes:** Delegated to Claude's discretion.

---

## ESLint Rule Design

### Q1: Exception mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| eslint-disable comments | Inline comments for legitimate usages. Each exception documented at call site. | |
| Allowlist in ESLint config | Exception patterns maintained in ESLint config (e.g., translate-x-* always allowed). | |
| You decide | Claude picks approach balancing strictness with maintainability. | ✓ |

**User's choice:** You decide
**Notes:** Delegated to Claude's discretion.

### Q2: Enforcement level

| Option | Description | Selected |
|--------|-------------|----------|
| CI + local (error) | Error in both CI and local dev. Red squiggles immediately. Strictest. | ✓ |
| CI error, local warning | Warning locally, error in CI. Less friction during development. | |
| CI error only | Only in CI. No local interference. Violations discovered at PR time. | |

**User's choice:** CI + local (error)
**Notes:** Strictest enforcement chosen.

---

## Claude's Discretion

The following areas were delegated to Claude's judgment:
- D-05: Conversion batching and ordering strategy
- D-06: shadcn/ui primitive conversion ordering
- D-07: Class mapping approach (strict table, contextual, or hybrid)
- D-08: Sidebar direction detection mechanism
- D-12: ESLint exception mechanism

## Deferred Ideas

None — discussion stayed within phase scope.
