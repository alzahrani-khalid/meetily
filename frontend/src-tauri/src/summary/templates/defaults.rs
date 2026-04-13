/// Embedded default templates using compile-time inclusion
///
/// These templates are bundled into the binary and serve as fallbacks
/// when custom templates are not available.
/// All 6 templates are embedded in both English and Arabic (12 total).

// English templates
pub const DAILY_STANDUP_EN: &str = include_str!("../../../templates/daily_standup.json");
pub const STANDARD_MEETING_EN: &str = include_str!("../../../templates/standard_meeting.json");
pub const PROJECT_SYNC_EN: &str = include_str!("../../../templates/project_sync.json");
pub const PSYCHATRIC_SESSION_EN: &str = include_str!("../../../templates/psychatric_session.json");
pub const RETROSPECTIVE_EN: &str = include_str!("../../../templates/retrospective.json");
pub const SALES_MARKETING_EN: &str = include_str!("../../../templates/sales_marketing_client_call.json");

// Arabic templates
pub const DAILY_STANDUP_AR: &str = include_str!("../../../templates/daily_standup.ar.json");
pub const STANDARD_MEETING_AR: &str = include_str!("../../../templates/standard_meeting.ar.json");
pub const PROJECT_SYNC_AR: &str = include_str!("../../../templates/project_sync.ar.json");
pub const PSYCHATRIC_SESSION_AR: &str = include_str!("../../../templates/psychatric_session.ar.json");
pub const RETROSPECTIVE_AR: &str = include_str!("../../../templates/retrospective.ar.json");
pub const SALES_MARKETING_AR: &str = include_str!("../../../templates/sales_marketing_client_call.ar.json");

/// Registry of all built-in templates
///
/// Maps (template_id, locale) to their embedded JSON content.
/// Returns all 12 entries (6 templates x 2 locales).
pub fn get_builtin_templates() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("daily_standup", "en", DAILY_STANDUP_EN),
        ("daily_standup", "ar", DAILY_STANDUP_AR),
        ("standard_meeting", "en", STANDARD_MEETING_EN),
        ("standard_meeting", "ar", STANDARD_MEETING_AR),
        ("project_sync", "en", PROJECT_SYNC_EN),
        ("project_sync", "ar", PROJECT_SYNC_AR),
        ("psychatric_session", "en", PSYCHATRIC_SESSION_EN),
        ("psychatric_session", "ar", PSYCHATRIC_SESSION_AR),
        ("retrospective", "en", RETROSPECTIVE_EN),
        ("retrospective", "ar", RETROSPECTIVE_AR),
        ("sales_marketing_client_call", "en", SALES_MARKETING_EN),
        ("sales_marketing_client_call", "ar", SALES_MARKETING_AR),
    ]
}

/// Get a built-in template by identifier and locale
///
/// Tries the requested locale first, falls back to English for unknown locales.
///
/// # Arguments
/// * `id` - Template identifier (e.g., "daily_standup", "standard_meeting")
/// * `locale` - Locale code (e.g., "en", "ar"). Unknown locales fall back to English.
///
/// # Returns
/// The template JSON content if found, None otherwise
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

/// List all built-in template identifiers (unique, not duplicated per locale)
pub fn list_builtin_template_ids() -> Vec<&'static str> {
    vec![
        "daily_standup",
        "standard_meeting",
        "project_sync",
        "psychatric_session",
        "retrospective",
        "sales_marketing_client_call",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::summary::templates::types::Template;

    #[test]
    fn test_all_12_templates_valid_json() {
        for (id, locale, content) in get_builtin_templates() {
            let result = serde_json::from_str::<serde_json::Value>(content);
            assert!(
                result.is_ok(),
                "Built-in template '{}' locale '{}' contains invalid JSON: {:?}",
                id,
                locale,
                result.err()
            );
        }
    }

    #[test]
    fn test_get_builtin_template_ar() {
        let content = get_builtin_template("daily_standup", "ar");
        assert!(content.is_some());
        let json = content.unwrap();
        assert!(json.contains("الاجتماع اليومي"), "AR template should contain Arabic name");
    }

    #[test]
    fn test_get_builtin_template_en() {
        let content = get_builtin_template("daily_standup", "en");
        assert!(content.is_some());
        let json = content.unwrap();
        assert!(json.contains("Daily Standup"), "EN template should contain English name");
    }

    #[test]
    fn test_get_builtin_template_fallback_to_en() {
        let content = get_builtin_template("daily_standup", "fr");
        assert!(content.is_some());
        let json = content.unwrap();
        assert!(json.contains("Daily Standup"), "Unknown locale should fall back to English");
    }

    #[test]
    fn test_get_builtin_template_nonexistent() {
        assert!(get_builtin_template("nonexistent", "ar").is_none());
        assert!(get_builtin_template("nonexistent", "en").is_none());
    }

    #[test]
    fn test_all_ar_templates_deserialize_and_validate() {
        let ar_ids = list_builtin_template_ids();
        for id in &ar_ids {
            let content = get_builtin_template(id, "ar")
                .unwrap_or_else(|| panic!("AR template '{}' not found", id));
            let template: Template = serde_json::from_str(content)
                .unwrap_or_else(|e| panic!("AR template '{}' failed to deserialize: {}", id, e));
            template.validate()
                .unwrap_or_else(|e| panic!("AR template '{}' failed validation: {}", id, e));
        }
    }

    #[test]
    fn test_all_en_templates_deserialize_and_validate() {
        let en_ids = list_builtin_template_ids();
        for id in &en_ids {
            let content = get_builtin_template(id, "en")
                .unwrap_or_else(|| panic!("EN template '{}' not found", id));
            let template: Template = serde_json::from_str(content)
                .unwrap_or_else(|e| panic!("EN template '{}' failed to deserialize: {}", id, e));
            template.validate()
                .unwrap_or_else(|e| panic!("EN template '{}' failed validation: {}", id, e));
        }
    }

    #[test]
    fn test_list_builtin_template_ids_returns_6() {
        let ids = list_builtin_template_ids();
        assert_eq!(ids.len(), 6, "Should have exactly 6 template IDs, got: {:?}", ids);
    }

    #[test]
    fn test_retrospective_en_content() {
        let content = get_builtin_template("retrospective", "en");
        assert!(content.is_some());
        assert!(content.unwrap().contains("Retrospective"));
    }

    #[test]
    fn test_get_builtin_templates_returns_12() {
        let templates = get_builtin_templates();
        assert_eq!(templates.len(), 12, "Should have 12 entries (6 templates x 2 locales)");
    }
}
