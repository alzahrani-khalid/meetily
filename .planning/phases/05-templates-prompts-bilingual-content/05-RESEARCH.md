# Phase 5: Templates & Prompts (Bilingual Content) - Research

**Researched:** 2026-04-10
**Domain:** Rust template/prompt system + BlockNote RTL integration
**Confidence:** HIGH

## Summary

Phase 5 adds bilingual (EN/AR) support to the existing meeting template system (6 templates) and externalizes 5 inline LLM prompts from `processor.rs` into a new `prompts/` module. The work is primarily Rust-side: extending the 3-tier template loader with locale resolution, fixing the 2/6 `defaults.rs` embed gap, creating a parallel `prompts/` module with `include_str!` embedding, and threading `summary_language` from the preferences cache through the summary pipeline. On the frontend, the BlockNote editor needs conditional RTL configuration when displaying Arabic summaries.

The codebase is well-structured for this change. The existing `templates/` module (`loader.rs`, `defaults.rs`, `types.rs`) provides a clean pattern to mirror for the new `prompts/` module. The `PREFS_CACHE` already holds `summary_language` and is accessible synchronously. The BlockNote RTL spike (Phase 3) confirmed the editable path works with `dictionary: ar` + `dir="rtl"`.

**Primary recommendation:** Mirror the existing `templates/` module architecture for the new `prompts/` module. Thread locale as a parameter through `get_template(id, locale)` and `get_prompt(id, locale)`. Do NOT read preferences inside the loader -- pass locale explicitly from the call site.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Claude drafts Arabic template content in formal MSA; Khalid reviews and approves before final embedding.
- **D-02:** Language register is formal MSA -- professional business language suitable for meeting minutes. General professional terminology throughout.
- **D-03:** Arabic templates use fully Arabic section headers and content. When `summary_language='ar'`, the entire output is Arabic.
- **D-04:** Arabic templates mirror the exact same sections in the same order as English templates. Only the text content changes, not the structure. `section_id` alignment 1:1 across locales.
- **D-05:** Language of prompt instructions is Claude's discretion per prompt. Likely pattern: English instructions with explicit directive to respond in Arabic MSA with Arabic template headers as examples.
- **D-06:** Every Arabic prompt MUST include explicit Arabic punctuation instructions: "Use Arabic punctuation: comma instead of Latin comma, semicolon instead of Latin semicolon, question mark instead of Latin question mark".
- **D-07:** General professional terminology -- no specialized glossaries needed.
- **D-08:** SUMM-04 path is EDITABLE (green path). Phase 3 spike confirmed BlockNote v0.36.0 has full RTL support via CSS inheritance + `dictionary: ar` + Tajawal font. No read-only markdown fallback needed. Configuration: `useCreateBlockNote({ dictionary: ar })` + wrapper `dir="rtl"` + exclude `@blocknote/core/fonts/inter.css`.

