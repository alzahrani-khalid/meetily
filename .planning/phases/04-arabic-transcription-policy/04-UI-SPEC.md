---
phase: 4
slug: arabic-transcription-policy
status: draft
shadcn_initialized: true
preset: "new-york / neutral / cssVariables / lucide"
created: 2026-04-09
milestone: v1.0 — Arabic Bilingual Support
requirements: [TRANS-01, TRANS-02, TRANS-03, TRANS-04]
upstream_sources:
  - .planning/phases/04-arabic-transcription-policy/04-CONTEXT.md
  - .planning/phases/02-i18n-framework-locale-bootstrap/02-UI-SPEC.md
  - .planning/REQUIREMENTS.md (TRANS-01..TRANS-04)
  - .planning/ROADMAP.md (Phase 4 goal + success criteria)
  - frontend/components.json (shadcn preset)
  - frontend/tailwind.config.ts (existing tokens)
  - frontend/src/components/ui/alert.tsx (info banner primitive)
  - frontend/src/components/ui/progress.tsx (download progress primitive)
  - CLAUDE.md (Web/Tailwind RTL rules)
---

# Phase 4 — UI Design Contract

> Visual and interaction contract for the **Arabic Transcription Policy** phase.
> This phase adds conditional UI surfaces that appear only when `uiLocale === 'ar'`:
> a Parakeet-filtering banner in TranscriptSettings, a recording gate with download progress
> on the main screen, and a provider-repoint notice in LanguageConfirmDialog.

---

## Phase Intent

Ensure an Arabic-locale user gets accurate transcription via Whisper `large-v3` and never encounters Parakeet. Three UI surfaces are affected:

1. **TranscriptSettings** -- Parakeet hidden from provider dropdown + informational banner (TRANS-02)
2. **Main recording screen** -- Record button disabled with inline progress while `large-v3` downloads (TRANS-03, shared gate for onboarding and locale-switch)
3. **LanguageConfirmDialog** -- Additional provider-repoint notice when switching to Arabic (TRANS-04)

**No new pages or layouts.** All changes are conditional additions to existing components.

**RTL rule (inherited from Phase 2):** All new markup uses logical Tailwind primitives only (`ms-*`, `me-*`, `ps-*`, `pe-*`, `text-start`, `text-end`). No physical-direction classes.

---

## Design System

Inherited from Phase 2 approved UI-SPEC. No changes.

| Property | Value | Source |
|----------|-------|--------|
| Tool | shadcn (already initialized) | `frontend/components.json` |
| Style preset | `new-york` | `components.json:style` |
| Base color | `neutral` | `components.json:tailwind.baseColor` |
| CSS variables | `true` | `components.json:tailwind.cssVariables` |
| Component library | Radix (via shadcn/ui) | existing `@/components/ui/*` |
| Icon library | `lucide-react` | `components.json:iconLibrary` |
| Primary font (EN) | `Source Sans 3` | Phase 2 shipped |
| Primary font (AR) | `Tajawal` | Phase 2 shipped |
| i18n library | `next-intl` (client mode) | Phase 2 shipped |

---

## Spacing Scale

Inherited from Phase 2. No new tokens needed.

| Token | Value | Usage in Phase 4 |
|-------|-------|-------------------|
| xs | 4px | Gap between `Info` icon and banner text |
| sm | 8px | Inner padding of progress bar container; gap between progress bar and percentage label |
| md | 16px | Padding inside the informational banner; gap between disabled record button and progress status text |
| lg | 24px | Vertical spacing between the banner and the provider dropdown in TranscriptSettings |
| xl | 32px | (not used by Phase 4 surfaces) |
| 2xl | 48px | (not used by Phase 4 surfaces) |
| 3xl | 64px | (not used by Phase 4 surfaces) |

**Exceptions:** none.

**Touch targets:** Record button retains existing 44x44 minimum even in disabled state (`min-h-11 min-w-11`).

---

## Typography

Inherited from `tailwind.config.ts`. Phase 4 surfaces use only these existing roles:

| Role | Size | Weight | Line Height | Usage in Phase 4 |
|------|------|--------|-------------|-------------------|
| Body | 16px | 400 | 1.6 | Banner description text, progress status text |
| Small | 14px | 400 | 1.5 | Provider repoint notice in LanguageConfirmDialog, download percentage label |
| Caption | 12px | 400 | 1.4 | (not used by Phase 4) |
| H2 | 18px | 500 | 1.4 | (not used by Phase 4 -- banner has no title, uses body text only) |

**Font selection:** Arabic text renders in Tajawal (via `--font-tajawal` CSS var, inherited from Phase 2). English text renders in Source Sans 3. No new font loading.

