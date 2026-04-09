# Phase 4: Arabic Transcription Policy - Research

**Researched:** 2026-04-09
**Domain:** React UI conditional rendering, Tauri model download events, locale-driven onboarding forking
**Confidence:** HIGH

## Summary

Phase 4 is a **UI-only phase** — the Rust backend enforcement (Parakeet-ban invariant in `set_user_preferences`) already shipped in Phase 1. This phase delivers three frontend outcomes: (1) hiding Parakeet from `TranscriptSettings` when `uiLocale === 'ar'` with an informational banner, (2) forking the onboarding flow so Arabic users skip the Parakeet download step and instead get Whisper `large-v3` downloaded in the background, and (3) enriching the `LanguageConfirmDialog` to mention provider repoint when switching to Arabic.

The codebase already has all the building blocks: `useLocale()` from `@/providers/I18nProvider` for reading the current locale, `WhisperAPI.downloadModel('large-v3')` for triggering Whisper downloads, `model-download-progress` / `model-download-complete` Tauri events for tracking Whisper download state, and `LanguageConfirmDialog` with `setUserPreferences({ uiLocale })` for locale switching. The main engineering challenge is creating a shared "model download gate" component that disables the record button with progress feedback, usable in both onboarding-completion and locale-switch contexts.

**Primary recommendation:** Build a shared `WhisperDownloadGate` component that listens to `model-download-progress` events, renders a disabled record button with inline progress, and auto-enables when `model-download-complete` fires. Wire it into `page.tsx` (main recording screen) conditional on `uiLocale === 'ar'` and model-not-ready state.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Parakeet is completely hidden (not disabled) from the provider dropdown in `TranscriptSettings.tsx` when `uiLocale === 'ar'`. The provider type union stays unchanged in TypeScript; only rendered options are filtered.
- **D-02:** Informational banner appears above the provider section in TranscriptSettings when `uiLocale === 'ar'`, with specific Arabic text explaining Whisper large-v3 usage.
- **D-03:** When `uiLocale === 'ar'`, onboarding completes immediately after permissions step. No blocking download step. Whisper `large-v3` download starts in background after onboarding finishes.
- **D-04:** Main recording screen shows record button disabled with inline progress bar during download. Same gate pattern for both onboarding and locale-switch. Auto-enables on completion.
- **D-05:** Provider repoint notice integrated into existing `LanguageConfirmDialog` — not a separate dialog. Additional line in confirmation body when switching to Arabic.
- **D-06:** If `large-v3` not downloaded when switching to Arabic, download starts automatically after reload, with same disabled-record-button + progress gate as onboarding path.

### Claude's Discretion
- How to detect `uiLocale` inside `TranscriptSettings` and `OnboardingContext` (context hook, prop drilling, or direct preferences read)
- Exact visual design of the informational banner (color, icon, layout) — match existing note/info patterns
- How to wire the background download trigger (Tauri event, effect hook, or onboarding context extension)
- Whether the progress gate component is shared or duplicated between onboarding and main screen contexts

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TRANS-01 | User records meeting in Arabic and receives MSA transcript via Whisper `large-v3` | `WhisperAPI.downloadModel('large-v3')` exists; `useRecordingStart` already gates on model readiness; needs locale-aware model check instead of Parakeet-only check |
| TRANS-02 | User in Arabic mode sees no Parakeet option; banner explains why | `TranscriptSettings.tsx` line 124 renders Parakeet as `SelectItem`; filter with `useLocale()` conditional; banner component above provider section |
| TRANS-03 | Arabic onboarding not blocked on 3GB download; gate waits in background | `OnboardingContext.tsx` has `startBackgroundDownloads()` pattern; fork to use `whisper_download_model` instead of `parakeet_download_model` when `uiLocale === 'ar'` |
| TRANS-04 | Locale switch en->ar auto-repoints provider to localWhisper + large-v3 | `LanguageConfirmDialog` calls `setUserPreferences({ uiLocale })` which triggers Phase 1's Parakeet-ban invariant in Rust; add notice text and post-reload download trigger |
</phase_requirements>