### Claude's Discretion
- Exact wording of each Arabic prompt (D-05) -- Claude chooses optimal instruction language per prompt
- File naming for prompt `.txt` files (TPL-03 specifies `frontend/src-tauri/prompts/*.txt` but exact names are flexible)
- Order of `include_str!` declarations in `prompts/defaults.rs`

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TPL-01 | Template loader resolves `{id}.{locale}.json` first, falls back to `{id}.json`, at every tier | Loader architecture pattern (Section: Architecture Patterns) |
| TPL-02 | All 6 meeting templates embedded in `defaults.rs`, fixing the 2/6 embed gap | defaults.rs pattern documented; all 6 JSON files confirmed present in `templates/` |
| TPL-03 | 5 LLM prompts externalized from `processor.rs` to `prompts/*.txt` with new `summary/prompts/` module | Prompt inventory (Section: Architecture Patterns, Pattern 2); processor.rs lines 215-216, 281-282, 316+ |
| TPL-04 | `processor.rs` reads prompts via `prompts::get_prompt(id, locale)` + all 10 prompt files embedded via `include_str!` | Prompts module design mirrors template module exactly |
| SUMM-01 | Summary in Arabic regardless of UI locale (summary language independent from UI language) | `PREFS_CACHE.summary_language` threading documented |
| SUMM-02 | All 6 templates available in both EN and AR | 6 EN templates exist; AR templates created as `{id}.ar.json` |
| SUMM-03 | Arabic summaries with native MSA phrasing and proper Arabic punctuation | D-06 punctuation enforcement in prompts; D-01 native speaker review gate |
| SUMM-04 | Arabic summary viewable in BlockNote editor (editable path) | Spike decision confirmed; Editor.tsx + BlockNoteSummaryView.tsx integration points documented |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde + serde_json | (already in Cargo.toml) | Template/prompt JSON parsing | Already used throughout codebase [VERIFIED: codebase grep] |
| include_str! | Rust built-in | Compile-time file embedding | Existing pattern in `defaults.rs` [VERIFIED: codebase] |
| @blocknote/core | 0.36.0 | Editor with RTL support | Locked by Phase 3 spike decision [VERIFIED: spike decision] |
| @blocknote/react | 0.36.0 | React bindings for BlockNote | Already installed [VERIFIED: codebase imports] |
| @blocknote/shadcn | 0.36.0 | BlockNote UI theme | Already installed [VERIFIED: codebase imports] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| @blocknote/core/locales | 0.36.0 | `ar` dictionary for BlockNote | When rendering Arabic summaries in editor [VERIFIED: spike decision Q3] |
| preferences module | (internal) | `PREFS_CACHE.read().summary_language` | To resolve locale at summary generation time [VERIFIED: codebase] |

**No new dependencies needed.** All libraries already present. [VERIFIED: codebase analysis]

## Architecture Patterns

### Pattern 1: Locale-Aware Template Resolution (TPL-01)

**What:** Extend `loader.rs` to accept a `locale` parameter and resolve `{id}.{locale}.json` before `{id}.json` at each tier.

**Current call chain:**
```
commands.rs:221 → SummaryService::process_transcript_background(template_id)
  → service.rs:75 → processor::generate_meeting_summary(template_id)
    → processor.rs:309 → templates::get_template(template_id)
      → loader.rs:95 → custom → bundled → builtin fallback
```

**New call chain (add `locale` parameter):**
```
commands.rs:221 → SummaryService::process_transcript_background(template_id, locale)
  → service.rs:75 → processor::generate_meeting_summary(template_id, locale)
    → processor.rs:309 → templates::get_template(template_id, locale)
      → loader.rs → custom({id}.{locale}.json → {id}.json)
                   → bundled({id}.{locale}.json → {id}.json)
                   → builtin({id}.{locale} → {id})
```

**Key design:** The `locale` parameter is threaded explicitly from the call site. At `commands.rs`, read `PREFS_CACHE.read().summary_language` and pass it down. Do NOT read preferences inside the loader -- this keeps the loader pure and testable. [ASSUMED]

**Loader resolution per tier:**
```rust
// For each tier (custom, bundled, builtin):
// 1. Try {id}.{locale}.json (e.g., daily_standup.ar.json)
// 2. Fall back to {id}.json (e.g., daily_standup.json)
fn load_custom_template(template_id: &str, locale: &str) -> Option<String> {
    let custom_dir = get_custom_templates_dir()?;
    // Try locale-specific first
    let locale_path = custom_dir.join(format!("{}.{}.json", template_id, locale));
    if let Ok(content) = std::fs::read_to_string(&locale_path) {
        return Some(content);
    }
    // Fall back to non-locale
    let base_path = custom_dir.join(format!("{}.json", template_id));
    std::fs::read_to_string(&base_path).ok()
}
```

### Pattern 2: Prompts Module (TPL-03, TPL-04)

**What:** Create a `summary/prompts/` module that mirrors `summary/templates/` structure.

**5 inline prompts to externalize from processor.rs:**

| # | Location | Current Content | Prompt ID |
|---|----------|----------------|-----------|
| 1 | Line 215 (system) | "You are an expert meeting summarizer." | `chunk_summarizer_system` |
| 2 | Line 216 (user) | "Provide a concise but comprehensive summary..." | `chunk_summarizer_user` |
| 3 | Line 281 (system) | "You are an expert at synthesizing meeting summaries." | `chunk_combiner_system` |
| 4 | Line 282 (user) | "The following are consecutive summaries..." | `chunk_combiner_user` |
| 5 | Lines 316+ (system) | Large final template-filling prompt | `final_report_system` |

