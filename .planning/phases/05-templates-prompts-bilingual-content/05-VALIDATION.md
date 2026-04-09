---
phase: 5
slug: templates-prompts-bilingual-content
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-10
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust unit tests) + manual verification |
| **Config file** | `frontend/src-tauri/Cargo.toml` |
| **Quick run command** | `cd frontend && cargo test --lib summary` |
| **Full suite command** | `cd frontend && cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && cargo test --lib summary`
- **After every plan wave:** Run `cd frontend && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | TPL-01 | — | Locale fallback chain resolves correctly | unit | `cargo test template_loader` | ❌ W0 | ⬜ pending |
| 5-01-02 | 01 | 1 | TPL-02 | — | All 12 templates (6×2) embedded via include_str! | unit | `cargo test builtin_templates` | ❌ W0 | ⬜ pending |
| 5-02-01 | 02 | 2 | TPL-03 | — | 5 prompts externalized from processor.rs | unit | `cargo test prompt_loader` | ❌ W0 | ⬜ pending |
| 5-02-02 | 02 | 2 | TPL-04 | — | All 10 prompt files (5×2) embedded via include_str! | unit | `cargo test builtin_prompts` | ❌ W0 | ⬜ pending |
| 5-03-01 | 03 | 3 | SUMM-01 | — | Summary language independent from UI locale | integration | `cargo test summary_locale` | ❌ W0 | ⬜ pending |
| 5-03-02 | 03 | 3 | SUMM-02 | — | All 6 templates available in both locales | unit | `cargo test template_coverage` | ❌ W0 | ⬜ pending |
| 5-03-03 | 03 | 3 | SUMM-03 | — | Arabic prompts include punctuation directives | grep | `grep -l '،' frontend/src-tauri/prompts/*ar*` | ❌ W0 | ⬜ pending |
| 5-04-01 | 04 | 4 | SUMM-04 | — | BlockNote renders Arabic summary with RTL | manual | Visual inspection in Arabic mode | N/A | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Template loader locale fallback tests — stubs for TPL-01 locale resolution
- [ ] Prompt loader tests — stubs for TPL-03/TPL-04 externalization
- [ ] Template coverage assertion — all 6 × 2 locales present in defaults

*Existing `cargo test` infrastructure covers compilation and basic module tests.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Arabic summary renders RTL in BlockNote | SUMM-04 | Requires visual inspection of RTL rendering | 1. Set summary_language='ar' 2. Generate summary 3. Verify RTL text direction and Arabic font in BlockNote editor |
| Arabic template content quality (MSA register) | SUMM-03 | Requires native speaker review | 1. Read each Arabic template JSON 2. Verify formal MSA, professional tone 3. Check Arabic punctuation used throughout |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
