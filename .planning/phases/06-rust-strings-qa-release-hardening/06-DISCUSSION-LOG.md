# Phase 6: Rust Strings, QA & Release Hardening - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-13
**Phase:** 06-rust-strings-qa-release-hardening
**Areas discussed:** Tray & notification i18n, Regression test strategy, Manual RTL pass scope, Quality spot-checks

---

## Tray & Notification i18n

| Option | Description | Selected |
|--------|-------------|----------|
| Rust constants | Parallel const arrays in tray.rs and notifications/types.rs. Simple, zero-dep, matches include_str! pattern. | ✓ |
| JSON message files | Load from messages/en.json and messages/ar.json. Single source of truth with frontend but adds I/O. | |
| Rust i18n crate | Use rust-i18n or fluent-rs. More scalable but adds dependency for ~23 strings. | |

**User's choice:** Rust constants (Recommended)
**Notes:** Zero new dependencies, matches existing Phase 5 patterns.

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri event listener | preferences::set emits 'locale-changed' event, tray.rs listens and rebuilds menu. | ✓ |
| Poll on tray interaction | Read preferences on each tray open. Simpler but stale until clicked. | |
| You decide | Claude picks based on existing infrastructure. | |

**User's choice:** Tauri event listener (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| Keep same emojis | Emojis are language-neutral, only text labels change. | ✓ |
| Remove emojis in Arabic | Cleaner look, avoids potential RTL emoji misplacement. | |
| You decide | Claude checks macOS tray RTL rendering. | |

**User's choice:** Keep same emojis

---

## Regression Test Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Rust #[cfg(test)] | Unit tests inside existing modules. Extends Phase 1's 5-test pattern. | ✓ |
| Rust integration tests | Separate tests/ directory with full app state setup. | |
| Both layers | Unit tests + integration tests for cross-module flows. | |

**User's choice:** Rust #[cfg(test)] (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| Reject branch only | Test set_user_preferences rejects parakeet when ar. Add en→ar repoint edge cases. | ✓ |
| Full UI + API surface | Also test frontend dropdown hiding and onboarding. Requires Vitest. | |
| You decide | Claude determines minimum tests for QA-02. | |

**User's choice:** Reject branch only (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| 3-case matrix | AR present→AR, AR missing+EN present→EN fallback, both missing→error. Both get_template() and get_prompt(). | ✓ |
| Full locale × template matrix | 6 templates × 2 locales × 3 tiers + 5 prompts × 2 locales. ~50+ test cases. | |
| You decide | Claude picks based on Phase 5 existing coverage gaps. | |

**User's choice:** 3-case matrix (Recommended)

---

## Manual RTL Pass Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Markdown checklist | QA-04-RTL-PASS.md with pass/fail per screen + screenshot if fail. | ✓ |
| Structured test report | Formal report with test steps, expected/actual behavior, severity. | |
| You decide | Claude picks format for QA-04. | |

**User's choice:** Markdown checklist (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| No overflow or clipping | Pass if no overflow, clipping, or asymmetry. Minor spacing OK. | ✓ |
| Pixel-perfect mirroring | Every element must be perfect mirror of English layout. | |
| Functional correctness only | Pass if features work and text is readable. Visual imperfections OK. | |

**User's choice:** No overflow or clipping (Recommended)

---

## Quality Spot-Checks

| Option | Description | Selected |
|--------|-------------|----------|
| User-provided MSA samples | 1-2 short MSA Arabic audio clips (30-60s). Test with Whisper large-v3. | ✓ |
| Public MSA datasets | Common Voice Arabic or similar. Reproducible but not meeting-like. | |
| You decide | Claude finds appropriate MSA test audio. | |

**User's choice:** User-provided MSA samples (Recommended)

| Option | Description | Selected |
|--------|-------------|----------|
| Claude + Ollama (both required) | Test both providers per requirement. Arabic template + transcript + prompt. | ✓ |
| Claude only for now | Test Claude primary, Ollama opportunistically. | |
| You decide | Claude determines testable providers in dev environment. | |

**User's choice:** Claude + Ollama (both required)

| Option | Description | Selected |
|--------|-------------|----------|
| Inline in QA checklist | Add QA-05/QA-06 to same QA-04-RTL-PASS.md file. One file for all manual QA. | ✓ |
| Separate quality report | Dedicated QA-QUALITY-REPORT.md with detailed metrics. | |
| You decide | Claude picks format for acceptance criteria. | |

**User's choice:** Inline in QA checklist (Recommended)

---

## Claude's Discretion

- Rust constant array structure (struct vs tuple vs enum)
- Exact Tauri event name for locale-changed
- QA-01 specific test case design
- QA checklist section organization

## Deferred Ideas

None — discussion stayed within phase scope.
