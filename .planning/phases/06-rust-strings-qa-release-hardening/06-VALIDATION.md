---
phase: 6
slug: rust-strings-qa-release-hardening
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-15
---

# Phase 6 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust `#[cfg(test)]` (cargo test) |
| **Config file** | `frontend/src-tauri/Cargo.toml` |
| **Quick run command** | `cd frontend/src-tauri && cargo test --lib` |
| **Full suite command** | `cd frontend/src-tauri && cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend/src-tauri && cargo test --lib`
- **After every plan wave:** Run `cd frontend/src-tauri && cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 6-01-01 | 01 | 1 | UI-07 | — | Tray strings localized in AR/EN | unit | `cargo test tray` | ❌ W0 | ⬜ pending |
| 6-01-02 | 01 | 1 | UI-07 | — | Notification strings localized | unit | `cargo test notification` | ❌ W0 | ⬜ pending |
| 6-02-01 | 02 | 2 | QA-01 | T-1-02 | Preference desync detected | unit | `cargo test preferences` | ✅ | ⬜ pending |
| 6-02-02 | 02 | 2 | QA-02 | — | Parakeet ban enforced | unit | `cargo test parakeet` | ✅ | ⬜ pending |
| 6-02-03 | 02 | 2 | QA-03 | — | Template/prompt fallback works | unit | `cargo test template` | ❌ W0 | ⬜ pending |
| 6-03-01 | 03 | 3 | QA-04 | — | RTL regression pass | manual | — | — | ⬜ pending |
| 6-03-02 | 03 | 3 | QA-05 | — | Arabic transcription quality | manual | — | — | ⬜ pending |
| 6-03-03 | 03 | 3 | QA-06 | — | Arabic summary quality | manual | — | — | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Tray string test stubs in `tray.rs` `#[cfg(test)]` module
- [ ] Notification string test stubs in `notifications/types.rs` `#[cfg(test)]` module
- [ ] Template/prompt fallback test stubs in `summary/templates/loader.rs` and `summary/prompts/` modules

*Existing infrastructure (preferences/tests.rs with T1–T5) covers QA-01 and QA-02 base cases.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| RTL visual regression | QA-04 | Visual layout verification requires human judgment | Walk Sidebar, Settings, Transcript, Summary, Onboarding, Tray, Meeting Details in Arabic mode. Check: no overflow, no clipping, sidebar animation correct |
| Arabic transcription accuracy | QA-05 | Requires real audio + Whisper model | Record MSA Arabic audio (30-60s), transcribe with large-v3, verify ~85-88% accuracy |
| Arabic summary quality | QA-06 | Requires real LLM providers | Generate Arabic summary with Claude and Ollama using AR template + AR transcript + AR prompt, verify fully Arabic RTL output |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
