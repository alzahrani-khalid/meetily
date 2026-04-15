use super::defaults;
use super::types::Template;
use std::path::PathBuf;
use tracing::{debug, info, warn};
use once_cell::sync::Lazy;
use std::sync::RwLock;

// Global storage for the bundled templates directory path
static BUNDLED_TEMPLATES_DIR: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

/// Set the bundled templates directory path (called once at app startup)
pub fn set_bundled_templates_dir(path: PathBuf) {
    info!("Bundled templates directory set to: {:?}", path);
    if let Ok(mut dir) = BUNDLED_TEMPLATES_DIR.write() {
        *dir = Some(path);
    }
}

/// Get the user's custom templates directory path
///
/// Returns the platform-specific application data directory for custom templates:
/// - macOS: ~/Library/Application Support/Meetily/templates/
/// - Windows: %APPDATA%\Meetily\templates\
/// - Linux: ~/.config/Meetily/templates/
fn get_custom_templates_dir() -> Option<PathBuf> {
    let mut path = dirs::data_dir()?;
    path.push("Meetily");
    path.push("templates");
    Some(path)
}

/// Load a template from the bundled resources directory
///
/// Tries locale-specific file first (`{id}.{locale}.json`), then base file (`{id}.json`).
///
/// # Arguments
/// * `template_id` - Template identifier (without .json extension)
/// * `locale` - Locale code (e.g., "en", "ar")
///
/// # Returns
/// The template JSON content if found, None otherwise
fn load_bundled_template(template_id: &str, locale: &str) -> Option<String> {
    let bundled_dir = BUNDLED_TEMPLATES_DIR.read().ok()?.clone()?;

    // Try locale-specific file first
    let locale_path = bundled_dir.join(format!("{}.{}.json", template_id, locale));
    debug!("Checking for bundled template at: {:?}", locale_path);
    if let Ok(content) = std::fs::read_to_string(&locale_path) {
        info!("Loaded bundled template '{}' locale '{}' from {:?}", template_id, locale, locale_path);
        return Some(content);
    }

    // Fall back to base file
    let base_path = bundled_dir.join(format!("{}.json", template_id));
    debug!("Checking for bundled template at: {:?}", base_path);
    match std::fs::read_to_string(&base_path) {
        Ok(content) => {
            info!("Loaded bundled template '{}' (base) from {:?}", template_id, base_path);
            Some(content)
        }
        Err(e) => {
            debug!("No bundled template '{}' found: {}", template_id, e);
            None
        }
    }
}

/// Load a template from the user's custom templates directory
///
/// Tries locale-specific file first (`{id}.{locale}.json`), then base file (`{id}.json`).
///
/// # Arguments
/// * `template_id` - Template identifier (without .json extension)
/// * `locale` - Locale code (e.g., "en", "ar")
///
/// # Returns
/// The template JSON content if found, None otherwise
fn load_custom_template(template_id: &str, locale: &str) -> Option<String> {
    let custom_dir = get_custom_templates_dir()?;

    // Try locale-specific file first
    let locale_path = custom_dir.join(format!("{}.{}.json", template_id, locale));
    debug!("Checking for custom template at: {:?}", locale_path);
    if let Ok(content) = std::fs::read_to_string(&locale_path) {
        info!("Loaded custom template '{}' locale '{}' from {:?}", template_id, locale, locale_path);
        return Some(content);
    }

    // Fall back to base file
    let base_path = custom_dir.join(format!("{}.json", template_id));
    debug!("Checking for custom template at: {:?}", base_path);
    match std::fs::read_to_string(&base_path) {
        Ok(content) => {
            info!("Loaded custom template '{}' (base) from {:?}", template_id, base_path);
            Some(content)
        }
        Err(e) => {
            debug!("No custom template '{}' found: {}", template_id, e);
            None
        }
    }
}

