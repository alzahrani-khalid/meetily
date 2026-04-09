# Phase 3: RTL Layout Conversion - Research

**Researched:** 2026-04-09
**Domain:** RTL layout conversion, Tailwind CSS logical properties, BlockNote RTL, ESLint guardrails
**Confidence:** HIGH

## Summary

Phase 3 converts all 47 `.tsx` files (122 physical-direction class occurrences) from physical Tailwind classes to logical equivalents, adds an ESLint guardrail to prevent regressions, fixes the sidebar collapse animation for bidirectional support, and runs a BlockNote RTL spike to gate SUMM-04's editable vs read-only decision.

The research reveals strong evidence that the BlockNote spike will likely succeed: BlockNote 0.36.0 ships with a built-in Arabic locale (`ar.ts`) containing full dictionary translations for slash menu, formatting toolbar, side menu, placeholders, and drag handle. The `dictionary` option on `useCreateBlockNote` accepts this locale directly. The editor inherits `direction` from the parent DOM's `dir` attribute (via ProseMirror's CSS). The Inter font import is the one known issue -- it lacks Arabic glyphs and must be replaced or supplemented.

Tailwind CSS 3.4.19 (installed) natively supports all logical property utilities (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`, `border-s-*`, `border-e-*`, `rounded-s-*`, `rounded-e-*`) and the `rtl:` variant. No plugins needed. The `space-x-*` utility requires adding `rtl:space-x-reverse` as a companion class in 19 files (39 occurrences).

**Primary recommendation:** Execute the spike first to lock SUMM-04's path, then land the ESLint rule, then batch-convert files from highest-occurrence hotspots outward.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** The spike tests ALL 4 questions from spec section 7: (1) Does BlockNote render RTL text correctly? (2) Does the slash menu work in RTL? (3) Can the `dictionary` prop localize block labels? (4) Is cursor behavior correct in RTL?
- **D-02:** The pass bar is strict -- **all 4 questions must pass** for SUMM-04 to ship as editable. If ANY question fails, SUMM-04 falls back to read-only rendered markdown (spec section 12.3 red path). Slash menu and dictionary failures are NOT deferred to v2 for the pass/fail decision -- they block the editable path.
- **D-03:** Spike deliverable is a written decision doc + a minimal test page (kept in repo as evidence) + a screenshot showing the RTL render result. All three artifacts live in the phase directory.
- **D-04:** The spike is the FIRST plan in this phase. No other RTL conversion work begins until the spike decision is locked.
- **D-09:** The sidebar must collapse toward the RIGHT edge in Arabic mode and toward the LEFT edge in English mode. This is the only place where logical properties don't apply -- `translate-x` has no logical equivalent. Explicit testing in both directions is required.
- **D-10:** The rule bans: `ml-*`, `mr-*`, `pl-*`, `pr-*`, `text-left`, `text-right`, `border-l-*`, `border-r-*`, `rounded-l-*`, `rounded-r-*` in all `frontend/src/**/*.tsx` files.
- **D-11:** The rule is an **error** in both CI and local development.
- **D-13:** The ESLint rule lands as its own plan, BEFORE the hotspot conversion plan starts.

### Claude's Discretion
- **D-05:** Optimal batching and ordering of file conversions
- **D-06:** shadcn/ui primitive conversion ordering (separate batch, inline with hotspots, or other)
- **D-07:** Mapping strategy (strict 1:1, contextual, or hybrid with exception list)
- **D-08:** How sidebar reads current direction for `translate-x` branching
- **D-12:** Exception mechanism for legitimate physical-direction usages (eslint-disable comments vs allowlist)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| UI-05 | All layout elements mirror correctly in RTL across 65 .tsx files / 286 directional hit surfaces | Tailwind 3.4.19 logical properties verified; 1:1 mapping table documented; 47 files / 122 physical-direction hits confirmed by current grep (lower than spec's 286 because some were already converted or are `space-x` which needs companion class, not replacement) |
| UI-06 | Sidebar collapse animation works in both directions (`translate-x-full` vs `-translate-x-full` branched on `dir`) | Sidebar code at line 664 confirmed; `translate-x` has no logical equivalent; spec section 4.4 provides the branching pattern |
| QA-07 | ESLint `no-restricted-syntax` rule prevents new physical-direction Tailwind classes | ESLint flat config (`eslint.config.mjs`) is minimal and ready for rule addition; `no-restricted-syntax` AST selector pattern documented below |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tailwindcss | 3.4.19 | Logical property utilities (`ms-*`, `me-*`, etc.) and `rtl:` variant | Already installed; native support for all needed logical properties since v3.3 [VERIFIED: node_modules] |
| @blocknote/core | 0.36.0 | Rich text editor with built-in Arabic locale | Already installed; ships `ar` dictionary with full slash menu, toolbar, and placeholder translations [VERIFIED: node_modules source] |
| @blocknote/react | 0.36.0 | React bindings for BlockNote (`useCreateBlockNote` accepts `dictionary` option) | Already installed [VERIFIED: node_modules] |
| @blocknote/shadcn | 0.36.0 | shadcn-styled BlockNote UI components | Already installed [VERIFIED: node_modules] |
| eslint | (existing) | Flat config with `@eslint/eslintrc` compat layer | Already configured at `frontend/eslint.config.mjs` [VERIFIED: codebase] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tailwindcss-animate | ^1.0.7 | Animation utilities (already installed) | Used by shadcn/ui components; no changes needed |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `no-restricted-syntax` ESLint rule | eslint-plugin-tailwindcss | Heavier dependency; `no-restricted-syntax` is built into ESLint and sufficient for class-name string matching |
| Manual `space-x-reverse` companion | `gap` utility | Would require layout restructuring; `rtl:space-x-reverse` is the standard non-breaking approach |

## Architecture Patterns

### Conversion Mapping Table
| Before | After | Notes |
|--------|-------|-------|
| `ml-*` | `ms-*` | margin-inline-start |
| `mr-*` | `me-*` | margin-inline-end |
| `pl-*` | `ps-*` | padding-inline-start |
| `pr-*` | `pe-*` | padding-inline-end |
| `left-0` | `start-0` | inset-inline-start |
| `right-0` | `end-0` | inset-inline-end |
| `text-left` | `text-start` | text-align logical |
| `text-right` | `text-end` | text-align logical |
| `border-l-*` | `border-s-*` | border-inline-start |
| `border-r-*` | `border-e-*` | border-inline-end |
| `rounded-l-*` | `rounded-s-*` | border-radius logical start |
| `rounded-r-*` | `rounded-e-*` | border-radius logical end |
| `space-x-*` | `space-x-* rtl:space-x-reverse` | Add companion class |

[VERIFIED: Tailwind docs and spec section 4.3]

### Pattern 1: BlockNote Arabic Configuration
**What:** Configure BlockNote with Arabic locale and proper font
**When to use:** When rendering the summary editor in Arabic mode
**Example:**
```typescript
// Source: BlockNote 0.36.0 source (node_modules/@blocknote/core/src/editor/BlockNoteEditor.ts)
import { ar } from "@blocknote/core/locales";

const editor = useCreateBlockNote({
  dictionary: ar,  // Full Arabic translations for slash menu, toolbar, placeholders
  initialContent: initialContent as PartialBlock[] | undefined,
});

// The editor inherits direction from <html dir="rtl"> via CSS inheritance
// Replace @blocknote/core/fonts/inter.css with project's Tajawal font
```
[VERIFIED: node_modules source inspection]

### Pattern 2: ESLint `no-restricted-syntax` for Tailwind Physical Classes
**What:** AST-based lint rule catching physical-direction classes in JSX string literals
**When to use:** In `eslint.config.mjs` targeting `frontend/src/**/*.tsx`
**Example:**
```javascript
// Source: ESLint docs for no-restricted-syntax + JSX string matching
{
  files: ["src/**/*.tsx"],
  rules: {
    "no-restricted-syntax": ["error",
      {
        selector: "JSXAttribute[name.name='className'] Literal[value=/\\b(m[lr]-|p[lr]-|text-(?:left|right)|border-[lr]-|rounded-[lr]-)/]",
        message: "Use logical properties (ms-/me-/ps-/pe-/text-start/text-end/border-s-/border-e-/rounded-s-/rounded-e-) instead of physical direction classes for RTL support."
      }
    ]
  }
}
```
**Important caveat:** `no-restricted-syntax` operates on AST nodes. Template literals (backtick strings) and `cn()` / `clsx()` calls need a different selector: `TemplateLiteral` elements and `CallExpression[callee.name='cn'] Literal`. The rule may need multiple selector entries to cover all patterns. [ASSUMED -- exact AST selectors need validation during implementation]

### Pattern 3: Sidebar Collapse Direction Branching
**What:** Branch `translate-x` on document direction for correct collapse animation
**When to use:** Sidebar component collapse/expand animation
**Example:**
```typescript
// Source: spec section 4.4 + codebase (Sidebar/index.tsx:664)
// translate-x has no logical CSS equivalent -- must branch explicitly

// Option A: Read from DOM
const dir = document.documentElement.dir;
const collapseTransform = dir === 'rtl'
  ? (isCollapsed ? 'translate-x-full' : 'translate-x-0')
  : (isCollapsed ? '-translate-x-full' : 'translate-x-0');

// Option B: Read from locale context (useLocale from I18nProvider)
const locale = useLocale();
const isRTL = locale === 'ar';
```
[VERIFIED: Sidebar code at line 664 confirmed `fixed top-0 left-0` positioning]

### Anti-Patterns to Avoid
- **Double-flip with `.reverse()`:** `forceRTL` / `dir="rtl"` already handles visual direction. Manual array `.reverse()` causes LTR rendering in RTL mode. (CLAUDE.md Rule 4)
- **`textAlign: "right"` or `text-right`:** Gets flipped by RTL to physical LEFT. Use `text-end` or `writingDirection: "rtl"` instead. (CLAUDE.md Rule 3)
- **Mixing physical and logical in same element:** Leads to unpredictable behavior. Convert ALL directional classes on an element at once.
- **Forgetting `rtl:space-x-reverse`:** `space-x-*` uses `margin-left` under the hood. Without the reverse companion, child spacing breaks in RTL.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| RTL text direction for BlockNote | Custom CSS overrides for editor direction | `<html dir="rtl">` inheritance + BlockNote's built-in CSS | ProseMirror (BlockNote's engine) respects inherited `direction` from parent DOM |
| Arabic editor labels | Manual string replacements | `import { ar } from "@blocknote/core/locales"` + `dictionary: ar` option | BlockNote ships complete Arabic translations for all UI strings |
| Tailwind RTL support | Custom PostCSS plugin | Built-in logical properties (`ms-*`, `me-*`, etc.) + `rtl:` variant | Tailwind 3.3+ has native support; no plugins needed |
| ESLint class-name validation | Custom ESLint plugin | `no-restricted-syntax` with AST selectors | Built-in ESLint rule; zero dependencies |

## Common Pitfalls

### Pitfall 1: `space-x-*` Not Reversing in RTL
**What goes wrong:** Horizontal spacing between children appears on the wrong side in RTL mode
**Why it happens:** `space-x-*` compiles to `margin-left` on non-first children, which is a physical property. Unlike `ml-*` which has a logical equivalent `ms-*`, `space-x` has no logical counterpart.
**How to avoid:** Always add `rtl:space-x-reverse` as a companion class: `space-x-2 rtl:space-x-reverse`
**Warning signs:** 39 occurrences across 19 files need this treatment

### Pitfall 2: Template Literals and `cn()` Bypassing ESLint
**What goes wrong:** The `no-restricted-syntax` rule catches string literals in `className="..."` but misses classes inside template literals or `cn()` / `clsx()` utility calls
**Why it happens:** Different AST node types. A `Literal` selector doesn't match `TemplateLiteral` or arguments inside function calls.
**How to avoid:** Add multiple selector entries covering: (1) JSX string literals, (2) template literal quasi strings, (3) arguments in `cn()` / `clsx()` calls. Test the rule against all three patterns.
**Warning signs:** New physical classes passing lint in code that uses `cn()` for conditional classes

### Pitfall 3: Inter Font Import Breaking Arabic in BlockNote
**What goes wrong:** Arabic text in the BlockNote editor renders with missing glyphs or fallback font
**Why it happens:** `Editor.tsx:6` imports `@blocknote/core/fonts/inter.css` -- Inter is a Latin/Cyrillic font with no Arabic coverage
**How to avoid:** Remove the Inter font import and let the editor inherit the project's Tajawal font from the global CSS (`html[dir="rtl"] body { font-family: var(--font-sans-ar); }`)
**Warning signs:** Tofu characters or serif fallback in the editor while the rest of the app renders Tajawal correctly

### Pitfall 4: Sidebar Positioning `left-0` Not Converting
**What goes wrong:** Sidebar stays pinned to the physical left edge in RTL mode
**Why it happens:** `fixed top-0 left-0` at line 664 uses physical positioning
**How to avoid:** Convert to `fixed top-0 start-0` (logical equivalent)
**Warning signs:** Sidebar overlapping content area on the wrong side in Arabic mode

### Pitfall 5: Collapse Button `-right-6` Positioning
**What goes wrong:** The floating collapse button at line 667 (`-right-6`) stays on the physical right edge instead of flipping
**Why it happens:** Physical positioning class
**How to avoid:** Convert to `-end-6` or equivalent logical positioning
**Warning signs:** Collapse button appearing inside the sidebar panel instead of outside its edge in one direction

### Pitfall 6: Chevron Icons Not Flipping
**What goes wrong:** Collapse/expand chevron icons point the wrong direction in RTL
**Why it happens:** `ChevronRightCircle` (line 672) and `ChevronLeftCircle` (line 674) have hardcoded directional names
**How to avoid:** Swap icon assignments based on direction (CLAUDE.md Rule 5: chevron-left = "drill-in" in RTL, chevron-right = "return" in RTL)
**Warning signs:** Visual contradiction between icon direction and animation direction

## Code Examples

### BlockNote Arabic Spike Test Page
```typescript
// Source: BlockNote 0.36.0 source + project codebase patterns
"use client";

import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import "@blocknote/shadcn/style.css";
// NOTE: Do NOT import @blocknote/core/fonts/inter.css -- use project font
import { ar } from "@blocknote/core/locales";

const arabicTestContent = "هذا نص تجريبي لاختبار دعم اللغة العربية في محرر BlockNote";

export default function BlockNoteRTLSpike() {
  const editor = useCreateBlockNote({
    dictionary: ar,
  });

  return (
    <div dir="rtl" style={{ fontFamily: "var(--font-sans-ar)" }}>
      <BlockNoteView editor={editor} editable={true} theme="light" />
    </div>
  );
}
```
[VERIFIED: `dictionary` option and `ar` locale confirmed in source]

### ESLint Rule Configuration
```javascript
// Source: ESLint no-restricted-syntax docs + Tailwind class patterns
// Add to frontend/eslint.config.mjs
{
  files: ["src/**/*.tsx"],
  rules: {
    "no-restricted-syntax": ["error",
      {
        selector: "Literal[value=/\\b(m[lr]-|p[lr]-|text-left|text-right|border-[lr]-|rounded-[lr]-)/]",
        message: "Use logical Tailwind properties (ms-/me-/ps-/pe-/text-start/text-end/border-s-/border-e-/rounded-s-/rounded-e-) for RTL support. See spec section 4.3."
      }
    ]
  }
}
```
[ASSUMED -- AST selector regex needs validation; may need separate selectors for template literals and cn() calls]

### Sidebar Direction-Aware Collapse
```typescript
// Source: spec section 4.4 + Sidebar/index.tsx:664
// Before:
<div className="fixed top-0 left-0 h-screen z-40">

// After:
<div className="fixed top-0 start-0 h-screen z-40">

// Collapse button position: -right-6 -> -end-6
// Chevron icons: swap based on dir
const locale = useLocale();
const isRTL = locale === 'ar';
{isCollapsed ? (
  isRTL ? <ChevronLeftCircle className="w-6 h-6" /> : <ChevronRightCircle className="w-6 h-6" />
) : (
  isRTL ? <ChevronRightCircle className="w-6 h-6" /> : <ChevronLeftCircle className="w-6 h-6" />
)}
```
[VERIFIED: codebase lines 664-675]

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `ml-*`/`mr-*` physical margins | `ms-*`/`me-*` logical margins | Tailwind 3.3 (2023) | Direct replacement; zero behavior change in LTR |
| Custom RTL Tailwind plugins | Native `rtl:` variant + logical properties | Tailwind 3.1+ | No plugin needed; built into core |
| BlockNote without i18n | Built-in `dictionary` option with 20+ locales | BlockNote 0.33+ | Arabic locale ships in `@blocknote/core/locales` |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | ESLint `no-restricted-syntax` AST selector can match Tailwind class patterns in string literals, template literals, and `cn()` calls with a regex-based selector | ESLint Rule Pattern | Medium -- if regex matching doesn't work on AST Literal nodes, may need a custom ESLint rule or eslint-plugin-tailwindcss; fallback is straightforward |
| A2 | BlockNote 0.36.0 inherits `direction: rtl` from parent DOM via ProseMirror CSS without explicit configuration | BlockNote RTL | Low -- ProseMirror is known to respect inherited CSS direction; the spike will verify this definitively |
| A3 | Removing `@blocknote/core/fonts/inter.css` import won't break non-Arabic text rendering in BlockNote | BlockNote Font | Low -- the project's global font stack (Source Sans 3 for LTR, Tajawal for RTL) should cover all scripts the editor needs |

## Open Questions

1. **BlockNote cursor behavior in RTL**
   - What we know: BlockNote uses ProseMirror which generally handles RTL cursor movement. The `ar` locale exists with full translations.
   - What's unclear: Whether cursor navigation (arrow keys, Home/End) works correctly in RTL blocks. This is spike question #4.
   - Recommendation: The spike test page must include multi-line Arabic content to test cursor movement thoroughly.

2. **`cn()` / `clsx()` ESLint coverage**
   - What we know: `no-restricted-syntax` can match `Literal` nodes. Many components use `cn()` for conditional class merging.
   - What's unclear: Whether a single regex-based AST selector can catch classes inside `cn("ml-2", condition && "mr-4")` call arguments.
   - Recommendation: Test the ESLint rule against `cn()` patterns during the ESLint plan implementation. If gaps found, add extra selectors or use `eslint-plugin-tailwindcss` as fallback.

3. **Occurrence count discrepancy (122 vs 286)**
   - What we know: Current grep finds 122 physical-direction hits across 47 files. Spec says 286 across 65 files.
   - What's unclear: The difference may be due to Phase 2 conversions already done, `space-x-*` counted separately, or `left-0`/`right-0` positioning classes included in spec's broader pattern.
   - Recommendation: Run the full spec grep pattern (including `left-|right-|space-x-`) for accurate current count at plan time. The 47 files / 122 hits figure is the conservative baseline for the `ml-|mr-|pl-|pr-|text-left|text-right|border-l-|border-r-|rounded-l-|rounded-r-` subset.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | ESLint (static analysis, not runtime tests) |
| Config file | `frontend/eslint.config.mjs` |
| Quick run command | `cd frontend && npx eslint src/**/*.tsx --quiet` |
| Full suite command | `cd frontend && npx eslint src/ --ext .tsx` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| UI-05 | All layout elements mirror correctly in RTL | manual (visual) + static lint | `cd frontend && npx eslint src/ --ext .tsx` (catches regressions) | eslint.config.mjs exists, rule not yet added -- Wave 0 |
| UI-06 | Sidebar collapse animation correct in both directions | manual (visual) | Manual: toggle sidebar in ar and en modes | N/A -- manual only |
| QA-07 | ESLint prevents new physical-direction classes | unit (lint rule) | `cd frontend && npx eslint src/**/*.tsx --quiet` | eslint.config.mjs exists, rule not yet added -- Wave 0 |

### Wave 0 Gaps
- [ ] ESLint `no-restricted-syntax` rule entries in `frontend/eslint.config.mjs` -- covers QA-07
- [ ] BlockNote RTL spike test page -- covers D-01 through D-04
- [ ] No runtime test framework needed -- this phase is CSS conversion + static analysis

## Security Domain

No security-relevant changes in this phase. RTL layout conversion is purely presentational (CSS class renaming). The ESLint rule is a development-time guardrail, not a runtime security control. BlockNote editor is already in the codebase; the spike only changes configuration (locale/dictionary), not trust boundaries.

## Project Constraints (from CLAUDE.md)

- **RTL-First Rules (CLAUDE.md global):** The 5 immutable RTL rules apply. Key for this phase: no `.reverse()` (Rule 4), no `text-right` (Rule 3 -- maps to no `text-right` Tailwind class), directional icon conventions (Rule 5 -- applies to sidebar chevrons).
- **Web/Tailwind RTL Requirements (CLAUDE.md):** Use logical properties ONLY: `ms-*`, `me-*`, `ps-*`, `pe-*`, `text-start`, `text-end`. NEVER use: `ml-*`, `mr-*`, `pl-*`, `pr-*`, `text-left`, `text-right`.
- **GSD Workflow:** Each plan's commits should be atomic and reference the REQ-ID they satisfy.
- **Commit granularity:** Standard granularity per `config.json`.

## Sources

### Primary (HIGH confidence)
- BlockNote 0.36.0 source code in `node_modules/@blocknote/core/src/` -- dictionary type, `ar.ts` locale, editor options [VERIFIED]
- Tailwind CSS 3.4.19 installed -- logical property utilities confirmed available [VERIFIED]
- ESLint flat config at `frontend/eslint.config.mjs` -- minimal config, ready for rule addition [VERIFIED]
- Sidebar/index.tsx lines 664-675 -- collapse animation, positioning, chevron icons [VERIFIED]
- Editor.tsx -- current BlockNote setup (no dictionary, Inter font import) [VERIFIED]
- Spec sections 4, 7, 11, 12.3 -- conversion rules, spike questions, phase structure, fallback strategy [VERIFIED]

### Secondary (MEDIUM confidence)
- BlockNote's ProseMirror engine respects CSS `direction` inheritance [ASSUMED based on ProseMirror architecture -- spike will verify]

### Tertiary (LOW confidence)
- ESLint `no-restricted-syntax` regex matching on `Literal` AST nodes for Tailwind class patterns [ASSUMED -- needs implementation validation]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all packages already installed and version-verified
- Architecture: HIGH -- conversion mapping is mechanical and well-documented in spec
- BlockNote RTL: MEDIUM -- strong evidence (built-in `ar` locale, `dictionary` option) but spike is required per D-01/D-02 to confirm runtime behavior
- ESLint rule: MEDIUM -- pattern is standard but exact AST selector syntax needs validation
- Pitfalls: HIGH -- all identified from codebase inspection

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable -- Tailwind and BlockNote versions locked for this milestone)
