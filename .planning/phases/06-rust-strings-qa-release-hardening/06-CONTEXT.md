# Phase 6: Rust Strings, QA & Release Hardening - Context

**Gathered:** 2026-04-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Every Rust-owned UI string (tray menu ~13 items, system notifications ~10 items) is localized in Arabic, every regression-prone surface has automated test coverage, and the Arabic experience is verified end-to-end against real audio and real LLM providers. This phase does NOT add new features — it hardens what Phases 1–5 built.

</domain>

<decisions>
## Implementation Decisions

### Tray & Notification i18n
- **D-01:** Translations stored as Rust constants — parallel `TRAY_STRINGS_EN` / `TRAY_STRINGS_AR` arrays in `tray.rs`, and equivalent constants in `notifications/types.rs`. No external JSON files, no new crate dependencies. Matches the `include_str!` pattern from Phase 5 prompts.
- **D-02:** Tray menu re-hydrates on locale change via a Tauri event listener. `preferences::set` emits a `locale-changed` event; `tray.rs` listens and calls `rebuild_menu()` with the new locale. Same pattern as frontend event listeners.
- **D-03:** Emoji prefixes (⏸ ⏹ 🔄 ▶) stay the same in both English and Arabic — emojis are language-neutral. Only the text labels change.

### Regression Test Strategy
- **D-04:** All automated tests (QA-01, QA-02, QA-03) use Rust `#[cfg(test)]` unit tests inside existing modules. Phase 1 already has 5 tests in `preferences/tests.rs` — Phase 6 extends the same pattern. No separate integration test directory needed.
- **D-05:** QA-02 (Parakeet ban) tests the reject branch only — `set_user_preferences` rejects `provider:'parakeet'` when `ui_locale='ar'`, plus edge cases for `en→ar` locale switch auto-repointing provider. No frontend component tests for the dropdown hiding.
- **D-06:** QA-03 (template/prompt fallback) uses the 3-case matrix from the requirement: AR file present → AR returned, AR missing + EN present → EN fallback, both missing → error. Covers both `get_template()` and `get_prompt()`. Not the full 6×2×3 matrix.

### Manual RTL Pass
- **D-07:** Manual RTL regression documented as a Markdown checklist in a single `QA-04-RTL-PASS.md` file. One section per screen (Sidebar, Settings, Transcript, Summary, Onboarding, Tray, Meeting Details). Each item: pass/fail + screenshot if fail.
- **D-08:** Pass/fail bar: no text overflow, no clipping, no visual asymmetry, sidebar animation correct in both directions. Minor spacing inconsistencies acceptable. Not pixel-perfect mirroring — functional correctness with clean visual presentation.

### Quality Spot-Checks
- **D-09:** QA-05 (Arabic transcription) uses user-provided MSA Arabic audio samples (30-60s each). Tested with Whisper `large-v3`. Expected ~85-88% accuracy for MSA. Results documented in the same QA checklist file.
- **D-10:** QA-06 (Arabic summary quality) tests BOTH Claude and Ollama providers — both are required. Arabic template + Arabic transcript + Arabic prompt → fully Arabic RTL-formatted output from each provider.
- **D-11:** QA-05 and QA-06 results documented inline in the same `QA-04-RTL-PASS.md` checklist (renamed conceptually to "QA Manual Pass" covering RTL + transcription + summary). One file for all manual QA.

### Claude's Discretion
- Implementation details for how Rust constant arrays are structured (struct vs tuple vs enum)
- Exact Tauri event name for locale-changed notification
- QA-01 (preference desync) specific test case design — Claude determines what scenarios to cover based on Phase 1's existing T1–T5 tests
- Whether to merge QA-04/QA-05/QA-06 checklist sections or keep them as separate headings in the same file

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Spec
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` — Authoritative bilingual design. §5 (transcription policy), §8 (Rust strings), §10 (QA requirements), §12 (release hardening)

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — Phase 6 REQ-IDs: UI-07, QA-01, QA-02, QA-03, QA-04, QA-05, QA-06
- `.planning/ROADMAP.md` — Phase 6 success criteria (5 items)

### Prior Phase Contracts
- `.planning/phases/01-preferences-foundation/01-CONTEXT.md` — Parakeet-ban invariant (D-08/D-09/D-10), preferences::read() API, PREFS_CACHE pattern
- `.planning/phases/05-templates-prompts-bilingual-content/05-CONTEXT.md` — Template/prompt loader API signatures, include_str! embedding pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `preferences/tests.rs` (356 lines, 5 tests): Extend with QA-01/QA-02 test cases using same test infrastructure
- `preferences::read().ui_locale`: Single source of truth for locale — tray and notifications read from here
- `summary/templates/loader.rs::get_template(id, locale)`: Already locale-aware with 3-tier fallback — QA-03 tests target this
- `summary/prompts/` module: `get_prompt(id, locale)` with same fallback pattern — QA-03 tests also target this

### Established Patterns
- Tauri event emission: `app.emit("event-name", payload)` used throughout for Rust→frontend communication
- `include_str!` embedding: Used in `defaults.rs` for templates and `prompts/defaults.rs` for prompts — tray/notification strings follow the same compile-time embedding philosophy (but as const arrays, not file includes)
- `#[cfg(test)] mod tests` in same file: Established by Phase 1 in preferences module

### Integration Points
- `tray.rs:build_menu()` (line 316–392): 13 hardcoded English strings to localize
- `notifications/types.rs` helper constructors (lines 116–198): 10 hardcoded English strings to localize
- `lib.rs` startup sequence: Tray initialization reads locale after `preferences::hydrate_from_db()`
- Tauri event bus: New `locale-changed` event connects preferences writes to tray rebuild

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches for implementation details. User consistently preferred recommended/simplest options across all gray areas, indicating a preference for pragmatic, low-ceremony solutions.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 06-rust-strings-qa-release-hardening*
*Context gathered: 2026-04-13*
