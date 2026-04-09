---
phase: 4
slug: arabic-transcription-policy
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-09
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | vitest (frontend) + cargo test (Rust) |
| **Config file** | `frontend/vitest.config.ts` |
| **Quick run command** | `cd frontend && pnpm vitest run --reporter=verbose` |
| **Full suite command** | `cd frontend && pnpm vitest run && cd src-tauri && cargo test` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cd frontend && pnpm vitest run --reporter=verbose`
- **After every plan wave:** Run full suite command
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| TBD | TBD | TBD | TRANS-01 | — | Whisper large-v3 enforced for Arabic | integration | TBD | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TRANS-02 | — | Parakeet hidden from Arabic UI | unit | TBD | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TRANS-03 | — | Non-blocking large-v3 download | integration | TBD | ❌ W0 | ⬜ pending |
| TBD | TBD | TBD | TRANS-04 | — | Atomic provider repoint on locale switch | unit | TBD | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Test stubs for TRANS-01..TRANS-04 verification
- [ ] Vitest config already exists — no new framework install needed

*Existing infrastructure covers framework requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Arabic transcript quality (~85-88% MSA) | TRANS-01 | Requires real Arabic audio sample + Whisper large-v3 model | Record MSA Arabic sample, verify transcript accuracy |
| Onboarding flow visual check | TRANS-03 | UI flow with download progress animation | Walk through Arabic onboarding, verify progress indicator |
| Parakeet absence in dropdown | TRANS-02 | Visual verification of filtered UI | Open TranscriptSettings in Arabic mode, confirm no Parakeet option |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
