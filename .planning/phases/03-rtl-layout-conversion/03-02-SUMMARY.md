---
phase: 03-rtl-layout-conversion
plan: 02
status: complete
started: 2026-04-09
completed: 2026-04-09
---

# Plan 03-02 Summary: ESLint RTL Guardrail

## Objective
Add ESLint `no-restricted-syntax` rule banning physical-direction Tailwind classes in `.tsx` files (QA-07).

## What Was Built
- **ESLint guardrail rule** in `frontend/eslint.config.mjs` that catches:
  - `ml-*`, `mr-*`, `pl-*`, `pr-*` (physical margin/padding)
  - `text-left`, `text-right` (physical text alignment)
  - `border-l-*`, `border-r-*` (physical borders)
  - `rounded-l-*`, `rounded-r-*` (physical border-radius)
- Covers 4 AST selector patterns: className Literals, TemplateLiterals, `cn()`, `clsx()`
- Severity: **error** (not warning)
- Scope: `src/**/*.tsx` files only

## Commits
| SHA | Message |
|-----|---------|
| 0382c29 | feat(03-02): add ESLint no-restricted-syntax rule banning physical-direction Tailwind classes [QA-07] |

## Requirements Satisfied
- **QA-07**: ESLint guardrail preventing new physical-direction Tailwind classes

## Deviations
None.

## Notes
- Legitimate physical-direction usages (e.g., `translate-x`) can be exempted via `eslint-disable` comments
- This rule must be in place BEFORE any conversion work starts (D-05, D-13)
