use super::defaults;

/// Get a prompt by ID and locale with English fallback.
///
/// Resolution: try exact locale match first, fall back to "en".
/// Returns Err if prompt ID is unknown.
///
/// # Arguments
/// * `prompt_id` - Prompt identifier (e.g., "chunk_summarizer_system")
/// * `locale` - Locale code (e.g., "en", "ar")
///
/// # Returns
/// The prompt content string, or an error if the prompt ID is not recognized
pub fn get_prompt(prompt_id: &str, locale: &str) -> Result<&'static str, String> {
    defaults::get_builtin_prompt(prompt_id, locale)
        .or_else(|| defaults::get_builtin_prompt(prompt_id, "en"))
        .ok_or_else(|| format!("Prompt '{}' not found for locale '{}'", prompt_id, locale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_prompt_en() {
        let result = get_prompt("chunk_summarizer_system", "en");
        assert!(result.is_ok());
        assert!(result.unwrap().contains("expert meeting summarizer"));
    }

    #[test]
    fn test_get_prompt_ar() {
        let result = get_prompt("chunk_summarizer_system", "ar");
        assert!(result.is_ok());
        assert!(result.unwrap().contains('\u{060C}'));
    }

    #[test]
    fn test_get_prompt_fallback_to_en() {
        let result = get_prompt("chunk_summarizer_system", "fr");
        assert!(result.is_ok());
        let content = result.unwrap();
        // Should fall back to English
        assert!(content.contains("expert meeting summarizer"));
        assert!(!content.contains('\u{060C}'));
    }

    #[test]
    fn test_get_prompt_unknown_id() {
        let result = get_prompt("nonexistent_prompt", "en");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_get_prompt_final_report_has_placeholders() {
        let result = get_prompt("final_report_system", "en");
        assert!(result.is_ok());
        let content = result.unwrap();
        assert!(content.contains("{section_instructions}"));
        assert!(content.contains("{template_markdown}"));
    }

    #[test]
    fn test_get_prompt_all_ids_resolve() {
        let ids = [
            "chunk_summarizer_system",
            "chunk_summarizer_user",
            "chunk_combiner_system",
            "chunk_combiner_user",
            "final_report_system",
        ];
        for id in ids {
            assert!(get_prompt(id, "en").is_ok(), "EN prompt '{}' should resolve", id);
            assert!(get_prompt(id, "ar").is_ok(), "AR prompt '{}' should resolve", id);
        }
    }
}
