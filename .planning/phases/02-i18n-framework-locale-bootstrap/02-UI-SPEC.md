---
phase: 2
slug: i18n-framework-locale-bootstrap
status: draft
shadcn_initialized: true
preset: "new-york / neutral / cssVariables / lucide"
created: 2026-04-08
revised: 2026-04-08
milestone: v1.0 — Arabic Bilingual Support
requirements: [UI-01, UI-02, UI-03, UI-04]
upstream_sources:
  - docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md
  - .planning/PROJECT.md (Active requirements, Key Decisions)
  - .planning/REQUIREMENTS.md (UI-01..UI-04)
  - .planning/ROADMAP.md (Phase 2 goal + success criteria)
  - .planning/phases/01-preferences-foundation/01-01-SUMMARY.md (preferences primitives available)
  - frontend/components.json (shadcn preset)
  - frontend/tailwind.config.ts (existing tokens)
  - frontend/src/app/layout.tsx (current root layout)
  - frontend/src/app/globals.css (existing CSS vars + scrollbar)
  - CLAUDE.md (Web/Tailwind RTL rules — logical properties only)
---

# Phase 2 — UI Design Contract

> Visual and interaction contract for the **i18n Framework & Locale Bootstrap** phase.
> This is the foundation for every Arabic-facing screen that follows. Phase 3 (RTL conversion),
> Phase 4 (Arabic transcription UI), Phase 5 (templates/prompts), and Phase 6 (Rust strings)
> all inherit the decisions locked here.

---

## Phase Intent

Bootstrap Meetily's bilingual UI: wire `next-intl` (client-provider mode), load the Arabic Tajawal font, switch `<html lang dir>` from preferences, add a first-run `navigator.language` detector with an `en` fallback, and gate the entire provider tree behind a `BootSplash` until `get_user_preferences` resolves. Every later phase calls into this contract — if the boot ordering, font strategy, or direction switch is wrong here, every RTL surface downstream inherits the bug.