[VERIFIED: processor.rs lines 215-216, 281-282, 316+]

**File structure:**
```
frontend/src-tauri/
├── prompts/
│   ├── chunk_summarizer_system.en.txt
│   ├── chunk_summarizer_system.ar.txt
│   ├── chunk_summarizer_user.en.txt
│   ├── chunk_summarizer_user.ar.txt
│   ├── chunk_combiner_system.en.txt
│   ├── chunk_combiner_system.ar.txt
│   ├── chunk_combiner_user.en.txt
│   ├── chunk_combiner_user.ar.txt
│   ├── final_report_system.en.txt
│   └── final_report_system.ar.txt
├── src/summary/prompts/
│   ├── mod.rs          # Re-exports
│   ├── defaults.rs     # include_str! for all 10 files
│   └── loader.rs       # get_prompt(id, locale) with fallback
```

**Module API:**
```rust
// summary/prompts/loader.rs
pub fn get_prompt(prompt_id: &str, locale: &str) -> Result<&'static str, String> {
    // Try locale-specific first, fall back to English
    defaults::get_builtin_prompt(prompt_id, locale)
        .or_else(|| defaults::get_builtin_prompt(prompt_id, "en"))
        .ok_or_else(|| format!("Prompt '{}' not found", prompt_id))
}
```

### Pattern 3: Arabic Template JSON Structure (SUMM-02, D-03, D-04)

**What:** Create 6 Arabic template JSON files mirroring English structure exactly.

**Example -- `daily_standup.ar.json`:**
```json
{
  "name": "الاجتماع اليومي",
  "description": "تحديثات يومية موجزة لفرق الهندسة والمنتجات.",
  "sections": [
    { "title": "التاريخ", "instruction": "YYYY-MM-DD", "format": "string" },
    { "title": "الحاضرون", "instruction": "قائمة المشاركين الحاضرين", "format": "list" },
    { "title": "أمس", "instruction": "ما تم إنجازه بالأمس (نقاط مختصرة)", "format": "list",
      "example_item_format": "| **المسؤول** | **العمل المنجز** |\n| --- | --- |" },
    { "title": "اليوم", "instruction": "العمل المخطط لليوم (نقاط مختصرة)", "format": "list",
      "example_item_format": "| **المسؤول** | **العمل المخطط** |\n| --- | --- |" },
    { "title": "العوائق", "instruction": "أي عقبات والمسؤول عنها إن عُرف", "format": "list",
      "item_format": "| **المسؤول** | **العائق** | الأثر |\n| --- | --- | --- |" },
    { "title": "ملاحظات", "instruction": "ملاحظات سريعة أو إعلانات اختيارية", "format": "paragraph" }
  ]
}
```

**Key:** `sections[]` order and `format` values are identical to English. Only `name`, `description`, `title`, `instruction`, `item_format`, `example_item_format` text content is translated. [VERIFIED: D-04 locked decision]

### Pattern 4: BlockNote Arabic Configuration (SUMM-04)

**What:** Conditionally configure BlockNote for Arabic when summary language is Arabic.

**Current Editor.tsx:**
```typescript
import "@blocknote/core/fonts/inter.css";  // REMOVE for Arabic
const editor = useCreateBlockNote({ initialContent });
return <BlockNoteView editor={editor} editable={true} theme="light" />;
```

**New pattern:**
```typescript
import { ar } from "@blocknote/core/locales";
// Conditionally import inter.css only for non-Arabic

const editor = useCreateBlockNote({
  initialContent,
  ...(isArabic ? { dictionary: ar } : {}),
});

return (
  <div dir={isArabic ? "rtl" : "ltr"}>
    <BlockNoteView editor={editor} editable={true} theme="light" />
  </div>
);
```

[VERIFIED: Phase 3 spike decision -- all 4 questions passed]

**Integration point:** `BlockNoteSummaryView.tsx` creates its own `useCreateBlockNote` instance (line 84) and renders `BlockNoteView` directly (lines 243-256 for markdown format). Both creation sites need the Arabic configuration. The `isArabic` flag comes from the summary's language, not from `uiLocale` (SUMM-01 independence).

### Pattern 5: Locale Threading Through Summary Pipeline

