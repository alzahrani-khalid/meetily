# Phase 4: Arabic Transcription Policy - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 04-arabic-transcription-policy
**Areas discussed:** Parakeet UI Filtering, Arabic Onboarding Fork, Locale Switch Experience

---

## Parakeet UI Filtering

### How to hide Parakeet in TranscriptSettings for Arabic users?

| Option | Description | Selected |
|--------|-------------|----------|
| إخفاء كامل + بانر | Remove Parakeet entirely from dropdown, show informational banner above provider section | ✓ |
| معطّل مع تفسير | Keep Parakeet in list but disabled with tooltip explaining incompatibility | |
| إخفاء صامت | Remove Parakeet silently with no banner or explanation | |

**User's choice:** إخفاء كامل + بانر (Complete removal + informational banner)
**Notes:** Banner positioned above provider section, explains Whisper large-v3 is used for Arabic accuracy.

### Banner position in TranscriptSettings?

| Option | Description | Selected |
|--------|-------------|----------|
| أعلى قسم المزود | Informational banner above provider dropdown — first thing user sees | ✓ |
| أسفل القسم | Small footnote-style note below transcript settings | |
| أنت قرر | Claude decides based on current design | |

**User's choice:** أعلى قسم المزود (Above provider section)

---

## Arabic Onboarding Fork

### How does Arabic onboarding handle the large-v3 download (~3GB)?

| Option | Description | Selected |
|--------|-------------|----------|
| إكمال فوري + تحميل خلفي | Onboarding completes after permissions, large-v3 downloads in background with "ready to record" gate | ✓ |
| تحميل أثناء الإعداد | Download step inside onboarding wizard with "continue in background" button | |
| أنت قرر | Claude decides best experience | |

**User's choice:** إكمال فوري + تحميل خلفي (Immediate completion + background download)

### How does the "ready to record" gate work?

| Option | Description | Selected |
|--------|-------------|----------|
| زر معطّل + تقدم | Record button disabled with inline progress bar showing download percentage | ✓ |
| نافذة تنبيه عند الضغط | Record button always enabled, alert dialog when pressed before download completes | |
| أنت قرر | Claude decides | |

**User's choice:** زر معطّل + تقدم (Disabled button + progress indicator)

---

## Locale Switch Experience

### What does the user see when switching en→ar and provider changes automatically?

| Option | Description | Selected |
|--------|-------------|----------|
| toast معلوماتي | Short toast after reload: "تم التبديل إلى Whisper large-v3 للنسخ العربي" | |
| صامت تماماً | No notification — user only notices in transcript settings | |
| نافذة تأكيد | Confirmation dialog asking user to approve the provider change | ✓ |

**User's choice:** نافذة تأكيد (Confirmation dialog)
**Notes:** User prefers transparency — wants explicit confirmation before provider repoint.

### What happens when large-v3 isn't downloaded yet?

| Option | Description | Selected |
|--------|-------------|----------|
| تحميل تلقائي + بوابة | Auto-start download with same disabled-button + progress gate as onboarding | ✓ |
| طلب موافقة التحميل | Dialog asking: "Arabic transcription requires ~3GB model download. Continue?" | |

**User's choice:** تحميل تلقائي + بوابة (Auto download + gate)

### Should the provider repoint notice be integrated into LanguageConfirmDialog?

| Option | Description | Selected |
|--------|-------------|----------|
| دمج في نفس الحوار | Add line to existing LanguageConfirmDialog — one dialog, not two | ✓ |
| حوار منفصل | Separate dialog after reload for provider change confirmation | |

**User's choice:** دمج في نفس الحوار (Integrated into existing dialog)
**Notes:** This satisfies the "confirmation dialog" preference while keeping UX clean — no extra dialog step.

---

## Claude's Discretion

- Detection mechanism for `uiLocale` in TranscriptSettings and OnboardingContext
- Visual design of the informational banner
- Background download trigger wiring
- Whether the progress gate component is shared or duplicated

## Deferred Ideas

None — discussion stayed within phase scope
