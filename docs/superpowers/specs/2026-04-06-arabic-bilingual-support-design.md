# Meetily Arabic Bilingual Support — Design Spec

**Date:** 2026-04-06
**Status:** Draft
**Scope:** Full bilingual (Arabic + English) support for Meetily

---

## 1. Overview

Add full Arabic language support to Meetily, making it a bilingual (Arabic/English) meeting transcription and AI summary tool. This covers UI localization, RTL layout, Arabic speech transcription, and Arabic AI summaries.

## 2. Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope | Full bilingual (AR + EN) | Preserve existing English user base |
| Language switching | OS default + in-app override | Most user-friendly |
| i18n library | `next-intl` | App Router native, server component support |
| RTL strategy | Tailwind logical properties | Clean, maintainable, no separate stylesheet |
| Arabic font | Tajawal (Google Fonts) | Optimized for Arabic, clean modern look |
| Transcription model | Whisper large-v3 only for Arabic | Smaller models hallucinate in Arabic |
| Summary language | Independent from UI locale | Supports bilingual teams |
| LLM prompts | External config files (en/ar) | Currently hardcoded in Rust |
| Meeting templates | Duplicated per language | 6 templates × 2 languages |

## 3. i18n Infrastructure

### 3.1 Library: `next-intl`

Chosen for native Next.js App Router integration, server component support, and built-in message formatting (pluralization, interpolation).

### 3.2 Locale Files

```
frontend/messages/
├── en.json    # English strings (extracted from current hardcoded text)
└── ar.json    # Arabic translations
```

String keys organized by component/feature:

```json
{
  "sidebar": {
    "search": "Search meetings...",
    "newMeeting": "New Meeting",
    "settings": "Settings"
  },
  "settings": {
    "title": "Settings",
    "language": "Language",
    "summaryLanguage": "Summary Language",
    "transcription": "Transcription"
  },
  "transcript": {
    "processing": "Processing...",
    "idle": "Idle"
  }
}
```

### 3.3 Settings Integration

Add two new settings to the existing `SettingsStore`:

- **`locale`** (ar | en) — Controls UI language and text direction
  - Default: auto-detect from `navigator.language`
  - Override: dropdown in Settings modal
- **`summaryLanguage`** (ar | en) — Controls LLM prompt language
  - Default: follows `locale`
  - Override: independent dropdown in Settings modal

### 3.4 Root Layout

```tsx
<html lang={locale} dir={locale === 'ar' ? 'rtl' : 'ltr'}>
```

Wrap app in `NextIntlClientProvider` with messages for the active locale.

## 4. RTL Layout Conversion

### 4.1 Tailwind Class Conversion

All directional Tailwind classes converted to logical equivalents across ~110 frontend files:

| Before | After |
|--------|-------|
| `ml-*` | `ms-*` |
| `mr-*` | `me-*` |
| `pl-*` | `ps-*` |
| `pr-*` | `pe-*` |
| `left-0` | `start-0` |
| `right-0` | `end-0` |
| `text-left` | `text-start` |
| `text-right` | `text-end` |
| `space-x-*` | `space-x-*` + `rtl:space-x-reverse` |
| `border-l-*` | `border-s-*` |
| `border-r-*` | `border-e-*` |
| `rounded-l-*` | `rounded-s-*` |
| `rounded-r-*` | `rounded-e-*` |

### 4.2 Sidebar (Special Handling)

The sidebar is the most complex RTL component:

- `fixed top-0 left-0` → `fixed top-0 start-0`
- Collapse/expand `translate-x` animations: flip direction based on `dir` attribute
- Collapse toggle button: repositioned to logical end of sidebar

### 4.3 Panel Layout (Meeting Details)

TranscriptPanel + SummaryPanel are in a `flex-row` container. Setting `dir="rtl"` on the parent automatically swaps their visual positions — no manual reordering needed.

### 4.4 Font Stack

Add Tajawal (Arabic-optimized Google Font) to the Tailwind config:

```js
// tailwind.config.js
fontFamily: {
  sans: ['Inter', 'Tajawal', ...defaultTheme.fontFamily.sans],
}
```

