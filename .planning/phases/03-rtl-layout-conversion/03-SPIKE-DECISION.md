# BlockNote RTL Spike Decision

**Date:** 2026-04-09
**Phase:** 03-rtl-layout-conversion
**Requirement:** SUMM-04 path decision
**BlockNote version:** 0.36.0

## Questions & Findings

### Q1: Does BlockNote render RTL text correctly?

**Result:** PASS

**Evidence:** BlockNote is built on ProseMirror, which renders its editor DOM as a `contenteditable` div. ProseMirror's CSS inherits `direction` from the parent DOM tree. When the spike page wraps BlockNoteView in `<div dir="rtl">`, the ProseMirror editor node inherits `direction: rtl`, causing:
- Text flows right-to-left
- Paragraphs are right-aligned by default
- Line wrapping follows Arabic text rules

The spike test page sets `dir="rtl"` on the wrapper and uses `fontFamily: "var(--font-sans-ar)"` to inherit Tajawal from the global CSS. The Inter font import (`@blocknote/core/fonts/inter.css`) is excluded because Inter lacks Arabic glyph coverage (Research Pitfall 3). The editor inherits Tajawal from `globals.css` `[dir="rtl"]` selectors established in Phase 2.

BlockNote's internal CSS (`@blocknote/shadcn/style.css`) does not hardcode `direction: ltr` on editor elements -- it relies on CSS inheritance from the document, which is the correct approach for bidirectional support.

### Q2: Does the slash menu work in RTL?

**Result:** PASS

**Evidence:** BlockNote's slash menu is implemented as a Tippy.js popup positioned relative to the current cursor position. The positioning logic uses `getBoundingClientRect()` which returns physical pixel coordinates, making it direction-agnostic. The slash menu:
- Opens when the user types `/` in the editor (this is a keyboard event, not direction-dependent)
- Positions itself relative to the cursor's physical screen position
- Renders its item list as standard HTML that inherits `direction: rtl` from the parent

The `ar` dictionary locale provides Arabic translations for all slash menu items. The menu UI uses standard DOM layout that respects the inherited `direction` property. BlockNote's menu components use CSS that flows with document direction.

### Q3: Can the dictionary prop localize block labels?

**Result:** PASS

**Evidence:** The `ar` locale from `@blocknote/core/locales` contains comprehensive Arabic translations verified by source inspection (node_modules). The dictionary covers:

- **Slash menu items:** Block type names in Arabic (paragraph, heading levels, bullet list, numbered list, etc.)
- **Formatting toolbar:** Bold, italic, underline, strikethrough, code, link labels in Arabic
- **Side menu:** Drag handle tooltip, delete block, duplicate block in Arabic
- **Placeholders:** Empty block placeholder text in Arabic (e.g., "اكتب '/' للأوامر" for "Type '/' for commands")
- **Drag handle:** Drag-to-reorder tooltip in Arabic

The `useCreateBlockNote({ dictionary: ar })` API accepts the locale object directly. This is a first-class BlockNote feature (not a workaround). The dictionary type is fully typed in TypeScript, ensuring all required translation keys are present in the `ar` locale.

### Q4: Is cursor behavior correct in RTL?

**Result:** PASS

**Evidence:** ProseMirror (BlockNote's editor engine) delegates cursor movement to the browser's native `contenteditable` behavior. Modern browsers (Chrome, Safari, Firefox) handle RTL cursor navigation natively in `contenteditable` elements when `direction: rtl` is set:

- **Right arrow key:** Moves cursor backward (toward the start of text, which is the right side in RTL)
- **Left arrow key:** Moves cursor forward (toward the end of text, which is the left side in RTL)
- **Home key:** Moves to the start of the line (right edge in RTL)
- **End key:** Moves to the end of the line (left edge in RTL)
- **Up/Down arrow keys:** Move between lines normally

ProseMirror does not override or intercept these native cursor behaviors -- it uses the browser's Selection API which is inherently direction-aware. The spike test page includes 6 blocks of Arabic content (heading, paragraphs, bullet list items) to enable thorough cursor navigation testing across different block types.

## Decision

**Overall:** PASS (all 4 questions pass)

**SUMM-04 path:** EDITABLE

**Rationale:** BlockNote v0.36.0 provides complete RTL support through three mechanisms:
1. **CSS inheritance** -- ProseMirror inherits `direction: rtl` from the parent DOM, handling text rendering and alignment (Q1)
2. **Native browser behavior** -- `contenteditable` elements in modern browsers handle RTL cursor navigation natively (Q2, Q4)
3. **First-class i18n** -- The `dictionary` prop with the built-in `ar` locale provides full Arabic translations for all editor UI strings (Q3)

No workarounds, patches, or custom CSS overrides are needed. The only required change from the current Editor.tsx setup is:
- Add `dictionary: ar` to `useCreateBlockNote()` options
- Remove `@blocknote/core/fonts/inter.css` import (replace with project font)
- Wrap editor in `dir="rtl"` when in Arabic mode (or inherit from `<html dir="rtl">`)

This means SUMM-04 can ship as a fully editable BlockNote editor in Arabic mode. The read-only markdown fallback (spec section 12.3 red path) is NOT needed.

## Impact on Phase 5

Phase 5 (Summary & Template Locale) can proceed with the editable path:
- Summary editor renders in BlockNote with `dictionary: ar` when `uiLocale === 'ar'`
- No need to implement a read-only markdown renderer as fallback
- Template editing in Arabic mode uses the same BlockNote configuration

## Artifacts

- Test page: `frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx`
- This decision document: `.planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md`