**What:** Thread `summary_language` from preferences through the entire summary call chain.

**Call chain to modify:**
1. `commands.rs:221` -- Read `PREFS_CACHE.read().summary_language`, pass to `process_transcript_background`
2. `service.rs:75` -- Accept `locale` param, pass to `generate_meeting_summary`
3. `processor.rs` -- Accept `locale` param, use for `templates::get_template(id, locale)` and `prompts::get_prompt(id, locale)`

**Frontend side:** The frontend must also know which language the summary was generated in, to configure BlockNote correctly. Options:
- Store `summary_language` in the meeting/summary DB record at generation time
- Or pass it as metadata alongside the summary response

[ASSUMED -- exact mechanism for frontend to know summary language needs planner decision]

### Recommended Project Structure
```
frontend/src-tauri/
├── prompts/                        # NEW: 10 prompt .txt files
│   ├── chunk_summarizer_system.en.txt
│   ├── chunk_summarizer_system.ar.txt
│   ├── chunk_summarizer_user.en.txt
│   ├── chunk_summarizer_user.ar.txt
│   ├── chunk_combiner_system.en.txt
│   ├── chunk_combiner_system.ar.txt
│   ├── chunk_combiner_user.en.txt
│   ├── chunk_combiner_user.ar.txt
│   ├── final_report_system.en.txt
│   └── final_report_system.ar.txt
├── templates/                      # EXISTING: add 6 Arabic JSONs
│   ├── daily_standup.json          # existing EN
│   ├── daily_standup.ar.json       # NEW AR
│   ├── standard_meeting.json
│   ├── standard_meeting.ar.json
│   ├── project_sync.json
│   ├── project_sync.ar.json
│   ├── psychatric_session.json
│   ├── psychatric_session.ar.json
│   ├── retrospective.json
│   ├── retrospective.ar.json
│   ├── sales_marketing_client_call.json
│   └── sales_marketing_client_call.ar.json
├── src/summary/
│   ├── prompts/                    # NEW module
│   │   ├── mod.rs
│   │   ├── defaults.rs
│   │   └── loader.rs
│   ├── templates/                  # EXISTING: modify
│   │   ├── defaults.rs             # Fix 2/6 gap + add AR
│   │   ├── loader.rs              # Add locale parameter
│   │   ├── mod.rs
│   │   └── types.rs               # No changes needed
│   ├── processor.rs               # Replace inline prompts
│   ├── service.rs                 # Thread locale
│   └── commands.rs                # Read PREFS_CACHE, pass locale
```

### Anti-Patterns to Avoid
- **Reading preferences inside loader:** Loader should be a pure function of `(id, locale)`. Reading `PREFS_CACHE` inside creates hidden coupling and makes testing harder.
- **Separate Arabic template struct:** Do NOT create a different Rust struct for Arabic templates. Same `Template` struct, different JSON content. D-04 locks this.
- **Conditional compilation for locale:** Do NOT use `#[cfg]` for locale. All locales are always compiled in. Runtime resolution only.
- **Hardcoding prompts in Rust source:** The whole point of TPL-03/TPL-04 is externalization to `.txt` files. Never inline prompt text in `.rs` files.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Template JSON parsing | Custom parser | `serde_json::from_str::<Template>` | Already working, validated, tested |
| Compile-time embedding | Build script or runtime loading | `include_str!()` macro | Zero-cost, offline-safe, existing pattern |
| BlockNote RTL | Custom RTL CSS hacks | `dictionary: ar` + `dir="rtl"` inheritance | Spike proved this is first-class support |
| Arabic punctuation in output | Post-processing replacer | Explicit prompt instructions (D-06) | LLM-native approach, more reliable |
| Locale detection | Custom logic | `PREFS_CACHE.read().summary_language` | Already implemented in Phase 1 |

## Common Pitfalls

### Pitfall 1: defaults.rs Include Path Depth
**What goes wrong:** `include_str!` paths are relative to the source file, not the project root. Getting the `../` depth wrong causes compile errors.
**Why it happens:** `defaults.rs` is at `src/summary/templates/defaults.rs`, so templates at `templates/*.json` need `../../../templates/` prefix.
**How to avoid:** Current `defaults.rs` already uses `include_str!("../../../templates/daily_standup.json")`. Mirror this exact depth for all new includes. For the `prompts/defaults.rs` at `src/summary/prompts/defaults.rs`, the path to `prompts/*.txt` is `../../../prompts/`.
**Warning signs:** Compile error mentioning file not found in `include_str!`. [VERIFIED: existing defaults.rs line 7]