---

## Color

Inherited from Phase 2 globals.css CSS variables. Phase 4 uses the existing palette with one semantic addition:

| Role | Value | Usage in Phase 4 |
|------|-------|-------------------|
| Dominant (60%) | `--background` (hsl 0 0% 100% / dark: 0 0% 3.9%) | Page background, unchanged |
| Secondary (30%) | `--card` / `--muted` | TranscriptSettings card background, unchanged |
| Accent (10%) | `--primary` (hsl 0 0% 9% / dark: 0 0% 98%) | Active progress bar fill, enabled record button |
| Destructive | `--destructive` (hsl 0 84.2% 60.2%) | Not used by Phase 4 |
| **Informational** | `hsl(221 83% 53% / 0.1)` border + `hsl(221 83% 53%)` icon | **Info banner in TranscriptSettings only** (uses existing `blue-600` from `tailwind.config.ts` `primary` / `accent` color) |

**Informational banner color contract:**
- Background: `bg-blue-600/10` (10% opacity blue over surface)
- Border: `border border-blue-600/20`
- Icon color: `text-blue-600`
- Text color: `text-foreground` (inherits from surface)

This matches the existing `Alert` component pattern but uses the informational (blue) semantic instead of default (neutral) or destructive (red). The `Alert` component from `@/components/ui/alert` is used directly with a custom className override for the blue tint.

**Disabled record button:**
- Button background: `bg-muted` (grayed out)
- Button text: `text-muted-foreground`
- Progress bar track: `bg-primary/20`
- Progress bar fill: `bg-primary`

---

## Component Inventory

### 1. Arabic Transcription Banner (TRANS-02)

**Location:** Inside `TranscriptSettings.tsx`, rendered above the provider dropdown section.
**Condition:** `uiLocale === 'ar'` (read via `useLocale()`)
**Primitive:** `<Alert>` from `@/components/ui/alert` with custom informational styling.
**Icon:** `Info` from `lucide-react`, 16x16, placed at `start-4` (logical).

**Layout:**
```
+--------------------------------------------------+
| [i] Banner text (body/16px, Tajawal)             |
|     explaining Whisper large-v3 usage             |
+--------------------------------------------------+
| [Provider dropdown — Parakeet REMOVED from list]  |
+--------------------------------------------------+
```

**Filtering rule:** When `uiLocale === 'ar'`, remove `'parakeet'` from `modelOptions` keys before rendering. The TypeScript union type is unchanged; only the rendered `<option>` / dropdown items are filtered.

### 2. Recording Gate (TRANS-03 / TRANS-04 shared)

**Location:** Main recording screen (`frontend/src/app/page.tsx`), replacing or wrapping the existing record button area.
**Condition:** Whisper `large-v3` model is not yet downloaded AND `uiLocale === 'ar'`.
**Primitives:** `<Button>` (disabled state) + `<Progress>` from `@/components/ui/progress`.

**Layout:**
```
+--------------------------------------------------+
|          [ Record Button — DISABLED ]             |
|     bg-muted, text-muted-foreground, min-h-11    |
|                                                   |
|  [=========>          ] 45%                       |
|  Progress bar (h-2, bg-primary fill)              |
|                                                   |
|  "Downloading Arabic transcription model..."      |
|  (small/14px, text-muted-foreground)              |
+--------------------------------------------------+
```

**States:**
| State | Record Button | Progress Bar | Status Text |
|-------|--------------|--------------|-------------|
| Downloading | Disabled, `bg-muted` | Visible, 0-100% | "جاري تحميل نموذج النسخ العربي ({N}%)..." |
| Download complete | Enabled, normal styling | Hidden | Hidden |
| Download error | Disabled, `bg-muted` | Hidden | "فشل تحميل النموذج. اضغط لإعادة المحاولة" + retry affordance |
| Model already present | Enabled, normal styling | Hidden | Hidden |

**Auto-enable:** When download completes, button transitions to enabled state without user action. Use Tauri `listen()` event pattern (mirrors existing `parakeet-model-download-complete`).

**Shared component:** The recording gate is a single component (`ModelDownloadGate` or similar) reusable in both onboarding-completion and locale-switch contexts. Same visual, same logic, different trigger.

### 3. Provider Repoint Notice (TRANS-04)

**Location:** Inside `LanguageConfirmDialog.tsx`, below the existing confirmation body text.
**Condition:** Target locale is `'ar'` (switching TO Arabic, not FROM).
**Primitive:** Additional `<p>` element inside the existing dialog body.