## Architecture Patterns

### Recommended Approach

```
frontend/src/
├── components/
│   ├── TranscriptSettings.tsx          # MODIFY: filter Parakeet, add banner
│   ├── RecordingControls.tsx           # MODIFY: integrate download gate
│   └── WhisperDownloadGate.tsx         # NEW: shared download progress gate
├── hooks/
│   ├── useRecordingStart.ts            # MODIFY: locale-aware model check
│   └── useWhisperDownloadState.ts      # NEW: hook for Whisper download tracking
├── contexts/
│   └── OnboardingContext.tsx            # MODIFY: fork downloads by locale
├── components/settings/
│   └── LanguageConfirmDialog.tsx        # MODIFY: add repoint notice
├── components/onboarding/
│   └── steps/DownloadProgressStep.tsx   # MODIFY: show Whisper progress for Arabic
└── messages/
    ├── en.json                          # ADD: transcript policy strings
    └── ar.json                          # ADD: transcript policy strings
```

### Pattern 1: Locale Detection in Components
**What:** Use `useLocale()` re-exported from `@/providers/I18nProvider` to read the current locale.
**When to use:** Any component that needs locale-conditional rendering.
**Why:** Already established in Phase 2. `useLocale()` returns `'en' | 'ar'`. No prop drilling needed.
**Example:**
```typescript
// Source: [VERIFIED: frontend/src/providers/I18nProvider.tsx line 39]
import { useLocale } from '@/providers/I18nProvider';

function TranscriptSettings() {
  const locale = useLocale();
  const isArabic = locale === 'ar';
  // Filter providers based on locale
}
```

### Pattern 2: Whisper Download Event Listening
**What:** Listen to existing Tauri events for Whisper model download progress.
**When to use:** Download gate component and onboarding fork.
**Example:**
```typescript
// Source: [VERIFIED: frontend/src-tauri/src/whisper_engine/commands.rs lines 441, 459]
// Existing events:
//   'model-download-progress' — { modelName, progress, totalBytes, downloadedBytes, speed }
//   'model-download-complete' — { modelName }
// Trigger download:
//   WhisperAPI.downloadModel('large-v3')  // [VERIFIED: frontend/src/lib/whisper.ts line 311]
```

### Pattern 3: Background Download with Gate
**What:** Start download in background, show progress gate on recording screen.
**When to use:** Arabic onboarding completion and locale switch.
**Example:**
```typescript
// Source: [VERIFIED: existing pattern in OnboardingContext.tsx lines 424-449]
// The startBackgroundDownloads pattern already exists for Parakeet.
// Fork it: when uiLocale === 'ar', call whisper_download_model('large-v3')
// instead of parakeet_download_model.
```

### Pattern 4: Recording Gate (Model Readiness Check)
**What:** `useRecordingStart` already checks `checkParakeetReady()` before allowing recording.
**When to use:** Must be extended to check Whisper readiness when `uiLocale === 'ar'`.
**Key insight:** The check at line 88 calls `parakeet_has_available_models`. For Arabic, it should instead call `whisper_has_available_models` AND verify `large-v3` specifically is available.
**Example:**
```typescript
// Source: [VERIFIED: frontend/src/hooks/useRecordingStart.ts lines 53-61, 88-107]
// Current: checkParakeetReady() → invoke('parakeet_has_available_models')
// Arabic: checkWhisperReady() → invoke('whisper_has_available_models') + verify model is large-v3
// WhisperAPI.hasAvailableModels() exists [VERIFIED: frontend/src/lib/whisper.ts line 323]
```