### Pitfall 2: Template Name/Description in to_markdown_structure
**What goes wrong:** `to_markdown_structure()` and `to_section_instructions()` on `types.rs` use `section.title` directly in the generated markdown. If the template is Arabic, these methods automatically produce Arabic headers -- which is correct (D-03). But the `to_section_instructions()` hard-codes English instruction preamble text on line 86-88.
**Why it happens:** The instruction preamble `"For the main title..."` is hardcoded in English in `types.rs`.
**How to avoid:** The `final_report_system` prompt (externalized to `.txt` file) should contain locale-aware instructions that override or complement `to_section_instructions()`. The planner must decide whether to modify `to_section_instructions()` to accept locale, or to handle this entirely in the prompt file.
**Warning signs:** Mixed English/Arabic in the LLM system prompt. [VERIFIED: types.rs lines 86-88]

### Pitfall 3: BlockNote Inter Font Override
**What goes wrong:** `@blocknote/core/fonts/inter.css` overrides the font family, making Arabic text render in Inter (which lacks Arabic glyphs) instead of Tajawal.
**Why it happens:** The import is at the top of `Editor.tsx` (line 6) and applies globally.
**How to avoid:** Conditionally import Inter only when NOT in Arabic mode. Or remove it entirely and rely on project fonts from `globals.css`. The spike decision explicitly says "exclude `@blocknote/core/fonts/inter.css`". [VERIFIED: Editor.tsx line 6, spike decision line 75]

### Pitfall 4: Final Report Prompt Has Dynamic Template Injection
**What goes wrong:** The `final_report_system` prompt (processor.rs lines 316-334) is not a simple static string -- it uses `format!()` to inject `section_instructions` and `clean_template_markdown` dynamically.
**Why it happens:** The prompt is a template itself, with `{}` placeholders for the template's section instructions and markdown structure.
**How to avoid:** The externalized `final_report_system.{locale}.txt` must contain placeholders (e.g., `{section_instructions}` and `{template_markdown}`) that `processor.rs` fills in at runtime. This prompt cannot be purely static. [VERIFIED: processor.rs lines 316-334]

### Pitfall 5: Locale Must Be Stored With Summary
**What goes wrong:** Summary is generated in Arabic but frontend doesn't know to enable RTL in BlockNote.
**Why it happens:** Currently `summary_language` is only in preferences, not stored with the summary record.
**How to avoid:** When generating a summary, store the `summary_language` value alongside it in the database (or derive it from the summary content). The frontend reads this to decide `dictionary: ar` vs default. [ASSUMED]

### Pitfall 6: User Prompt Template Has {} Placeholder
**What goes wrong:** The `chunk_summarizer_user` prompt template uses `{}` as a placeholder for the transcript chunk (line 216). If externalized as-is, Rust's `format!()` will still need to replace it.
**Why it happens:** The current code does `.replace("{}", chunk.as_str())` -- this is fragile if the prompt text itself contains literal `{}`.
**How to avoid:** Use named placeholders like `{transcript_chunk}` in externalized prompts and replace with a proper templating approach (e.g., `.replace("{transcript_chunk}", chunk)`). [VERIFIED: processor.rs line 228]

## Code Examples

### Template defaults.rs -- Fixed 2/6 Gap + Arabic (TPL-02)