**Layout:**
```
+--------------------------------------------------+
| Change Language?                                  |
|                                                   |
| [Existing confirmation text]                      |
|                                                   |
| [Provider repoint notice — small/14px,            |
|  text-muted-foreground, italic styling]           |
|                                                   |
|        [Cancel]           [Confirm & Reload]      |
+--------------------------------------------------+
```

**Notice styling:**
- Font size: 14px (small)
- Color: `text-muted-foreground`
- Top margin: `mt-2` (8px gap from existing body text)
- No icon -- the notice is supplementary, not a warning

**One-direction rule:** The provider repoint notice appears ONLY when switching `en -> ar`. Switching `ar -> en` does NOT show the notice and does NOT auto-repoint the provider. This matches the Phase 1 invariant: Arabic + Parakeet is banned, English + any provider is fine.

---

## Copywriting Contract

All user-facing strings MUST be defined in `messages/ar.json` and `messages/en.json` via `next-intl`. No hardcoded strings.

| Element | Key Path | Arabic Copy | English Copy |
|---------|----------|-------------|--------------|
| Info banner (TRANS-02) | `transcriptSettings.arabicBanner` | يستخدم Meetily نموذج Whisper large-v3 للنسخ العربي لأنه يوفر أعلى دقة للغة العربية | Meetily uses Whisper large-v3 for Arabic transcription as it provides the highest accuracy for Arabic |
| Download progress (TRANS-03) | `recording.downloadingModel` | جاري تحميل نموذج النسخ العربي ({progress}%)... | Downloading Arabic transcription model ({progress}%)... |
| Download complete toast | `recording.modelReady` | نموذج النسخ العربي جاهز للاستخدام | Arabic transcription model ready to use |
| Download error | `recording.modelDownloadError` | فشل تحميل النموذج. اضغط لإعادة المحاولة | Model download failed. Tap to retry |
| Provider repoint notice (TRANS-04) | `languageConfirm.providerRepoint` | سيتم أيضاً تبديل مزود النسخ إلى Whisper large-v3 لدقة أعلى للعربية | The transcription provider will also be switched to Whisper large-v3 for higher Arabic accuracy |
| Empty state (no model, no download) | `recording.noArabicModel` | يجب تحميل نموذج Whisper large-v3 للنسخ العربي | Whisper large-v3 model is required for Arabic transcription |

**Destructive actions in this phase:** None. No delete, reset, or irreversible action is introduced. The locale switch confirmation already exists from Phase 2; Phase 4 only appends informational text to it.

---

## Interaction Contracts

### Banner Visibility
- Banner appears instantly when `uiLocale` is `'ar'` (no animation)
- Banner disappears instantly when `uiLocale` switches away from `'ar'`
- Banner is static -- no dismiss, no close button

### Download Progress
- Progress bar updates via Tauri event listener (`listen()` pattern)
- Progress percentage is integer, displayed as `{N}%`
- On download completion: progress bar and status text fade out (200ms `transition-opacity`), record button transitions to enabled state
- On download error: progress bar hidden, error text shown with retry affordance (tap/click on status text to retry)

### Record Button State Machine
```
[Page Mount]
  |
  +-- uiLocale !== 'ar' --> [Enabled] (no gate, normal behavior)
  |
  +-- uiLocale === 'ar'
        |
        +-- large-v3 present --> [Enabled]
        |
        +-- large-v3 absent
              |
              +-- download in progress --> [Disabled + Progress]
              |
              +-- download not started --> trigger download --> [Disabled + Progress]
              |
              +-- download failed --> [Disabled + Error + Retry]
```

### LanguageConfirmDialog Enhancement
- When target locale is `'ar'`: existing dialog body + provider repoint `<p>` appended
- When target locale is NOT `'ar'`: existing dialog body unchanged
- No additional confirmation step -- one "Confirm & Reload" button handles both locale switch and provider repoint atomically

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | `alert` (existing), `progress` (existing), `button` (existing), `dialog` (existing via LanguageConfirmDialog) | not required -- all already installed |

**No new shadcn components to install.** All primitives needed (`Alert`, `Progress`, `Button`, `Dialog`) are already present in `@/components/ui/`.

**No third-party registries.**

---

## Accessibility

| Surface | Requirement |
|---------|-------------|
| Info banner | `role="alert"` (inherited from `Alert` component), `aria-live="polite"` |
| Disabled record button | `aria-disabled="true"`, `aria-describedby` pointing to progress status text |
| Progress bar | `aria-label` with download status, `aria-valuenow` with percentage (inherited from Radix `Progress`) |
| Provider repoint notice | No special ARIA -- it is part of the dialog content, read naturally by screen readers |
| Download error retry | `role="button"` or `<button>` element, keyboard accessible |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