### Anti-Patterns to Avoid
- **Double-checking locale in Rust AND JS:** The Rust `set_user_preferences` already enforces the Parakeet ban. JS-side filtering is purely UI cosmetic. Don't add redundant Rust-side checks for UI rendering.
- **Separate download confirmation dialog:** D-06 explicitly says no separate download confirmation when switching to Arabic. The user already confirmed the switch.
- **Blocking onboarding on download:** D-03 explicitly says onboarding completes immediately. Never gate onboarding completion on model download.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Model download progress tracking | Custom download state management | Listen to existing `model-download-progress` Tauri event | Whisper engine already emits granular progress events with bytes/speed |
| Model availability check | Custom file-system check | `WhisperAPI.hasAvailableModels()` + `WhisperAPI.getAvailableModels()` | Rust backend already handles model validation, corruption detection |
| Locale detection | Read from localStorage or preferences directly | `useLocale()` from `@/providers/I18nProvider` | Already set up in Phase 2, consistent across all components |
| Provider repoint logic | Manual SQLite update or separate API call | `setUserPreferences({ uiLocale: 'ar' })` | Phase 1 invariant auto-rejects Parakeet when uiLocale is 'ar' |

**Key insight:** The Rust side already handles all transcription policy enforcement. Phase 4 is purely about making the UI reflect and complement those backend invariants.

## Common Pitfalls

### Pitfall 1: OnboardingContext Still Downloads Parakeet for Arabic Users
**What goes wrong:** If the onboarding flow doesn't check locale, it will still call `parakeet_download_model` even for Arabic users, wasting ~600MB of bandwidth on a model they can't use.
**Why it happens:** `OnboardingContext.tsx` line 433 unconditionally calls `parakeet_download_model`. The `PARAKEET_MODEL` constant is hardcoded at line 8.
**How to avoid:** Fork `startBackgroundDownloads` to check locale. When `uiLocale === 'ar'`, call `whisper_download_model('large-v3')` instead.
**Warning signs:** Arabic user's first download is Parakeet, not Whisper.

### Pitfall 2: useRecordingStart Only Checks Parakeet
**What goes wrong:** Even after downloading Whisper large-v3, the recording start is blocked because `checkParakeetReady()` returns false (Parakeet was never downloaded for Arabic users).
**Why it happens:** `useRecordingStart.ts` lines 53-61 only check Parakeet model status.
**How to avoid:** Add a locale-aware check: when `uiLocale === 'ar'`, check Whisper model readiness instead of Parakeet.
**Warning signs:** Arabic user has Whisper large-v3 downloaded but can't start recording.

### Pitfall 3: Download Gate Doesn't Survive Page Reload
**What goes wrong:** User switches to Arabic, page reloads (per D-05/LanguageConfirmDialog), download gate state is lost.
**Why it happens:** The download gate state is React state that doesn't persist across reloads. The download continues in the background (Rust side), but the UI doesn't know about it.
**How to avoid:** On mount, check if a Whisper download is in progress (similar to `checkActiveDownloads()` in OnboardingContext.tsx lines 453-467). Re-attach to progress events if download is active.
**Warning signs:** Progress bar appears briefly then disappears on reload.

### Pitfall 4: Banner and Filter Out of Sync
**What goes wrong:** Banner says "using Whisper large-v3" but the provider dropdown still shows Parakeet.
**Why it happens:** Banner and filter use different locale sources or the filter condition is slightly different.
**How to avoid:** Both the banner visibility and the Parakeet filter should derive from the same `useLocale() === 'ar'` check in the same component.
**Warning signs:** Visual inconsistency between banner message and available options.

### Pitfall 5: Reverse Switch (ar->en) Incorrectly Auto-Downloads Parakeet
**What goes wrong:** User switches back from Arabic to English and Parakeet auto-downloads without consent.
**Why it happens:** The download gate logic triggers on locale change without checking direction.
**How to avoid:** Per CONTEXT.md specifics section: only en->ar triggers forced repoint. ar->en is NOT automatic — user keeps whatever provider they had. The download gate should only activate when `uiLocale === 'ar'` AND Whisper large-v3 is not ready.
**Warning signs:** Switching to English triggers unexpected downloads.

## Code Examples