/// Load and parse a template by identifier and locale
///
/// This function implements a locale-aware fallback strategy:
/// At each tier, tries `{id}.{locale}.json` before `{id}.json`:
/// 1. Check user's custom templates directory (locale-specific, then base)
/// 2. Check bundled resources directory (locale-specific, then base)
/// 3. Fall back to built-in embedded templates (locale-aware with English fallback)
/// 4. Return error if not found in any location
///
/// # Arguments
/// * `template_id` - Template identifier (e.g., "daily_standup", "standard_meeting")
/// * `locale` - Locale code (e.g., "en", "ar"). Unknown locales fall back to English.
///
/// # Returns
/// Parsed and validated Template struct
pub fn get_template(template_id: &str, locale: &str) -> Result<Template, String> {
    info!("Loading template: {} (locale: {})", template_id, locale);

    // Try custom template first, then bundled, then built-in
    let json_content = if let Some(custom_content) = load_custom_template(template_id, locale) {
        debug!("Using custom template for '{}' locale '{}'", template_id, locale);
        custom_content
    } else if let Some(bundled_content) = load_bundled_template(template_id, locale) {
        debug!("Using bundled template for '{}' locale '{}'", template_id, locale);
        bundled_content
    } else if let Some(builtin_content) = defaults::get_builtin_template(template_id, locale) {
        debug!("Using built-in template for '{}' locale '{}'", template_id, locale);
        builtin_content.to_string()
    } else {
        return Err(format!(
            "Template '{}' not found. Available templates: {}",
            template_id,
            list_template_ids().join(", ")
        ));
    };

    // Parse and validate
    validate_and_parse_template(&json_content)
}

/// Validate and parse template JSON
///
/// # Arguments
/// * `json_content` - Raw JSON string
///
/// # Returns
/// Parsed and validated Template struct
pub fn validate_and_parse_template(json_content: &str) -> Result<Template, String> {
    let template: Template = serde_json::from_str(json_content)
        .map_err(|e| format!("Failed to parse template JSON: {}", e))?;

    template.validate()?;

    Ok(template)
}

/// Extract the base template ID from a filename, stripping locale suffixes
///
/// Examples:
/// - "daily_standup.json" -> "daily_standup"
/// - "daily_standup.ar.json" -> "daily_standup"
/// - "project_sync.en.json" -> "project_sync"
fn extract_base_template_id(filename: &str) -> Option<String> {
    let name = filename.strip_suffix(".json")?;
    // Check if there's a locale suffix (e.g., ".ar", ".en")
    if let Some(base) = name.strip_suffix(".ar")
        .or_else(|| name.strip_suffix(".en"))
    {
        Some(base.to_string())
    } else {
        Some(name.to_string())
    }
}

