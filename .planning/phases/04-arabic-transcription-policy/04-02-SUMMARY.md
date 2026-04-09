---
phase: 04-arabic-transcription-policy
plan: 02
status: complete
started: 2026-04-09
completed: 2026-04-09
commits: [8335178, 53ba8e8, 877b2e4]
requirements_satisfied: [TRANS-01, TRANS-03, TRANS-04]
---

# Plan 04-02 Summary — Download Gate + Onboarding Fork

## What was built

1. **useWhisperDownloadState hook** — Tracks Whisper model download state via Tauri events (`model-download-progress`, `model-download-complete`, `model-download-error`), checks initial state on mount, provides `startDownload` and `retryDownload` actions
2. **WhisperDownloadGate component** — Gate that auto-downloads specified Whisper model; renders children when ready, shows disabled button + progress bar while downloading, retry affordance on error
3. **Onboarding fork** — Arabic users skip DownloadProgressStep, `startBackgroundDownloads` downloads Whisper large-v3 instead of Parakeet, event listeners forked by locale
4. **Locale-aware recording start** — `useRecordingStart` uses `checkWhisperReady` for Arabic (all 3 paths: manual, auto-start, sidebar), `checkParakeetReady` for English
5. **page.tsx integration** — Recording controls wrapped with `WhisperDownloadGate` when Arabic, with sonner toast on model ready

## Files created

| File | Purpose |
|------|---------|
| frontend/src/hooks/useWhisperDownloadState.ts | Whisper download state tracking hook |
| frontend/src/components/WhisperDownloadGate.tsx | Download gate with progress UI |

## Files modified

| File | Change |
|------|--------|
| frontend/src/contexts/OnboardingContext.tsx | Locale fork for downloads, event listeners, verification |
| frontend/src/components/onboarding/OnboardingFlow.tsx | Skip DownloadProgressStep for Arabic |
| frontend/src/components/onboarding/steps/DownloadProgressStep.tsx | Whisper events + model for Arabic |
| frontend/src/hooks/useRecordingStart.ts | checkWhisperReady + locale-aware fork |
| frontend/src/app/page.tsx | WhisperDownloadGate integration + toast |

## Deviations

None — executed as planned.

## Pitfall avoidance (from RESEARCH.md)

- Pitfall 1: startBackgroundDownloads forks by locale — Arabic never downloads Parakeet ✓
- Pitfall 2: useRecordingStart checks Whisper when Arabic — not blocked on Parakeet ✓
- Pitfall 3: useWhisperDownloadState checks initial state on mount — survives reload ✓
- Pitfall 4: Banner and filter both use same useLocale() === 'ar' — in sync ✓
- Pitfall 5: Only en→ar triggers repoint; ar→en does NOT auto-download Parakeet ✓