### TranscriptSettings Parakeet Filter
```typescript
// Source: [VERIFIED: TranscriptSettings.tsx lines 123-131]
// Current code has Parakeet as first SelectItem.
// D-01: Filter it out when Arabic.
import { useLocale } from '@/providers/I18nProvider';

// Inside TranscriptSettings:
const locale = useLocale();
const isArabic = locale === 'ar';

// In SelectContent:
{!isArabic && (
  <SelectItem value="parakeet">Parakeet (Recommended - Real-time / Accurate)</SelectItem>
)}
<SelectItem value="localWhisper">Local Whisper (High Accuracy)</SelectItem>
```

### LanguageConfirmDialog Repoint Notice
```typescript
// Source: [VERIFIED: LanguageConfirmDialog.tsx lines 78-85]
// D-05: Add repoint notice when targetLocale is 'ar'
{targetLocale === 'ar' && (
  <p className="mt-2 text-small font-normal text-muted-foreground">
    {t('confirm.providerRepoint')}
  </p>
)}
```

### Whisper Download Gate Hook
```typescript
// New hook: useWhisperDownloadState.ts
// Listens to 'model-download-progress' and 'model-download-complete' events
// Returns: { isDownloading, progress, isReady }
// Used by WhisperDownloadGate component on main recording screen

import { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import { WhisperAPI } from '@/lib/whisper';

export function useWhisperDownloadState(modelName: string) {
  const [isReady, setIsReady] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    // Check initial state
    WhisperAPI.getAvailableModels().then(models => {
      const model = models.find(m => m.name === modelName);
      if (model?.status === 'Available') setIsReady(true);
      else if (model?.status && typeof model.status === 'object' && 'Downloading' in model.status) {
        setIsDownloading(true);
        setProgress(model.status.Downloading);
      }
    });

    // Listen for progress
    const unlistenProgress = listen('model-download-progress', (event: any) => {
      if (event.payload.modelName === modelName) {
        setIsDownloading(true);
        setProgress(event.payload.progress);
      }
    });

    const unlistenComplete = listen('model-download-complete', (event: any) => {
      if (event.payload.modelName === modelName) {
        setIsReady(true);
        setIsDownloading(false);
        setProgress(100);
      }
    });

    return () => {
      unlistenProgress.then(fn => fn());
      unlistenComplete.then(fn => fn());
    };
  }, [modelName]);

  return { isReady, isDownloading, progress };
}
```

## Integration Point Inventory

| File | Line(s) | What Changes | Why |
|------|---------|-------------|-----|
| `TranscriptSettings.tsx` | 124 | Hide Parakeet `SelectItem` when `isArabic` | D-01 |
| `TranscriptSettings.tsx` | 104 (above provider div) | Add informational banner | D-02 |
| `TranscriptSettings.tsx` | 165-173 | Hide `ParakeetModelManager` when `isArabic` | D-01 corollary |
| `OnboardingContext.tsx` | 8, 424-449 | Fork `startBackgroundDownloads` by locale | D-03 |
| `OnboardingContext.tsx` | 326-337 | Skip Parakeet verification for Arabic | D-03 corollary |
| `OnboardingFlow.tsx` | 42-46 | Skip download step for Arabic | D-03 |
| `LanguageConfirmDialog.tsx` | 78-85 | Add repoint notice line | D-05 |
| `useRecordingStart.ts` | 53-61, 88 | Locale-aware model readiness check | TRANS-01 |
| `page.tsx` | ~236 | Integrate `WhisperDownloadGate` | D-04 |
| `en.json` / `ar.json` | new keys | Add transcript policy i18n strings | D-02, D-04, D-05 |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual testing via Tauri dev mode |
| Config file | N/A (no automated test framework for frontend) |
| Quick run command | `pnpm run tauri:dev` |
| Full suite command | Manual UAT per success criteria |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TRANS-01 | Arabic recording produces Whisper large-v3 transcript | manual | N/A — requires Arabic audio input | N/A |
| TRANS-02 | No Parakeet in TranscriptSettings when ar | manual | Visual inspection in dev mode with `uiLocale='ar'` | N/A |
| TRANS-03 | Arabic onboarding non-blocking, background download | manual | Complete onboarding with `navigator.language='ar'`, verify download gate | N/A |
| TRANS-04 | en->ar switch repoints provider and starts download | manual | Switch locale in settings, verify dialog text and post-reload behavior | N/A |