Load Tajawal via `next/font/google` or `<link>` in the root layout.

## 5. Transcription Engine

### 5.1 Whisper Arabic Configuration

- Arabic language code `"ar"` is already in the `LanguageSelection.tsx` dropdown
- Pass through the existing `TranscriptionProvider` chain unchanged
- **Enforce large-v3 model** when Arabic is selected — disable smaller models in the UI with a tooltip: "Arabic transcription requires the large-v3 model for accuracy"

### 5.2 Parakeet

- Disable for Arabic locale — Parakeet TDT variants do not support Arabic
- Show informational tooltip when Arabic is selected: "Parakeet does not support Arabic. Whisper will be used."

### 5.3 Quality Expectations

- MSA (Modern Standard Arabic): ~85-88% accuracy with large-v3
- Dialectal Arabic: lower accuracy, out of scope for v1
- No mixed-language (code-switching) support in v1

## 6. AI Summaries & Prompts

### 6.1 Prompt Externalization

Move the 5 hardcoded English prompts from `processor.rs` into external config files:

```
frontend/src-tauri/prompts/
├── en/
│   ├── chunk_system.txt
│   ├── chunk_user.txt
│   ├── combine_system.txt
│   ├── combine_user.txt
│   └── report_system.txt
└── ar/
    ├── chunk_system.txt
    ├── chunk_user.txt
    ├── combine_system.txt
    ├── combine_user.txt
    └── report_system.txt
```

Rust loads prompts at runtime based on `summaryLanguage` setting.

### 6.2 Arabic Prompt Design

Arabic prompts will:
- Instruct the LLM to generate Arabic output
- Use MSA (formal Arabic) for consistency
- Request proper RTL formatting in output
- Maintain the same structure as English prompts (chunk → combine → report)

### 6.3 Meeting Templates

Duplicate the 6 existing JSON templates into Arabic versions:

```
frontend/src-tauri/templates/
├── en/
│   ├── standard_meeting.json
│   ├── daily_standup.json
│   ├── project_sync.json
│   ├── retrospective.json
│   ├── sales_marketing_client_call.json
│   └── psychiatric_session.json
└── ar/
    ├── standard_meeting.json
    ├── daily_standup.json
    ├── project_sync.json
    ├── retrospective.json
    ├── sales_marketing_client_call.json
    └── psychiatric_session.json
```

Template selection follows `summaryLanguage` setting.

## 7. BlockNote Editor

- Set `direction: "rtl"` on the BlockNote editor instance when locale is Arabic
- Apply Tajawal font to editor content via CSS
- Existing formatting features (bold, lists, headings) work with RTL — Radix primitives are RTL-aware

## 8. Rust Backend Strings

- Tray menu items (~13 strings): extract into a locale map, loaded at startup based on `locale` setting
- Notification messages (~10 strings): same approach
- Log/error messages: remain in English (developer-facing, not user-visible)

## 9. Out of Scope (YAGNI)

- Dialect-specific Arabic support (MSA only for v1)
- Mixed-language transcription (single language per session)
- Arabic-specific Whisper fine-tuning
- Custom Arabic keyboard input handling (OS handles this)
- Arabic onboarding wizard rewrite (just translate strings)
- Per-component language switching (whole app switches at once)

## 10. Testing Strategy

- Visual RTL regression: manual pass through all screens in Arabic mode
- Key test surfaces: Sidebar (collapse/expand), Settings modal, Transcript panel, Summary panel, Onboarding flow, Tray menu
- Verify no text overflow or clipping with Arabic strings (Arabic text can be wider than English)
- Test Whisper Arabic transcription with MSA audio samples
- Test LLM summaries in Arabic with at least 2 providers (Claude + Ollama)

## 11. Implementation Phases

| Phase | Scope | Estimated Effort |
|-------|-------|-----------------|
| P0 | i18n framework setup + string extraction + Arabic translations | 7-11 days |
| P1 | RTL CSS conversion + sidebar flip + LLM prompts | 8-12 days |
| P2 | Arabic font + BlockNote RTL + Whisper config | 3-5 days |
| P3 | Rust backend strings + testing & polish | 4-7 days |
| **Total** | | **22-35 days** |