```rust
// src/summary/templates/defaults.rs
// All 6 EN templates
pub const DAILY_STANDUP_EN: &str = include_str!("../../../templates/daily_standup.json");
pub const STANDARD_MEETING_EN: &str = include_str!("../../../templates/standard_meeting.json");
pub const PROJECT_SYNC_EN: &str = include_str!("../../../templates/project_sync.json");
pub const PSYCHATRIC_SESSION_EN: &str = include_str!("../../../templates/psychatric_session.json");
pub const RETROSPECTIVE_EN: &str = include_str!("../../../templates/retrospective.json");
pub const SALES_MARKETING_EN: &str = include_str!("../../../templates/sales_marketing_client_call.json");

// All 6 AR templates
pub const DAILY_STANDUP_AR: &str = include_str!("../../../templates/daily_standup.ar.json");
pub const STANDARD_MEETING_AR: &str = include_str!("../../../templates/standard_meeting.ar.json");
pub const PROJECT_SYNC_AR: &str = include_str!("../../../templates/project_sync.ar.json");
pub const PSYCHATRIC_SESSION_AR: &str = include_str!("../../../templates/psychatric_session.ar.json");
pub const RETROSPECTIVE_AR: &str = include_str!("../../../templates/retrospective.ar.json");
pub const SALES_MARKETING_AR: &str = include_str!("../../../templates/sales_marketing_client_call.ar.json");

pub fn get_builtin_template(id: &str, locale: &str) -> Option<&'static str> {
    match (id, locale) {
        ("daily_standup", "ar") => Some(DAILY_STANDUP_AR),
        ("daily_standup", _) => Some(DAILY_STANDUP_EN),
        ("standard_meeting", "ar") => Some(STANDARD_MEETING_AR),
        ("standard_meeting", _) => Some(STANDARD_MEETING_EN),
        ("project_sync", "ar") => Some(PROJECT_SYNC_AR),
        ("project_sync", _) => Some(PROJECT_SYNC_EN),
        ("psychatric_session", "ar") => Some(PSYCHATRIC_SESSION_AR),
        ("psychatric_session", _) => Some(PSYCHATRIC_SESSION_EN),
        ("retrospective", "ar") => Some(RETROSPECTIVE_AR),
        ("retrospective", _) => Some(RETROSPECTIVE_EN),
        ("sales_marketing_client_call", "ar") => Some(SALES_MARKETING_AR),
        ("sales_marketing_client_call", _) => Some(SALES_MARKETING_EN),
        _ => None,
    }
}
```

### Prompt defaults.rs (TPL-04)

```rust
// src/summary/prompts/defaults.rs
pub const CHUNK_SUMMARIZER_SYSTEM_EN: &str = include_str!("../../../prompts/chunk_summarizer_system.en.txt");
pub const CHUNK_SUMMARIZER_SYSTEM_AR: &str = include_str!("../../../prompts/chunk_summarizer_system.ar.txt");
// ... (10 total constants)

pub fn get_builtin_prompt(id: &str, locale: &str) -> Option<&'static str> {
    match (id, locale) {
        ("chunk_summarizer_system", "ar") => Some(CHUNK_SUMMARIZER_SYSTEM_AR),
        ("chunk_summarizer_system", _) => Some(CHUNK_SUMMARIZER_SYSTEM_EN),
        // ... all 5 prompt IDs x 2 locales
        _ => None,
    }
}
```

### Processor.rs Prompt Replacement (TPL-04)

```rust
// Before (inline):
let system_prompt_chunk = "You are an expert meeting summarizer.";

// After (externalized):
let locale = locale; // passed from caller
let system_prompt_chunk = prompts::get_prompt("chunk_summarizer_system", locale)
    .unwrap_or("You are an expert meeting summarizer.");
let user_prompt_template = prompts::get_prompt("chunk_summarizer_user", locale)
    .unwrap_or("Provide a concise...");
let user_prompt_chunk = user_prompt_template.replace("{transcript_chunk}", chunk.as_str());
```

### BlockNote Arabic Integration (SUMM-04)