### Sampling Rate
- **Per task commit:** `pnpm run dev` visual verification
- **Per wave merge:** Full UAT walkthrough of all 4 success criteria
- **Phase gate:** All 4 success criteria verified manually

### Wave 0 Gaps
None — this phase is UI-only with manual verification. No test framework gaps to fill.

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | N/A |
| V3 Session Management | No | N/A |
| V4 Access Control | Yes | Phase 1 Rust invariant enforces Parakeet ban; UI filtering is defense-in-depth |
| V5 Input Validation | No | No new user input surfaces |
| V6 Cryptography | No | N/A |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| User manually sets provider to Parakeet via DevTools | Tampering | Phase 1 `set_user_preferences` rejects the write in Rust — UI filter is cosmetic only |
| Model download MITM | Tampering | Whisper download uses HTTPS with checksum verification (existing Rust implementation) |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `model-download-progress` event payload includes `modelName` field that can be filtered for `large-v3` | Architecture Patterns | Download gate would track wrong model's progress |
| A2 | `WhisperAPI.hasAvailableModels()` returns true only when at least one model file passes integrity check | Architecture Patterns | Gate might enable recording before model is actually usable |
| A3 | The Whisper download continues across page reloads (Rust-side async task survives frontend reload) | Pitfall 3 | Download would restart from scratch on locale switch reload |

## Open Questions

1. **Download resume after reload**
   - What we know: OnboardingContext has `checkActiveDownloads()` that queries Parakeet download status
   - What's unclear: Does the Whisper engine expose a similar API to check if a download is in progress? Need to verify `whisper_get_available_models` returns `{ Downloading: percent }` status like Parakeet does
   - Recommendation: Check Whisper engine commands during implementation; if not available, the gate hook should poll `WhisperAPI.getAvailableModels()` on mount

2. **large-v3 model name string**
   - What we know: `MODEL_CONFIGS` in `whisper.ts` lists `'large-v3'` with `size_mb: 2951` (~3GB)
   - What's unclear: Is the Tauri command `whisper_download_model` expecting exactly `'large-v3'` as the model name string?
   - Recommendation: Verify by checking Rust `whisper_engine/commands.rs` download handler; likely matches since `MODEL_CONFIGS` keys align with Rust model names

## Sources

### Primary (HIGH confidence)
- `frontend/src/components/TranscriptSettings.tsx` — full source read, provider dropdown structure confirmed
- `frontend/src/contexts/OnboardingContext.tsx` — full source read, download flow and event patterns confirmed
- `frontend/src/components/settings/LanguageConfirmDialog.tsx` — full source read, dialog structure confirmed
- `frontend/src/hooks/useRecordingStart.ts` — full source read, Parakeet-only gate confirmed
- `frontend/src/lib/whisper.ts` — full source read, `WhisperAPI` class and `MODEL_CONFIGS` confirmed
- `frontend/src/providers/I18nProvider.tsx` — full source read, `useLocale()` re-export confirmed
- `frontend/src/services/preferencesService.ts` — full source read, `setUserPreferences` patch shape confirmed
- `frontend/src-tauri/src/whisper_engine/commands.rs` — grep confirmed event names: `model-download-progress`, `model-download-complete`

### Secondary (MEDIUM confidence)
- `frontend/src/services/transcriptService.ts` — event listener patterns for model downloads confirmed

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all libraries already in use, no new dependencies
- Architecture: HIGH - all integration points verified by source reading
- Pitfalls: HIGH - derived from actual code inspection of current behavior

**Research date:** 2026-04-09
**Valid until:** 2026-05-09 (stable — no external dependency changes expected)
