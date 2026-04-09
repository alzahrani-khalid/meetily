# Phase 5: Templates & Prompts (Bilingual Content) - Context

**Gathered:** 2026-04-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Make all 6 meeting templates and all 5 LLM prompts available in both English and Arabic, resolved through a single locale-aware loader, with the pre-existing `defaults.rs` 2/6 embed gap fixed. Summary language is independent from UI locale — controlled by `preferences::read().summary_language`.

</domain>

<decisions>
## Implementation Decisions

### Arabic Content Authoring (D-01, D-02)
- **D-01:** Claude drafts Arabic template content in formal Modern Standard Arabic (MSA); Khalid reviews and approves before final embedding. This satisfies SUMM-03's "authored by a native speaker" requirement through the review gate.
- **D-02:** Language register is formal MSA — professional business language suitable for meeting minutes. General professional terminology throughout (no specialized medical/legal jargon in templates).

### Template Structure (D-03, D-04)
- **D-03:** Arabic templates use fully Arabic section headers and content. When `summary_language='ar'`, the entire output is Arabic — headers like "ملخص الاجتماع", "القرارات الرئيسية", "المهام المطلوبة" (not English headers with Arabic body text).
- **D-04:** Arabic templates mirror the exact same sections in the same order as English templates. Only the text content changes, not the structure. This keeps `section_id` alignment 1:1 across locales and simplifies the loader.

### LLM Prompt Strategy (D-05, D-06, D-07)
- **D-05:** Language of prompt instructions is Claude's discretion per prompt. The likely pattern: English instructions (better LLM comprehension across providers including Ollama) with explicit directive to respond in Arabic MSA and Arabic template headers as examples.
- **D-06:** Every Arabic prompt MUST include explicit Arabic punctuation instructions: "Use Arabic punctuation: `،` instead of `,`, `؛` instead of `;`, `؟` instead of `?`". This is load-bearing for SUMM-03.
- **D-07:** General professional terminology — no specialized glossaries needed. Templates cover: daily_standup, standard_meeting, project_sync, psychiatric_session, retrospective, sales_marketing_client_call.

### BlockNote Arabic Display (D-08)
- **D-08:** SUMM-04 path is EDITABLE (green path). Phase 3 spike confirmed BlockNote v0.36.0 has full RTL support via CSS inheritance + `dictionary: ar` + Tajawal font. No read-only markdown fallback needed. Configuration: `useCreateBlockNote({ dictionary: ar })` + wrapper `dir="rtl"` + exclude `@blocknote/core/fonts/inter.css`.

### Claude's Discretion
- Exact wording of each Arabic prompt (D-05) — Claude chooses optimal instruction language per prompt
- File naming for prompt `.txt` files (TPL-03 specifies `frontend/src-tauri/prompts/*.txt` but exact names are flexible)
- Order of `include_str!` declarations in `prompts/defaults.rs`

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design Specification
- `docs/superpowers/specs/2026-04-07-arabic-bilingual-support-design-v2.md` — Authoritative bilingual design spec; §2 (summary language independence), §7 (BlockNote spike), §12.3 (SUMM-04 fallback decision)

### Phase 3 Spike Decision
- `.planning/phases/03-rtl-layout-conversion/03-SPIKE-DECISION.md` — BlockNote RTL spike result: EDITABLE path confirmed. Documents all 4 test questions passing. Defines exact BlockNote configuration for Arabic mode.

### Existing Template System
- `frontend/src-tauri/src/summary/templates/loader.rs` — Current 3-tier template loader (custom → bundled → built-in). Must be extended with locale parameter.
- `frontend/src-tauri/src/summary/templates/defaults.rs` — Current 2/6 embed gap. Only `DAILY_STANDUP` and `STANDARD_MEETING` embedded. Must add all 6 × 2 locales.
- `frontend/src-tauri/src/summary/templates/types.rs` — Template schema: `Template { name, description, sections[] }`. Defines `to_markdown_structure()` and `to_section_instructions()`.
- `frontend/src-tauri/templates/` — 6 English template JSON files (the source content to mirror for Arabic)

### Inline Prompts to Externalize
- `frontend/src-tauri/src/summary/processor.rs` — Lines 215-216 (chunk summarizer), 281-282 (chunk combiner), 316+ (final template filler). These 5 inline strings become 10 externalized files (5 EN + 5 AR).

### Prior Phase Contracts
- `.planning/phases/01-preferences-foundation/01-CONTEXT.md` — `summary_language` preference is the resolution key (cross-phase contract line 236)
- `.planning/phases/02-i18n-framework-locale-bootstrap/02-CONTEXT.md` — `messages/{en,ar}.json` structure, `useTranslations()` pattern, no feature flags (D-20)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Template loader** (`templates/loader.rs`): 3-tier fallback chain (custom → bundled → built-in) — extend with `locale` parameter for `{id}.{locale}.json` resolution per TPL-01
- **Template types** (`templates/types.rs`): `Template`, `TemplateSection` structs with `to_markdown_structure()` and `to_section_instructions()` methods — no changes needed to the schema, just the content
- **defaults.rs pattern** (`templates/defaults.rs`): `include_str!()` + `get_builtin_templates()` + `get_builtin_template(id)` — mirror this pattern for prompts module
- **6 English template JSONs** (`frontend/src-tauri/templates/`): Complete content to use as basis for Arabic translations
- **BlockNote RTL spike** (`frontend/src/components/BlockNoteEditor/BlockNoteRTLSpike.tsx`): Working reference for Arabic BlockNote configuration

### Established Patterns
- **`include_str!()` for compile-time embedding**: All templates and prompts must be embedded for offline builds (TPL-02, TPL-04)
- **Preferences-based locale resolution**: `preferences::read().summary_language` returns `"en"` or `"ar"` — the key for template/prompt selection
- **Fallback chain**: TPL-01 requires `{id}.{locale}.json` → `{id}.json` fallback at every tier

### Integration Points
- **`processor.rs` call sites**: 5 inline prompt strings (lines 215, 216, 281, 282, 316+) must be replaced with `prompts::get_prompt(id, locale)` calls
- **`service.rs:74`**: `process_transcript_background()` accepts `template_id` — needs `locale` parameter added
- **Summary editor component**: Must conditionally pass `dictionary: ar` and `dir="rtl"` based on summary language

</code_context>

<specifics>
## Specific Ideas

- Arabic template headers should feel natural in a professional meeting context — "ملخص الاجتماع" not "الملخص" (too generic), "القرارات الرئيسية" not "قرارات" (too terse)
- Arabic punctuation enforcement via explicit prompt instruction is critical — LLMs (especially local Ollama models) tend to default to Latin punctuation even when writing Arabic
- The review workflow for Arabic content: Claude generates → writes to JSON files → Khalid reviews in PR or directly → adjustments committed before `include_str!` embedding

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-templates-prompts-bilingual-content*
*Context gathered: 2026-04-09*
