/// Embedded default prompts using compile-time inclusion
///
/// These prompts are bundled into the binary and serve as the built-in
/// prompt content for LLM-based meeting summarization.

/// Chunk summarizer system prompt (English)
pub const CHUNK_SUMMARIZER_SYSTEM_EN: &str = include_str!("../../../prompts/chunk_summarizer_system.en.txt");
/// Chunk summarizer system prompt (Arabic)
pub const CHUNK_SUMMARIZER_SYSTEM_AR: &str = include_str!("../../../prompts/chunk_summarizer_system.ar.txt");

/// Chunk summarizer user prompt (English)
pub const CHUNK_SUMMARIZER_USER_EN: &str = include_str!("../../../prompts/chunk_summarizer_user.en.txt");
/// Chunk summarizer user prompt (Arabic)
pub const CHUNK_SUMMARIZER_USER_AR: &str = include_str!("../../../prompts/chunk_summarizer_user.ar.txt");

/// Chunk combiner system prompt (English)
pub const CHUNK_COMBINER_SYSTEM_EN: &str = include_str!("../../../prompts/chunk_combiner_system.en.txt");
/// Chunk combiner system prompt (Arabic)
pub const CHUNK_COMBINER_SYSTEM_AR: &str = include_str!("../../../prompts/chunk_combiner_system.ar.txt");

/// Chunk combiner user prompt (English)
pub const CHUNK_COMBINER_USER_EN: &str = include_str!("../../../prompts/chunk_combiner_user.en.txt");
/// Chunk combiner user prompt (Arabic)
pub const CHUNK_COMBINER_USER_AR: &str = include_str!("../../../prompts/chunk_combiner_user.ar.txt");

/// Final report system prompt (English)
pub const FINAL_REPORT_SYSTEM_EN: &str = include_str!("../../../prompts/final_report_system.en.txt");
/// Final report system prompt (Arabic)
pub const FINAL_REPORT_SYSTEM_AR: &str = include_str!("../../../prompts/final_report_system.ar.txt");

/// Get a built-in prompt by identifier and locale
///
/// # Arguments
/// * `id` - Prompt identifier (e.g., "chunk_summarizer_system")
/// * `locale` - Locale code ("ar" for Arabic, anything else falls back to English)
///
/// # Returns
/// The prompt content if the identifier is known, None otherwise
pub fn get_builtin_prompt(id: &str, locale: &str) -> Option<&'static str> {
    match (id, locale) {
        ("chunk_summarizer_system", "ar") => Some(CHUNK_SUMMARIZER_SYSTEM_AR),
        ("chunk_summarizer_system", _) => Some(CHUNK_SUMMARIZER_SYSTEM_EN),
        ("chunk_summarizer_user", "ar") => Some(CHUNK_SUMMARIZER_USER_AR),
        ("chunk_summarizer_user", _) => Some(CHUNK_SUMMARIZER_USER_EN),
        ("chunk_combiner_system", "ar") => Some(CHUNK_COMBINER_SYSTEM_AR),
        ("chunk_combiner_system", _) => Some(CHUNK_COMBINER_SYSTEM_EN),
        ("chunk_combiner_user", "ar") => Some(CHUNK_COMBINER_USER_AR),
        ("chunk_combiner_user", _) => Some(CHUNK_COMBINER_USER_EN),
        ("final_report_system", "ar") => Some(FINAL_REPORT_SYSTEM_AR),
        ("final_report_system", _) => Some(FINAL_REPORT_SYSTEM_EN),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_embedded_prompts_are_non_empty() {
        assert!(!CHUNK_SUMMARIZER_SYSTEM_EN.is_empty(), "chunk_summarizer_system.en is empty");
        assert!(!CHUNK_SUMMARIZER_SYSTEM_AR.is_empty(), "chunk_summarizer_system.ar is empty");
        assert!(!CHUNK_SUMMARIZER_USER_EN.is_empty(), "chunk_summarizer_user.en is empty");
        assert!(!CHUNK_SUMMARIZER_USER_AR.is_empty(), "chunk_summarizer_user.ar is empty");
        assert!(!CHUNK_COMBINER_SYSTEM_EN.is_empty(), "chunk_combiner_system.en is empty");
        assert!(!CHUNK_COMBINER_SYSTEM_AR.is_empty(), "chunk_combiner_system.ar is empty");
        assert!(!CHUNK_COMBINER_USER_EN.is_empty(), "chunk_combiner_user.en is empty");
        assert!(!CHUNK_COMBINER_USER_AR.is_empty(), "chunk_combiner_user.ar is empty");
        assert!(!FINAL_REPORT_SYSTEM_EN.is_empty(), "final_report_system.en is empty");
        assert!(!FINAL_REPORT_SYSTEM_AR.is_empty(), "final_report_system.ar is empty");
    }

    #[test]
    fn test_all_arabic_prompts_contain_arabic_comma() {
        let ar_prompts = [
            ("chunk_summarizer_system", CHUNK_SUMMARIZER_SYSTEM_AR),
            ("chunk_summarizer_user", CHUNK_SUMMARIZER_USER_AR),
            ("chunk_combiner_system", CHUNK_COMBINER_SYSTEM_AR),
            ("chunk_combiner_user", CHUNK_COMBINER_USER_AR),
            ("final_report_system", FINAL_REPORT_SYSTEM_AR),
        ];
        for (id, content) in ar_prompts {
            assert!(
                content.contains('\u{060C}'),
                "Arabic prompt '{}' missing Arabic comma (،)",
                id
            );
        }
    }

    #[test]
    fn test_get_builtin_prompt_returns_correct_locale() {
        // English
        let en = get_builtin_prompt("chunk_summarizer_system", "en").unwrap();
        assert!(en.contains("expert meeting summarizer"));
        assert!(!en.contains('\u{060C}'), "EN prompt should not contain Arabic comma");

        // Arabic
        let ar = get_builtin_prompt("chunk_summarizer_system", "ar").unwrap();
        assert!(ar.contains('\u{060C}'), "AR prompt should contain Arabic comma");
    }

    #[test]
    fn test_get_builtin_prompt_unknown_id() {
        assert!(get_builtin_prompt("nonexistent_prompt", "en").is_none());
    }

    #[test]
    fn test_get_builtin_prompt_unknown_locale_falls_back_to_en() {
        let result = get_builtin_prompt("chunk_summarizer_system", "fr");
        assert!(result.is_some());
        // Should be the English version
        let content = result.unwrap();
        assert!(content.contains("expert meeting summarizer"));
        assert!(!content.contains('\u{060C}'));
    }

    #[test]
    fn test_final_report_contains_named_placeholders() {
        assert!(FINAL_REPORT_SYSTEM_EN.contains("{section_instructions}"));
        assert!(FINAL_REPORT_SYSTEM_EN.contains("{template_markdown}"));
        assert!(FINAL_REPORT_SYSTEM_AR.contains("{section_instructions}"));
        assert!(FINAL_REPORT_SYSTEM_AR.contains("{template_markdown}"));
    }

    #[test]
    fn test_user_prompts_contain_named_placeholders() {
        assert!(CHUNK_SUMMARIZER_USER_EN.contains("{transcript_chunk}"));
        assert!(CHUNK_SUMMARIZER_USER_AR.contains("{transcript_chunk}"));
        assert!(CHUNK_COMBINER_USER_EN.contains("{summaries}"));
        assert!(CHUNK_COMBINER_USER_AR.contains("{summaries}"));
    }
}
