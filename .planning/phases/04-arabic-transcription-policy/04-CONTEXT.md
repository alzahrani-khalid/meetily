---
phase: 4
slug: arabic-transcription-policy
status: ready-for-research
gathered: 2026-04-09
milestone: v1.0 — Arabic Bilingual Support
requirements: [TRANS-01, TRANS-02, TRANS-03, TRANS-04]
upstream_artifacts:
  - .planning/phases/01-preferences-foundation/01-CONTEXT.md
  - .planning/phases/02-i18n-framework-locale-bootstrap/02-CONTEXT.md
  - .planning/phases/03-rtl-layout-conversion/03-CONTEXT.md
  - .planning/PROJECT.md
  - .planning/REQUIREMENTS.md
  - .planning/ROADMAP.md
  - docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md §5, §6, §12.2
---

# Phase 4: Arabic Transcription Policy — Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

An Arabic-locale user gets accurate Arabic transcription via Whisper `large-v3` and never encounters Parakeet anywhere in the product. This phase delivers three outcomes:

1. **Parakeet UI filtering (TRANS-02)** — Hide the Parakeet provider from `TranscriptSettings` when `uiLocale === 'ar'` and display an informational banner explaining why Whisper large-v3 is used.
2. **Arabic onboarding fork (TRANS-03)** — When `uiLocale === 'ar'`, onboarding completes immediately after permissions and downloads Whisper `large-v3` in the background instead of Parakeet. A "ready to record" gate prevents recording until the model is available.
3. **Locale switch provider repoint (TRANS-04)** — Switching `uiLocale` from `en` to `ar` atomically repoints the transcript provider to `localWhisper` + `large-v3` via the existing `set_user_preferences` invariant, with user-visible confirmation and automatic model download if needed.

**Requirements covered:** TRANS-01, TRANS-02, TRANS-03, TRANS-04

**Phase 1 foundation this phase consumes:**
- `set_user_preferences` Parakeet-ban invariant (A1 Option B) — `provider: 'parakeet'` + `ui_locale: 'ar'` is rejected atomically before SQLite is touched
- `UserPreferencesPatch.provider` field and `transcript_settings::force_provider` write surface already owned by Phase 1
- Commit-then-cache ordering (D-07) for atomic preference writes

**Phase 2 surface this phase consumes:**
- `uiLocale` from preferences, available via `I18nProvider` / `useLocale()`
- `LanguageConfirmDialog` component for locale switch confirmation
- `navigator.language` detection for initial locale

**Fixed out-of-scope for this phase:**
- The Parakeet-ban enforcement in `set_user_preferences` (already shipped in Phase 1)
- Template/prompt locale resolution (Phase 5)
- Transcription quality spot-checks (Phase 6 QA-05)
- Full preference desync regression suite (Phase 6 QA-01)

</domain>

<decisions>
## Implementation Decisions

### Parakeet UI Filtering (TRANS-02)
- **D-01:** Parakeet is **completely hidden** from the provider dropdown in `TranscriptSettings.tsx` when `uiLocale === 'ar'`. Not disabled — removed entirely from the list. The provider type union (`'localWhisper' | 'parakeet' | ...`) stays unchanged in TypeScript; only the rendered options are filtered.
- **D-02:** An **informational banner** appears **above the provider section** in TranscriptSettings when `uiLocale === 'ar'`, explaining: "يستخدم Meetily نموذج Whisper large-v3 للنسخ العربي لأنه يوفر أعلى دقة للغة العربية" (Meetily uses Whisper large-v3 for Arabic transcription because it provides the highest accuracy for Arabic). The banner uses the existing informational/note styling pattern.

### Arabic Onboarding Fork (TRANS-03)
- **D-03:** When `uiLocale === 'ar'`, onboarding **completes immediately** after the permissions step. No blocking download step inside the onboarding wizard. Whisper `large-v3` download starts automatically in the background after onboarding finishes.
- **D-04:** The main recording screen shows the **record button disabled** with an inline progress bar: "جاري تحميل نموذج النسخ العربي (45%)..." (Downloading Arabic transcription model (45%)...). The button **auto-enables** when the download completes. This reuses the same gate pattern for both onboarding and locale-switch scenarios.