/// List all available template identifiers
///
/// Returns a combined list of unique base template IDs (no locale duplicates):
/// - Built-in template IDs
/// - Bundled template IDs (from app resources)
/// - Custom template IDs (from user's data directory)
pub fn list_template_ids() -> Vec<String> {
    let mut ids: Vec<String> = defaults::list_builtin_template_ids()
        .into_iter()
        .map(|s| s.to_string())
        .collect();

    // Add bundled templates if directory is set
    if let Ok(bundled_dir_lock) = BUNDLED_TEMPLATES_DIR.read() {
        if let Some(bundled_dir) = bundled_dir_lock.as_ref() {
            if bundled_dir.exists() {
                match std::fs::read_dir(bundled_dir) {
                    Ok(entries) => {
                        for entry in entries.flatten() {
                            if let Some(filename) = entry.file_name().to_str() {
                                if filename.ends_with(".json") {
                                    if let Some(id) = extract_base_template_id(filename) {
                                        if !ids.contains(&id) {
                                            ids.push(id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read bundled templates directory: {}", e);
                    }
                }
            }
        }
    }

    // Add custom templates if directory exists
    if let Some(custom_dir) = get_custom_templates_dir() {
        if custom_dir.exists() {
            match std::fs::read_dir(&custom_dir) {
                Ok(entries) => {
                    for entry in entries.flatten() {
                        if let Some(filename) = entry.file_name().to_str() {
                            if filename.ends_with(".json") {
                                if let Some(id) = extract_base_template_id(filename) {
                                    if !ids.contains(&id) {
                                        ids.push(id);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read custom templates directory: {}", e);
                }
            }
        }
    }

    ids.sort();
    ids
}

/// List all available templates with their metadata for a given locale
///
/// Returns a list of (id, name, description) tuples
pub fn list_templates(locale: &str) -> Vec<(String, String, String)> {
    let mut templates = Vec::new();

    for id in list_template_ids() {
        match get_template(&id, locale) {
            Ok(template) => {
                templates.push((id, template.name, template.description));
            }
            Err(e) => {
                warn!("Failed to load template '{}': {}", id, e);
            }
        }
    }

    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_builtin_template() {
        let template = get_template("daily_standup", "en");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.name, "Daily Standup");
        assert!(!template.sections.is_empty());
    }

    #[test]
    fn test_get_builtin_template_arabic() {
        let template = get_template("daily_standup", "ar");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.name, "الاجتماع اليومي");
        assert!(!template.sections.is_empty());
    }

    #[test]
    fn test_get_nonexistent_template() {
        let result = get_template("nonexistent_template", "en");
        assert!(result.is_err());
    }

    #[test]
    fn test_locale_fallback() {
        let template = get_template("daily_standup", "fr");
        assert!(template.is_ok());

        let template = template.unwrap();
        assert_eq!(template.name, "Daily Standup", "Unknown locale should fall back to English");
    }

    #[test]
    fn test_list_template_ids() {
        let ids = list_template_ids();
        assert_eq!(ids.len(), 6, "Should have exactly 6 template IDs, got: {:?}", ids);
        assert!(ids.contains(&"daily_standup".to_string()));
        assert!(ids.contains(&"standard_meeting".to_string()));
        assert!(ids.contains(&"project_sync".to_string()));
        assert!(ids.contains(&"psychatric_session".to_string()));
        assert!(ids.contains(&"retrospective".to_string()));
        assert!(ids.contains(&"sales_marketing_client_call".to_string()));
    }

    #[test]
    fn test_validate_invalid_json() {
        let result = validate_and_parse_template("invalid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_base_template_id() {
        assert_eq!(extract_base_template_id("daily_standup.json"), Some("daily_standup".to_string()));
        assert_eq!(extract_base_template_id("daily_standup.ar.json"), Some("daily_standup".to_string()));
        assert_eq!(extract_base_template_id("daily_standup.en.json"), Some("daily_standup".to_string()));
        assert_eq!(extract_base_template_id("not_json"), None);
    }

    // =================================================================
    // QA-03: 3-case fallback matrix for get_template()
    // =================================================================

    #[test]
    fn qa03_ar_locale_returns_ar_template() {
        // Case 1: AR file present -> AR returned
        let result = get_template("daily_standup", "ar");
        assert!(result.is_ok(), "AR template must resolve");
        let template = result.unwrap();
        // Verify Arabic content by checking template name contains Arabic Unicode range
        let has_arabic = template.name.chars().any(|c| c >= '\u{0600}' && c <= '\u{06FF}');
        assert!(has_arabic, "AR template name must contain Arabic characters");
    }

    #[test]
    fn qa03_unknown_locale_falls_back_to_en() {
        // Case 2: Unknown locale (no "fr" file) + EN present -> EN fallback
        let result = get_template("daily_standup", "fr");
        assert!(result.is_ok(), "fallback to EN must succeed");
        let template = result.unwrap();
        // EN template name should be ASCII (no Arabic chars)
        let has_arabic = template.name.chars().any(|c| c >= '\u{0600}' && c <= '\u{06FF}');
        assert!(!has_arabic, "fallback must return EN template, not AR");
    }

    #[test]
    fn qa03_nonexistent_id_returns_error() {
        // Case 3: Both missing -> error
        let result = get_template("qa03_nonexistent_template", "ar");
        assert!(result.is_err(), "nonexistent template must error");
    }
}