**Hard rule (from CLAUDE.md Web/Tailwind section + spec §4):** this phase touches only **logical Tailwind primitives** (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`, `border-s-*`, `border-e-*`, `rounded-s-*`, `rounded-e-*`). No `ml-*`/`mr-*`/`pl-*`/`pr-*`/`text-left`/`text-right` may be introduced. The ESLint guardrail that enforces this lives in Phase 3; Phase 2 must not regress the rule by writing any new physical-direction classes.

---

## Design System

Detected from `frontend/components.json` + `frontend/tailwind.config.ts` — no re-initialization, no new registry.

| Property | Value | Source |
|----------|-------|--------|
| Tool | shadcn (already initialized) | `frontend/components.json` |
| Style preset | `new-york` | `components.json:style` |
| Base color | `neutral` | `components.json:tailwind.baseColor` |
| CSS variables | `true` | `components.json:tailwind.cssVariables` |
| Component library | Radix (via shadcn/ui) | existing `@/components/ui/*` |
| Icon library | `lucide-react` | `components.json:iconLibrary` |
| Primary font (EN) | `Source Sans 3` via `next/font/google` | `src/app/layout.tsx:30`, CSS var `--font-source-sans-3` |
| Primary font (AR) | **Tajawal** via `next/font/google` | NEW this phase — CSS var `--font-tajawal` |
| i18n library | `next-intl` (`NextIntlClientProvider` — client mode only) | spec §2, PROJECT Key Decisions |
| Locale store | SQLite `user_preferences` via `preferences::` Rust module | Phase 1 (shipped) |
| Direction contract | `dir={uiLocale === 'ar' ? 'rtl' : 'ltr'}` on `<html>` | spec §3.3 |

**Why no shadcn gate rerun:** Phase 1 already shipped with `components.json` present. Phase 2 introduces no new shadcn blocks — it only wires `next-intl`, a font, a boot splash, and a language switch inside the existing `SettingsModal`. Registry safety gate does not apply.

---

## Spacing Scale

Inherit Tailwind's default 4px scale. Phase 2 surfaces (BootSplash, Settings language row, reload toast) do not need an 8-point override — shadcn new-york already ships 4/8/12/16/20/24/32/48/64 via Tailwind defaults and we follow the stricter GSD 4/8/16/24/32/48/64 subset for the new surfaces.

| Token | Value | Usage in Phase 2 |
|-------|-------|------------------|
| xs | 4px | Gap between locale flag icon and label in the Settings row |
| sm | 8px | Inner padding of the BootSplash spinner container |
| md | 16px | Default gap between the BootSplash logo, title, and tagline; padding inside `LanguageRow` |
| lg | 24px | Vertical rhythm inside the Settings "Language" card / BootSplash container |
| xl | 32px | BootSplash full-bleed vertical centering margin; confirmation dialog padding |
| 2xl | 48px | (reserved — not used by Phase 2 surfaces) |
| 3xl | 64px | BootSplash top/bottom margin on full-screen layouts |

**Exceptions:** none. Every new surface in Phase 2 must use the tokens above, expressed in logical-property Tailwind classes (`ps-*`, `pe-*`, `ms-*`, `me-*`).

**Touch targets:** minimum 44×44 for the language switcher trigger and the "Confirm & Reload" button (`min-h-11 min-w-11` or explicit `h-11`).

---

## Typography

Phase 2's declared type contract is scoped to exactly **4 font sizes** and **2 font weights**, chosen from the existing `frontend/tailwind.config.ts:20-27` token set. Phase 2 ships only two surfaces (BootSplash and the Settings language section) — this subset covers both without any token additions.

> **Phase 2 contract scopes to 4 sizes / 2 weights. The existing 18px `text-h2` token and 700 `font-bold` weight are out-of-phase tokens not used by Phase 2 surfaces.** They remain in `tailwind.config.ts` unchanged (no token edits this phase); Phase 2 simply does not consume them. Phase 3+ may re-enter them under their own contracts.

### Declared size contract (exactly 4)

| Role | Size | Weight | Line Height | Token | Used by (Phase 2) |
|------|------|--------|-------------|-------|-------------------|
| Display (BootSplash wordmark) | 32px | 600 | 1.2 | `text-display` | BootSplash "Meetily" wordmark |
| Heading (h1 / section + dialog) | 24px | 600 | 1.3 | `text-h1` | Settings "Language" section heading; `LanguageConfirmDialog` title |
| Body (default) | 16px | 400 | 1.6 | `text-body` | Dialog body copy; option Label text; switcher CTA button label |
| Label / UI small | 14px | 400 | 1.5 | `text-small` | Settings description; BootSplash tagline; muted secondary copy |

**Phase 2 size inventory (exactly 4):** `14 / 16 / 24 / 32`.

### Declared weight contract (exactly 2)

- **Regular 400** — body, labels, descriptions, captions, BootSplash tagline
- **Semibold 600** — all headings (h1), dialog titles, button labels, and the **BootSplash wordmark** (`text-display` is rendered at `font-semibold`, overriding the token's default 700 for this phase's usage)

**Phase 2 weight inventory (exactly 2):** `400 / 600`.

### Out-of-phase tokens (exist in tailwind.config.ts, NOT used by Phase 2)

| Token / value | Why not used in Phase 2 |
|---------------|-------------------------|
| `text-h2` (18px / 500) | Phase 2's only sub-section need is the Settings "Language" heading, which consolidates to `text-h1` 24px and differentiates via weight (600) and vertical spacing instead of introducing a second heading size. |
| `font-bold` 700 | `text-display` token defaults to 700, but Phase 2 explicitly renders the BootSplash wordmark at `font-semibold` (600). The wordmark remains the visual focal point through its 32px display size, not its weight. |
| `text-caption` (12px / 400) | Not used by any Phase 2 surface; smallest size in Phase 2 is `text-small` (14px). |
| `font-medium` 500 | Not used by any Phase 2 surface; all mid-weight emphasis collapses to semibold (600). |

**Important:** Do NOT modify `tailwind.config.ts` in Phase 2. The token file legitimately contains 6 sizes and multiple weights for the whole app — this is fine. Phase 2's design contract narrows *which tokens are consumed* for the two surfaces this phase ships, nothing more.

### Line-height policy

Body 1.5–1.6, headings 1.2–1.3. **Arabic needs extra headroom** — Tajawal ascenders + diacritics push ~1.1–1.2x English height. All new text containers must use the declared token line-height; **never** lower than 1.4 for body when `dir="rtl"`.

### Font Loading (NEW — authoritative contract)

Both fonts load via `next/font/google` (mirrors `layout.tsx:30`). Both are always loaded — even in English mode — so the user never sees a FOIT when switching to Arabic from Settings. They expose distinct CSS variables.

```ts
// frontend/src/app/layout.tsx (new, at module scope alongside sourceSans3)
import { Source_Sans_3, Tajawal } from 'next/font/google'

const sourceSans3 = Source_Sans_3({
  subsets: ['latin'],
  weight: ['400', '600'],       // Phase 2 contract = 2 weights only
  variable: '--font-source-sans-3',
  display: 'swap',
})

const tajawal = Tajawal({
  subsets: ['arabic'],          // arabic subset — do NOT load latin; Source Sans 3 covers EN
  weight: ['400', '500'],       // Tajawal's 600 equivalent ships as 500 in Google Fonts; treat as semibold
  variable: '--font-tajawal',
  display: 'swap',
})
```

> **Note on Tajawal weights:** Google Fonts' Tajawal family does not publish a 600 weight — its semibold equivalent is `500`. The CSS `font-weight: 600` declaration on headings will map to Tajawal 500 via the browser's nearest-match, which renders as the family's semibold. This does not violate the 2-weight contract (the contract is expressed in CSS weight values 400/600, not font-file weights).

### Applied font family (driven by `dir` attribute, not a className)

`globals.css` declares a single `font-sans` that branches on the `[dir]` attribute so **no** component needs to know the current locale. This keeps the 65+ existing `.tsx` files out of the font logic.

```css
/* globals.css — add to @layer base */
:root {
  --font-sans-en: var(--font-source-sans-3), ui-sans-serif, system-ui, sans-serif;
  --font-sans-ar: var(--font-tajawal), var(--font-source-sans-3), ui-sans-serif, system-ui, sans-serif;
}

html[dir="ltr"] body { font-family: var(--font-sans-en); }
html[dir="rtl"] body { font-family: var(--font-sans-ar); }
```

**Invariant:** Arabic text MUST NEVER render in Source Sans 3 (no Arabic glyphs), and English text MUST NEVER render in Tajawal. The `[dir]` selector makes this structural — no component-level font prop is allowed.

**BlockNote is explicitly out of scope for this phase.** BlockNote imports `@blocknote/core/fonts/inter.css` at `BlockNoteEditor/Editor.tsx:6`. Inter has no Arabic glyphs. Do NOT attempt to fix BlockNote here — the spike in Phase 3 (spec §7) decides that outcome.

---

## Color

Inherit existing CSS variables from `src/app/globals.css:80-106`. Phase 2 adds **zero new colors**. The entire palette is reused and the 60/30/10 discipline is defined against Phase 2 surfaces (BootSplash, Settings language row, confirmation dialog, reload toast).

| Role | Value (light) | Value (dark) | Usage |
|------|---------------|--------------|-------|
| Dominant 60% — Background | `hsl(0 0% 100%)` (`--background`) | `hsl(0 0% 3.9%)` | BootSplash canvas, Settings modal body, app chrome |
| Secondary 30% — Muted/Card | `hsl(0 0% 96.1%)` (`--muted` / `--card`) | `hsl(0 0% 14.9%)` | Settings rows, confirm-dialog card, subtle borders |
| Accent 10% — Primary | `hsl(221 83% 53%)` (`tailwind.config.ts:primary`, blue-600) | same | **Reserved-for list below** |
| Destructive | `hsl(0 84% 60%)` (`tailwind.config.ts:destructive`, red-500) | `hsl(0 62.8% 30.6%)` | NOT USED in Phase 2 (no destructive surfaces in this phase) |
| Foreground | `hsl(0 0% 3.9%)` (`--foreground`) | `hsl(0 0% 98%)` | All text |
| Muted foreground | `hsl(0 0% 45.1%)` (`--muted-foreground`) | `hsl(0 0% 63.9%)` | BootSplash tagline, Settings secondary copy |

**Accent (blue-600) is reserved for, and ONLY for, in this phase:**
1. BootSplash spinner stroke/ring
2. "Confirm & Reload" primary button in the language-switch confirmation dialog
3. The active radio indicator next to the selected language in the Settings language switcher
4. The focus ring on the language switcher trigger button (matches shadcn `focus-visible:ring` default)

Accent must NOT be used for: section dividers, icon fills, text labels, hover backgrounds, or "informational" banners. If a reviewer sees blue on any surface not on the list above, it is a contract violation.

**Dark mode:** all CSS variables already have `.dark` overrides in `globals.css:108-133`. Phase 2 introduces nothing that breaks them.

---

## Copywriting Contract

Phase 2's user-visible copy is small but load-bearing — it's the first Arabic the user ever sees. All strings land in `frontend/src/messages/en.json` and `frontend/src/messages/ar.json`. Arabic translations must use **MSA** (`،` `؛` `؟` punctuation) and be written by a native speaker per spec §6.5. No inline string literals; every user-facing string in this phase goes through `useTranslations()`.

### Message catalogue (Phase 2 delivers both languages)

| Key | English | Arabic (MSA) |
|-----|---------|--------------|
| `boot.loading` (BootSplash tagline) | "Preparing your meeting assistant…" | "جاري تجهيز مساعد اجتماعاتك…" |
| `boot.appName` (BootSplash display) | "Meetily" | "Meetily" (brand retained untransliterated) |
| `settings.language.sectionTitle` | "Language" | "اللغة" |
| `settings.language.description` | "Choose the language Meetily uses for its interface. Switching will reload the app." | "اختر لغة واجهة Meetily. سيؤدي التبديل إلى إعادة تشغيل التطبيق." |
| `settings.language.option.en` | "English" | "الإنجليزية" |
| `settings.language.option.ar` | "العربية" | "العربية" |
| `settings.language.currentLabel` | "Current language" | "اللغة الحالية" |
| `settings.language.switchCta` | "Switch to {lang}" | "التبديل إلى {lang}" |
| `settings.language.confirm.title` | "Switch Meetily to {lang}?" | "تبديل Meetily إلى {lang}؟" |
| `settings.language.confirm.body` | "Meetily will reload to apply the new language. Any in-progress recording must be stopped first." | "سيتم إعادة تشغيل Meetily لتطبيق اللغة الجديدة. يجب إيقاف أي تسجيل قيد التقدم أولاً." |
| `settings.language.confirm.primaryCta` | "Confirm & Reload" | "تأكيد وإعادة التشغيل" |
| `settings.language.confirm.cancelCta` | "Cancel" | "إلغاء" |
| `settings.language.confirm.recordingBlocker` | "Stop the current recording before switching language." | "أوقف التسجيل الحالي قبل تبديل اللغة." |
| `settings.language.empty` | (n/a — this phase has no empty state) | — |
| `settings.language.error.persistFailed` | "Couldn't save language preference. Try again or restart Meetily." | "تعذر حفظ تفضيل اللغة. حاول مرة أخرى أو أعد تشغيل Meetily." |
| `settings.language.error.bootstrapFailed` | "Couldn't load preferences. Starting in English — set your language in Settings." | "تعذر تحميل التفضيلات. بدء التشغيل بالإنجليزية — اضبط اللغة من الإعدادات." |

### Standard required entries

| Element | Copy |
|---------|------|
| **Primary CTA (phase)** | `settings.language.confirm.primaryCta` — "Confirm & Reload" / "تأكيد وإعادة التشغيل" |
| **Empty state** | **N/A.** Phase 2 surfaces render only when preferences exist (or have just been defaulted by the bootstrap detector). The BootSplash is the "loading" equivalent; there is no dataless state because preferences always resolve to a row. |
| **Error state (bootstrap path)** | `settings.language.error.bootstrapFailed` — app falls back to `en`, user is told why, and the language row in Settings is the recovery path. |
| **Error state (persistence path)** | `settings.language.error.persistFailed` — surfaced via the existing `sonner` toast (`layout.tsx:9, 279`). |
| **Destructive actions** | **None.** Switching locale is **not** destructive — it's a full reload. The confirmation dialog exists to prevent accidental reloads mid-recording, not because data is lost. No `variant="destructive"` buttons in Phase 2. |

### Placeholder semantics

- `{lang}` is always resolved through `useTranslations()` + the **target** language's native name (`ar.json` → `العربية`, `en.json` → `English`). Never "switch to ar" or "switch to en".
- Use `Intl.DisplayNames(locale, { type: 'language' })` as the fallback source if the catalogue is missing a language name; do not hardcode "Arabic" / "English" strings anywhere outside the catalogue.

---

## Component Inventory (new or modified this phase)

All components live under `frontend/src/` and use Tailwind logical-property classes only.

| Component | Path | Status | Purpose |
|-----------|------|--------|---------|
| `BootSplash` | `frontend/src/components/BootSplash.tsx` | NEW | Blocking loader until `get_user_preferences` resolves. Prevents LTR-English flash (UI-01 success criterion 4, ROADMAP phase 2 criterion 4). |
| `LanguageSwitcher` | `frontend/src/components/settings/LanguageSwitcher.tsx` | NEW | Row inside `SettingsModal` with the two language options, a radio-group visual, and the "Confirm & Reload" flow. Consumes `useTranslations('settings.language')`. |
| `LanguageConfirmDialog` | `frontend/src/components/settings/LanguageConfirmDialog.tsx` | NEW | shadcn `AlertDialog` wrapping the confirm/cancel + recording-blocker path. Uses `useRecordingState()` to disable confirm while recording. |
| `I18nProvider` | `frontend/src/providers/I18nProvider.tsx` | NEW | Wraps `NextIntlClientProvider` + exposes `uiLocale`. Sits between `ConfigProvider` and the rest of the tree (spec §3.3). |
| `messages/en.json` | `frontend/src/messages/en.json` | NEW | English message catalogue — all keys above plus any existing UI strings migrated opportunistically for the new surfaces only. |
| `messages/ar.json` | `frontend/src/messages/ar.json` | NEW | Arabic MSA message catalogue — same shape as en.json. |
| `RootLayout` (modified) | `frontend/src/app/layout.tsx` | MODIFIED | Add Tajawal font, add preferences bootstrap `useEffect` + `BootSplash` gate, change `<html lang="en">` to `<html lang={uiLocale} dir={...} suppressHydrationWarning>`, wrap children in `I18nProvider`. Hoist the existing onboarding logic below the provider tree. |
| `globals.css` (modified) | `frontend/src/app/globals.css` | MODIFIED | Add `--font-sans-en` / `--font-sans-ar` CSS variables and the `html[dir]` font-family selectors. |
| `preferencesService.ts` (consumer only) | `frontend/src/services/preferencesService.ts` | UNCHANGED | Already shipped in Phase 1. Phase 2 reads `uiLocale` from `getUserPreferences()` during bootstrap and writes via `setUserPreferences({ uiLocale })`. |

**shadcn blocks consumed:** `AlertDialog`, `Button`, `RadioGroup`, `Label` — all from the official shadcn/ui registry (`@/components/ui/*`). No third-party registries.

---

## Locale Bootstrap Sequence (authoritative order)

This order is the phase's load-bearing contract. Phase 2 gates everything else behind it. Every downstream phase can rely on these invariants.

```
1. App mount: <html lang="en" dir="ltr" suppressHydrationWarning>   ← placeholder, NEVER seen (gated)
2. RootLayout state: { initialPreferences: null, detectedLocale: null }
3. BootSplash renders (full-bleed, no sidebar, no main content mounted)
4. useEffect: invoke<UserPreferences>('get_user_preferences')
     ├─ success path:
     │    ├─ if row has a persisted uiLocale → use it as-is
     │    └─ if row is still at seed default ('en') AND this is first run:
     │         ├─ read navigator.language
     │         ├─ if startsWith('ar') → setUserPreferences({ uiLocale: 'ar' })
     │         ├─ else                → keep 'en' (no write needed)
     │         └─ use resolved locale
     └─ error path:
          ├─ fall back to 'en'
          ├─ toast settings.language.error.bootstrapFailed
          └─ still unblock the provider tree so user can recover from Settings
5. setInitialPreferences(prefs) + set <html lang={uiLocale} dir={...}>
6. Mount ConfigProvider → I18nProvider(NextIntlClientProvider) → rest of tree
7. BootSplash unmounts
```

**First-run detection logic (UI-01 acceptance):**
- "First run" = the preferences row exists (created by Phase 1 migration as `id='1'` with `ui_locale='en'`) AND no prior `uiLocale` write has been persisted.
- Phase 1's seed writes `ui_locale='en'`, so we cannot distinguish "user chose English" from "never ran before" at the row level. **Decision:** add a `bootstrapped` boolean column **via a lightweight companion migration in Phase 2** (default `0`). The bootstrap detector sets it to `1` the first time it runs successfully (whether it kept `en` or switched to `ar`). This is the ONE schema change Phase 2 makes, and it lives in the same `preferences::repository` module Phase 1 shipped — no new module.
- If the schema change is judged out-of-scope for a UI phase during planning, the fallback is: always run detection on every launch that has `ui_locale='en'`, on the theory that a user who deliberately chose English will see no behavior change (detection re-confirms `en`). The planner decides. The UI contract is unchanged either way.

**Switch flow (UI-02, UI-03 — full reload, not hot-swap):**
```
1. User opens Settings → Language row
2. User clicks "Switch to العربية" (or "Switch to English")
3. LanguageConfirmDialog opens
   └─ if recording: disable confirm button, show settings.language.confirm.recordingBlocker
4. User clicks "Confirm & Reload"
5. await setUserPreferences({ uiLocale: 'ar' })
   └─ This also auto-repoints transcript provider via the Phase 1 invariant (TRANS-04 Phase 1 scope)
6. window.location.reload()   ← full reload, matches spec §12.4 and Phase 1 onboarding reload pattern (layout.tsx:230)
7. Next mount: bootstrap resolves to persisted 'ar', BootSplash shows Arabic tagline,
   tree mounts with NextIntlClientProvider(locale='ar'), <html dir="rtl">
```

**No hot-swap. No mid-session i18n updates. No optimistic UI.** Every locale change goes through a reload.

---

## BootSplash Visual Contract

The BootSplash is the phase's most visible new surface. Its job is to look intentional — not "loading screen" — because the user sees it on **every cold start** (even ~50ms ones).

| Attribute | Value |
|-----------|-------|
| Layout | Full-bleed centered flex column, `min-h-screen`, `flex items-center justify-center` |
| Background | `bg-background` (dominant) |
| Direction | Fixed LTR during splash (we don't know the locale yet). Text inside uses `text-center`, which is RTL-safe. |
| Content (vertical order) | 1. App wordmark "Meetily" (`text-display` size, explicitly `font-semibold` — 600 weight, overriding `text-display`'s token default of 700 to honor the Phase 2 2-weight contract) <br> 2. `gap-4` (16px) <br> 3. Spinner (Lucide `Loader2` with `animate-spin`, `w-6 h-6`, `text-primary`) <br> 4. `gap-2` (8px) <br> 5. Tagline `boot.loading` (`text-small text-muted-foreground`, `font-normal` 400) |
| Copy language | Defaults to English during bootstrap (Arabic catalogue isn't loaded yet). Once preferences resolve, if this re-renders the Arabic tagline will be used — but typical case is the splash dismounts before the re-render. The contract accepts that the tagline may briefly read English even on an Arabic first-run — this is acceptable because the splash is non-chrome and the first "real" UI paint is Arabic. |
| Minimum visible time | 150ms artificial minimum to prevent flash-on-fast-machines; implemented via `Promise.all([prefsPromise, new Promise(r => setTimeout(r, 150))])`. |
| Maximum acceptable time | 2000ms before the error toast triggers and the splash falls back to `en`. |
| Accessibility | `role="status"` on the container, `aria-live="polite"` on the tagline, visually-hidden "Loading Meetily" on the spinner. |

**Wordmark weight note:** The `text-display` Tailwind token in `tailwind.config.ts:21` declares `fontWeight: '700'` as its default. Phase 2 intentionally renders the wordmark with an **explicit** `font-semibold` class (600) to stay within the 2-weight contract. The visual focal role of the wordmark is carried by its 32px display **size**, not by a bold weight. Do NOT modify the token itself — just apply `font-semibold` on the element.

**Explicit non-goals:** no progress bar, no percentage, no multi-step list. This is a splash, not an installer.

---

## Settings Language Switcher — Visual Contract

Lives inside the existing `SettingsModal.tsx` (spec cross-reference: `app/_components/SettingsModal.tsx:229` — already the Language region). Phase 2 does **not** redesign the modal — it adds one "Language" section above the existing model/provider settings.

| Attribute | Value |
|-----------|-------|
| Section heading | `settings.language.sectionTitle` as an h2 element, styled with `text-h1` (24px / 600) — Phase 2 consolidates all headings to the h1 size and differentiates sub-sections via vertical spacing (`mt-6` / `mb-2`) and weight (600) rather than a second size. |
| Description | `settings.language.description` as body copy (`text-small text-muted-foreground font-normal`), `mt-1` |
| Option list | shadcn `RadioGroup` with 2 items; `gap-3` between rows |
| Option row | `flex items-center gap-3 rounded-md border p-4` — logical properties only |
| Option content order (JSX left-to-right) | `<RadioGroupItem>` → `<Label>` with language name (`text-body font-normal`) → optional locale tag (`text-small font-normal`). Because we use logical alignment and the dir switch happens at `<html>`, the native name sits at the **start** edge of the row in both locales. |
| Selected state | Accent ring (`ring-2 ring-primary`), `border-primary`, checked radio dot uses accent |
| Unselected state | `border-input` (default shadcn), hover `bg-muted/50` |
| Action button | shadcn `Button` variant `default` (primary), `text-body font-semibold`, `h-11` (touch target), label = `settings.language.switchCta` with `{lang}` = the *other* language's native name. Disabled when the current locale matches the selection. |

**Section heading rationale:** Earlier drafts used `text-h2` (18px / 500) for the section heading. Phase 2's 4-size / 2-weight contract eliminates both the 18px size and the 500 weight, so the section heading is promoted to `text-h1` (24px / 600). Visual differentiation from the dialog title (also `text-h1`) is handled by context (section vs. dialog) and vertical spacing, not a separate size.

**RTL correctness verification checklist for the Language section:**
1. Does every class in `LanguageSwitcher.tsx` use logical properties? (`ps-*`, `pe-*`, `ms-*`, `me-*`, `text-start`, `border-s-*`…)
2. Is there any `ml-*`, `mr-*`, `pl-*`, `pr-*`, `text-left`, `text-right`, `border-l-*`, `border-r-*`, `rounded-l-*`, `rounded-r-*`? **If yes, remove.**
3. Is any `flexDirection` or `flex-row-reverse` used? **If yes, remove** — rely on the `dir` attribute.
4. Is there any hard-coded "English" or "Arabic" string outside `messages/*.json`? **If yes, move to the catalogue.**
5. Does the confirm button become disabled when a recording is active? (`useRecordingState`)
6. Does any element use `text-h2`, `font-medium`, or `font-bold`? **If yes, switch to the 4-size / 2-weight contract** (14/16/24/32 sizes; 400/600 weights).

The checker (`gsd-ui-checker`) will grep the new files for physical-direction classes and for out-of-phase typography tokens. Zero tolerance.

---

## Provider Tree (after Phase 2)

The existing tree in `layout.tsx:236-277` is preserved. Phase 2 inserts two new layers:

```
<html lang={uiLocale} dir={uiLocale === 'ar' ? 'rtl' : 'ltr'} suppressHydrationWarning>
  <body className={`${sourceSans3.variable} ${tajawal.variable} font-sans antialiased`}>
    {initialPreferences ? (
      <AnalyticsProvider>
        <RecordingStateProvider>
          <TranscriptProvider>
            <ConfigProvider initialPreferences={initialPreferences}>   ← Phase 1 shipped initialPreferences wiring
              <I18nProvider locale={uiLocale} messages={messages[uiLocale]}>   ← NEW
                <OllamaDownloadProvider>
                  … (unchanged Phase 1 tree)
                </OllamaDownloadProvider>
              </I18nProvider>
            </ConfigProvider>
          </TranscriptProvider>
        </RecordingStateProvider>
      </AnalyticsProvider>
    ) : (
      <BootSplash />    ← NEW, blocking
    )}
    <Toaster position="bottom-center" richColors closeButton />
  </body>
</html>
```

**Placement rationale:** `I18nProvider` sits **inside** `ConfigProvider` because `ConfigProvider` owns `uiLocale` as the source of truth (from Phase 1). The i18n provider takes a `locale` prop from context and must remount on locale change — but since locale change triggers a **full reload**, the remount is free.

**Toaster stays outside** the i18n provider because `sonner` uses its own copy via `toast('...')` call sites. Phase 2 updates all new `toast(...)` calls inside the new components to use `t('…')` from `useTranslations` — pre-existing toast strings are not migrated in this phase (they belong to Phase 3/Phase 6 per ROADMAP).

---

## Interaction States (phase-wide checklist)

Every new surface must define these five states. Missing any is a checker failure.

| State | BootSplash | LanguageSwitcher row | Confirm Dialog |
|-------|-----------|---------------------|----------------|
| Default | Wordmark + spinner + tagline | Unchecked radio, neutral border | Body copy, primary CTA enabled |
| Hover | N/A (no interactive elements) | `bg-muted/50` | Primary CTA darker blue (shadcn default) |
| Focus | N/A | `ring-2 ring-ring ring-offset-2` (shadcn default) | `ring-2 ring-ring` on CTA |
| Loading | Spinner (animate-spin) | N/A | Spinner inside primary CTA while `setUserPreferences` is in flight (`<Loader2 className="animate-spin" />`), CTA text becomes `settings.language.confirm.primaryCta` + spinner |
| Error | Toast `error.bootstrapFailed` + fall through to `en` | Toast `error.persistFailed`, selection reverts | Toast `error.persistFailed`, dialog stays open with CTA re-enabled |
| Disabled | N/A | Current-language radio is visually selected; CTA reads `settings.language.currentLabel` with `cursor-default` | Confirm CTA disabled during recording, explanation line shows `error.recordingBlocker` |

---

## Accessibility Contract

| Requirement | Implementation |
|-------------|----------------|
| `<html lang>` matches active locale | Driven by `uiLocale` state — never hardcoded. |
| `<html dir>` matches active locale | `dir={uiLocale === 'ar' ? 'rtl' : 'ltr'}`. |
| Focus ring visible on all interactive Phase 2 elements | shadcn default `focus-visible:ring-2 ring-ring ring-offset-2` — inherited by Button, RadioGroup. |
| Keyboard reachable | Tab order: BootSplash (N/A), LanguageSwitcher (section → radio 1 → radio 2 → CTA → close). Confirm dialog: auto-focus primary CTA, Esc closes. |
| Minimum touch target | 44×44 (`h-11`) on primary CTA and switcher trigger. |
| Screen reader labels | BootSplash: `role="status"`, `aria-live="polite"`. Confirm dialog: `aria-labelledby` pointing at the title, `aria-describedby` pointing at the body. |
| Contrast | All existing shadcn neutral palette ratios. Accent blue-600 on white ≈ 4.55:1 ≥ AA for non-large text. |
| RTL mirroring of icons | Lucide directional icons (`ChevronLeft`, `ChevronRight`, `ArrowLeft`, `ArrowRight`) are NOT used in Phase 2's new components. If any are added during implementation, they must be mirrored via `rtl:-scale-x-100` or branched on `dir`. |
| Arabic font diacritic clearance | Body line-height ≥ 1.5. Never use `leading-none` or `leading-tight` in Arabic contexts. |

---

## Registry Safety

| Registry | Blocks Used | Safety Gate |
|----------|-------------|-------------|
| shadcn official | `AlertDialog`, `Button`, `RadioGroup`, `Label` | not required |
| (no third-party registries) | — | not applicable |

**No third-party blocks vetted or included in this phase.** If the planner introduces one during `/gsd-plan-phase 2`, the vetting gate in `gsd-ui-researcher` must rerun before this contract is re-approved.

---

## Out of Scope for Phase 2 (ignore even if tempting)

These items belong to downstream phases. Do **not** bundle them into Phase 2 plans.

| Out-of-scope item | Belongs to |
|-------------------|------------|
| Converting the 286 directional Tailwind hits across the 10 hotspot files | Phase 3 |
| ESLint `no-restricted-syntax` rule banning physical-direction classes | Phase 3 |
| BlockNote RTL spike and editor font fix | Phase 3 |
| Sidebar collapse `translate-x` direction branch | Phase 3 |
| Parakeet-ban UI filter, onboarding fork, ready-to-record gate | Phase 4 |
| Template/prompt locale resolution + Arabic prompt authoring | Phase 5 |
| Tray menu + notification Arabic strings | Phase 6 |
| Arabic UAT / QA regression pass | Phase 6 |
| Migrating existing non-Phase-2 component strings into message catalogues | Phase 3+ (opportunistically, not here) |
| Hot-swap locale switching | Permanently out of scope (spec §9, §12.4) |
| Per-component language switching | Permanently out of scope (spec §9) |
| `navigator.language` → seed `summary_language` / `transcription_language` | Out of scope — only `ui_locale` is seeded from `navigator.language`. The other two stay at their Phase 1 defaults. |
| Editing `tailwind.config.ts` typography tokens | Phase 2 narrows *which* tokens are consumed; it does not edit the token file. Any token removal is a future-phase decision. |

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS (revised 2026-04-08 — scoped to 4 sizes / 2 weights)
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending

---

## Traceability

| Requirement | Covered by this spec section |
|-------------|------------------------------|
| UI-01 (first-run `navigator.language` detection + `en` fallback) | "Locale Bootstrap Sequence" step 4, "BootSplash Visual Contract" |
| UI-02 (Settings language switch) | "Settings Language Switcher" section, "Copywriting Contract" keys |
| UI-03 (full reload on switch, persisted across restarts) | "Locale Bootstrap Sequence" switch flow, "Copywriting Contract" `confirm.body` |
| UI-04 (Tajawal loaded via `next/font/google`, Arabic in Tajawal / English in Source Sans 3) | "Font Loading" section, "Applied font family" CSS contract |

---

## Revision History

| Date | Change | Reason |
|------|--------|--------|
| 2026-04-08 | Initial draft | Phase 2 UI contract, committed `78f0cbf` |
| 2026-04-08 | Typography section revised to 4 sizes / 2 weights; BootSplash wordmark weight changed from 700 to 600; Settings section heading consolidated from `text-h2` 18px to `text-h1` 24px | Checker blocking issues: Dim 4 Typography had 5 sizes (max 4) and 3 weights (max 2). `tailwind.config.ts` untouched — contract narrows consumed tokens only. |

---

*UI-SPEC drafted: 2026-04-08 by gsd-ui-researcher for Phase 2. Revised 2026-04-08 for checker typography blocks. Upstream: spec v2 (2026-04-07), Phase 1 SUMMARY, PROJECT.md Key Decisions, existing shadcn + Tailwind config.*