### Locale Switch Provider Repoint (TRANS-04)
- **D-05:** When switching `uiLocale` from `en` to `ar`, the provider repoint notice is **integrated into the existing `LanguageConfirmDialog`** — not a separate dialog. An additional line appears in the confirmation body: "سيتم أيضاً تبديل مزود النسخ إلى Whisper large-v3 لدقة أعلى للعربية" (The transcription provider will also be switched to Whisper large-v3 for higher Arabic accuracy). One confirmation, one action.
- **D-06:** If `large-v3` is **not already downloaded** when the user switches to Arabic, the download starts automatically in the background after the reload, with the same disabled-record-button + progress gate as the onboarding path (D-04). No separate download confirmation — the user already confirmed the switch.

### Claude's Discretion
- How to detect `uiLocale` inside `TranscriptSettings` and `OnboardingContext` (context hook, prop drilling, or direct preferences read)
- The exact visual design of the informational banner (color, icon, layout) — should match existing note/info patterns in the app
- How to wire the background download trigger (Tauri event, effect hook, or onboarding context extension)
- Whether the progress gate component is shared or duplicated between onboarding and main screen contexts

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Spec
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` §5 (Transcription Policy), §6 (Onboarding Fork), §12.2 (Non-blocking download gate)

### Phase 1 Foundation
- `.planning/phases/01-preferences-foundation/01-CONTEXT.md` — D-06 through D-12 define the `set_user_preferences` invariant, patch shape, and commit-then-cache ordering that TRANS-04 rides on

### Phase 2 Surface
- `.planning/phases/02-i18n-framework-locale-bootstrap/02-CONTEXT.md` — LanguageConfirmDialog, I18nProvider, locale detection

### Requirements
- `.planning/REQUIREMENTS.md` — TRANS-01, TRANS-02, TRANS-03, TRANS-04 acceptance criteria

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `TranscriptSettings.tsx` — Provider dropdown with `modelOptions` object and `ParakeetModelManager` / `ModelManager` components. The dropdown filtering for Arabic can be done by conditionally excluding `'parakeet'` from the provider list.
- `WhisperModelManager.tsx` (`ModelManager`) — Full download progress UI, model listing, selection callback. Can be reused for the Arabic path's large-v3 download and selection.
- `LanguageConfirmDialog.tsx` — Already handles locale switch confirmation with reload. D-05 adds a line to the dialog body when switching to Arabic.
- `OnboardingContext.tsx` — Manages download state (`parakeetDownloaded`, `parakeetProgress`, `isBackgroundDownloading`). The Arabic fork needs to replace Parakeet references with Whisper large-v3 conditionally.
- `sonner` toast library — Already used across the app for notifications.

### Established Patterns
- **Background model download:** `OnboardingContext.tsx` already implements background downloading with progress events via Tauri `listen()`. The same pattern applies for large-v3.
- **Tauri events:** `parakeet-model-download-complete` event pattern in `transcriptService.ts` — Arabic path needs an equivalent `whisper-model-download-complete` or generic model download event.
- **Preference-driven UI:** Components read locale via `useLocale()` (re-exported from `I18nProvider.tsx`) — same mechanism for conditional rendering.

### Integration Points
- `TranscriptSettings.tsx:53` — `modelOptions` object where Parakeet is listed; filter here
- `TranscriptSettings.tsx:9` — `ParakeetModelManager` import; conditionally render based on locale
- `OnboardingContext.tsx:8` — `PARAKEET_MODEL` constant; fork to Whisper large-v3 for Arabic
- `OnboardingContext.tsx:424+` — Download initiation logic; branch on locale
- `LanguageConfirmDialog.tsx` — Add provider repoint notice to dialog body when target locale is Arabic
- `frontend/src/app/page.tsx` — Main recording interface; add download gate UI (disabled button + progress)

</code_context>

<specifics>
## Specific Ideas

- The "ready to record" gate (disabled button + progress) should be a **shared component** usable in both onboarding completion and locale-switch scenarios — same visual, same logic, different trigger points
- The informational banner in TranscriptSettings should use Arabic text from the message catalogue (`messages/ar.json`), not hardcoded strings
- When switching ar→en (reverse direction), the provider repoint is NOT automatic — the user keeps whatever provider they had. Only en→ar triggers the forced repoint (per the Phase 1 invariant: Arabic + Parakeet is banned, but English + any provider is fine)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-arabic-transcription-policy*
*Context gathered: 2026-04-09*
