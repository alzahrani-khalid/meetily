# Phase 5: Templates & Prompts (Bilingual Content) - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-09
**Phase:** 05-templates-prompts-bilingual-content
**Areas discussed:** Arabic Content Authoring, Section Headers, Prompt Strategy, BlockNote Display

---

## Arabic Content Authoring

| Option | Description | Selected |
|--------|-------------|----------|
| Draft + Review (Recommended) | Claude drafts MSA Arabic, Khalid reviews before embedding | ✓ |
| User writes | Khalid writes all Arabic content personally | |
| Direct embed | Claude writes without review, QA in Phase 6 | |

**User's choice:** Draft + Review
**Notes:** Satisfies SUMM-03 native speaker requirement through review gate

---

| Option | Description | Selected |
|--------|-------------|----------|
| Formal MSA (Recommended) | Professional business language for meeting context | ✓ |
| Simplified MSA | Easier, more direct language | |

**User's choice:** Formal MSA
**Notes:** None

---

| Option | Description | Selected |
|--------|-------------|----------|
| General terminology sufficient | Standard professional Arabic terms | ✓ |
| Specific preferences | User-provided glossary for specialized templates | |

**User's choice:** General terminology
**Notes:** No specialized glossaries needed for any of the 6 templates

---

## Section Headers

| Option | Description | Selected |
|--------|-------------|----------|
| Fully Arabic (Recommended) | All headers + content in Arabic | ✓ |
| English headers + Arabic content | Mixed language output | |

**User's choice:** Fully Arabic
**Notes:** Headers like "ملخص الاجتماع", "القرارات الرئيسية", "المهام المطلوبة"

---

| Option | Description | Selected |
|--------|-------------|----------|
| Same sections exactly (Recommended) | Arabic mirrors English structure 1:1 | ✓ |
| Light modifications possible | Allow cultural adaptations to structure | |

**User's choice:** Same sections exactly
**Notes:** Simplifies loader — same section_id maps to same section across locales

---

## Prompt Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| English + Arabic response directive (Recommended) | English instructions, explicit "respond in Arabic MSA" | |
| Fully Arabic prompts | All instructions in Arabic | |
| Claude decides | Optimal language per prompt | ✓ |

**User's choice:** Claude decides
**Notes:** Likely pattern: English instructions for better LLM comprehension + explicit Arabic response directive

---

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, explicit instruction (Recommended) | Include Arabic punctuation rules in every Arabic prompt | ✓ |
| No, LLM handles it | Trust modern models to use Arabic punctuation | |

**User's choice:** Explicit instruction
**Notes:** Critical for SUMM-03; Ollama models tend to default to Latin punctuation

---

## BlockNote Display

| Option | Description | Selected |
|--------|-------------|----------|
| Editable editor (Recommended) | Full BlockNote with dictionary: ar + dir="rtl" + Tajawal | ✓ |
| Read-only Markdown | Rendered markdown without editing | |

**User's choice:** Editable editor
**Notes:** Phase 3 spike confirmed BlockNote v0.36.0 full RTL support. Green path (EDITABLE) locked.

---

## Claude's Discretion

- Exact wording/language of each Arabic prompt (D-05)
- File naming for prompt `.txt` files
- Order of `include_str!` declarations in `prompts/defaults.rs`

## Deferred Ideas

None — discussion stayed within phase scope
