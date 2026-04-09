---
phase: 3
slug: rtl-layout-conversion
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | ESLint 9.x (lint rules) + manual visual inspection (RTL layout) |
| **Config file** | `frontend/eslint.config.mjs` (ESLint) |
| **Quick run command** | `cd frontend && npx eslint src/ --rule 'no-restricted-syntax: error'` |
| **Full suite command** | `cd frontend && pnpm run lint` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && pnpm run lint`
- **After every plan wave:** Run full lint suite + visual spot-check of modified components
- **Before `/gsd-verify-work`:** Full lint must be green + manual RTL pass on all 10 hotspot files
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 3-01-01 | 01 | 1 | UI-05 | — | N/A | spike | BlockNote RTL spike decision document | N/A | ⬜ pending |
| 3-02-01 | 02 | 1 | QA-07 | — | N/A | lint | `cd frontend && pnpm run lint` | ❌ W0 | ⬜ pending |
| 3-03-01 | 03 | 2 | UI-05 | — | N/A | lint+visual | `cd frontend && pnpm run lint` + visual RTL check | ✅ | ⬜ pending |
| 3-03-02 | 03 | 2 | UI-06 | — | N/A | lint+visual | `cd frontend && pnpm run lint` + sidebar collapse test | ✅ | ⬜ pending |
| 3-04-01 | 04 | 3 | UI-05 | — | N/A | lint | `cd frontend && pnpm run lint` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] ESLint `no-restricted-syntax` rule for physical-direction Tailwind classes — installed in Plan 02 (QA-07)
- [ ] No new test framework needed — lint-based validation

*Existing infrastructure covers automated verification. Visual RTL checks are manual-only.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| All 10 hotspot files render as proper RTL mirrors | UI-05 | Visual layout cannot be verified by lint | Switch to Arabic locale, navigate to each hotspot screen, verify no visual asymmetry |
| Sidebar collapse animates toward right edge in Arabic | UI-06 | Animation direction requires visual verification | In Arabic mode: collapse sidebar → verify slides right; in English: verify slides left |
| BlockNote editor displays Arabic text correctly | UI-05 | Rich text editor RTL behavior requires runtime testing | Open summary editor in Arabic mode, type Arabic text, verify cursor and text flow |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