```typescript
// BlockNoteSummaryView.tsx -- conditional Arabic config
import { ar } from "@blocknote/core/locales";

// Determine if summary is Arabic (from summary metadata, not UI locale)
const isArabicSummary = summaryLanguage === "ar";

const editor = useCreateBlockNote({
  initialContent: undefined,
  ...(isArabicSummary ? { dictionary: ar } : {}),
});

// Wrap in RTL div
<div dir={isArabicSummary ? "rtl" : "ltr"}>
  <BlockNoteView editor={editor} editable={true} theme="light" />
</div>
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust `#[cfg(test)]` + `cargo test` |
| Config file | `frontend/src-tauri/Cargo.toml` |
| Quick run command | `cd frontend/src-tauri && cargo test summary::templates -- --nocapture` |
| Full suite command | `cd frontend/src-tauri && cargo test -- --nocapture` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| TPL-01 | Locale-aware template resolution with fallback | unit | `cargo test summary::templates::tests -- --nocapture` | Partial (extend existing) |
| TPL-02 | All 12 templates (6 EN + 6 AR) parse as valid JSON | unit | `cargo test summary::templates::defaults::tests -- --nocapture` | Partial (extend existing) |
| TPL-03 | All 10 prompt files embedded and retrievable | unit | `cargo test summary::prompts::tests -- --nocapture` | Wave 0 |
| TPL-04 | `get_prompt(id, locale)` returns correct content | unit | `cargo test summary::prompts::tests -- --nocapture` | Wave 0 |
| SUMM-01 | Locale parameter independent of UI locale | unit | Test that `get_template("x", "ar")` works regardless of other state | Wave 0 |
| SUMM-02 | All 6 templates available in both locales | unit | `cargo test summary::templates::tests::test_all_templates_both_locales` | Wave 0 |
| SUMM-03 | Arabic prompts include punctuation instructions | unit | Assert prompt content contains Arabic punctuation chars | Wave 0 |
| SUMM-04 | BlockNote Arabic config | manual | Visual check of RTL rendering | manual-only (UI) |

### Wave 0 Gaps
- [ ] `src/summary/prompts/mod.rs` -- new module with tests for prompt loading
- [ ] Extended `src/summary/templates/loader.rs` tests for locale fallback
- [ ] Test for all 6 x 2 = 12 template JSON validity
- [ ] Test for all 5 x 2 = 10 prompt file embedding

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | N/A |
| V3 Session Management | no | N/A |
| V4 Access Control | no | N/A |
| V5 Input Validation | yes | Template JSON validated by `serde_json::from_str` + `Template::validate()` |
| V6 Cryptography | no | N/A |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malicious custom template JSON | Tampering | `Template::validate()` enforces schema; `serde_json` rejects malformed input |
| Prompt injection via custom prompt files | Tampering | Custom prompts not supported (only builtin embedded); user `custom_prompt` is wrapped in `<user_context>` tags |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Locale parameter should be threaded explicitly (not read from PREFS_CACHE inside loader) | Architecture Pattern 1 | Minor -- just a design preference; either approach works |
| A2 | Summary language should be stored with the summary record for frontend to know | Pitfall 5 | MEDIUM -- frontend needs to know which language to configure BlockNote for; if not stored, would need another mechanism |

## Open Questions

1. **How does frontend know summary language?**
   - What we know: Summary is generated with a locale. Frontend needs to configure BlockNote RTL accordingly.
   - What's unclear: Whether to store `summary_language` in the summary DB record, or derive it from preferences at display time.
   - Recommendation: Store it with the summary record -- preferences can change between generation and viewing.

2. **to_section_instructions() English preamble**
   - What we know: `types.rs` line 86-88 hardcodes English instruction text.
   - What's unclear: Whether to localize `to_section_instructions()` or handle it entirely in the externalized prompt.
   - Recommendation: Handle in the externalized `final_report_system` prompt -- it already wraps the section instructions. The prompt itself provides the locale context.

## Sources

### Primary (HIGH confidence)
- Codebase analysis: `loader.rs`, `defaults.rs`, `types.rs`, `processor.rs`, `service.rs`, `commands.rs`, `mod.rs`
- Codebase analysis: `Editor.tsx`, `BlockNoteSummaryView.tsx`
- Phase 3 spike decision: `.planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md`
- Phase 5 CONTEXT.md: `.planning/phases/05-templates-prompts-bilingual-content/05-CONTEXT.md`

### Secondary (MEDIUM confidence)
- Template JSON files in `frontend/src-tauri/templates/` (all 6 confirmed present)
- Preferences module `PREFS_CACHE` pattern verified in `preferences/mod.rs`

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all libraries already in codebase, no new dependencies
- Architecture: HIGH - mirrors existing proven patterns exactly
- Pitfalls: HIGH - all verified against actual source code
- Arabic content: MEDIUM - D-01 requires native speaker review of Claude-drafted Arabic

**Research date:** 2026-04-10
**Valid until:** 2026-05-10 (stable -- no external dependency changes expected)
