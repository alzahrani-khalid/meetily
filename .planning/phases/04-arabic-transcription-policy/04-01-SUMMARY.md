---
phase: 04-arabic-transcription-policy
plan: 01
status: complete
started: 2026-04-09
completed: 2026-04-09
commits: [63f760d, 8e9af8d, bea87cf]
requirements_satisfied: [TRANS-02, TRANS-04]
---

# Plan 04-01 Summary — UI Filtering + i18n Strings

## What was built

1. **i18n strings** — Added `transcriptSettings`, `recording`, and `languageConfirm` namespaces to both `en.json` and `ar.json` (8 keys per locale)
2. **Parakeet filtering** — `TranscriptSettings.tsx` hides Parakeet from provider dropdown when `locale === 'ar'`, shows blue info banner explaining Whisper large-v3 usage, and auto-corrects provider if Parakeet was previously selected
3. **Repoint notice** — `LanguageConfirmDialog.tsx` shows italic provider repoint notice only when switching TO Arabic (en→ar), not ar→en

## Files modified

| File | Change |
|------|--------|
| frontend/src/messages/en.json | +3 i18n sections (transcriptSettings, recording, languageConfirm) |
| frontend/src/messages/ar.json | +3 matching Arabic i18n sections |
| frontend/src/components/TranscriptSettings.tsx | Locale-aware filtering, banner, auto-correction useEffect |
| frontend/src/components/settings/LanguageConfirmDialog.tsx | Provider repoint notice for en→ar |

## Deviations

None — executed exactly as planned.

## Decisions

- All new Tailwind classes use logical properties only (no physical-direction classes)
- useTranslations used via next-intl for all new UI strings
