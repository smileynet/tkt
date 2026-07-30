//! Input validation for user-provided values entering frontmatter, filenames, and commit messages.

/// Windows reserved device names (case-insensitive, with or without extensions).
const RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Validate a ticket slug for use in filenames.
/// Rules: lowercase alphanumeric + dashes, starts with alphanumeric, max 100 chars,
/// not a Windows reserved device name.
pub fn validate_slug(slug: &str) -> Result<(), String> {
    if slug.is_empty() {
        return Err("slug cannot be empty".to_string());
    }
    if slug.len() > 100 {
        return Err(format!("slug too long ({} chars, max 100)", slug.len()));
    }
    // Must match pattern
    if !slug.chars().next().unwrap().is_ascii_alphanumeric()
        || !slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(
            "slug must be lowercase letters, digits, and dashes, starting with alphanumeric"
                .to_string(),
        );
    }
    // Check Windows reserved names (case-insensitive, strip any extension)
    let base = slug.split('.').next().unwrap_or(slug);
    if RESERVED_NAMES.contains(&base.to_ascii_lowercase().as_str()) {
        return Err(format!("'{}' is a Windows reserved device name", slug));
    }
    Ok(())
}

/// Validate free-text fields (titles, specs, notes) for YAML/commit safety.
/// Rules: no literal newlines, no carriage returns, no null bytes. Max length enforced.
pub fn validate_free_text(value: &str, field_name: &str, max_len: usize) -> Result<(), String> {
    if value.len() > max_len {
        return Err(format!(
            "{} too long ({} chars, max {})",
            field_name,
            value.len(),
            max_len
        ));
    }
    if value.contains('\n') {
        return Err(format!("{} must not contain newlines", field_name));
    }
    if value.contains('\r') {
        return Err(format!("{} must not contain carriage returns", field_name));
    }
    if value.contains('\0') {
        return Err(format!("{} must not contain null bytes", field_name));
    }
    Ok(())
}

/// Validate a ticket ID reference (for --blocked-by values).
/// Must be all digits.
pub fn validate_id(id: &str) -> Result<(), String> {
    if id.is_empty() {
        return Err("id cannot be empty".to_string());
    }
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return Err(format!("id {:?} must be all digits", id));
    }
    Ok(())
}

/// Validate that blocked_by doesn't contain self-references.
pub fn validate_no_self_dep(own_id: &str, blocked_by: &[&str]) -> Result<(), String> {
    if blocked_by.contains(&own_id) {
        return Err(format!("ticket cannot block itself (id {})", own_id));
    }
    Ok(())
}

/// Validate env value.
pub fn validate_env(env: &str) -> Result<(), String> {
    const VALID: &[&str] = &["corp", "personal", "either"];
    if !VALID.contains(&env) {
        return Err(format!(
            "env must be one of {} (got {:?})",
            VALID.join("/"),
            env
        ));
    }
    Ok(())
}

/// Validate priority value.
pub fn validate_priority(priority: &str) -> Result<(), String> {
    if priority != "high" {
        return Err(format!("priority must be 'high' (got {:?})", priority));
    }
    Ok(())
}

/// Check for duplicate slugs in a batch.
pub fn validate_no_duplicate_slugs(slugs: &[&str]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    for slug in slugs {
        if !seen.insert(*slug) {
            return Err(format!("duplicate slug {:?} in batch", slug));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_valid() {
        assert!(validate_slug("auth-system").is_ok());
        assert!(validate_slug("a1").is_ok());
        assert!(validate_slug("fix-bug-123").is_ok());
    }

    #[test]
    fn slug_rejects_reserved_names() {
        assert!(validate_slug("con").is_err());
        assert!(validate_slug("prn").is_err());
        assert!(validate_slug("aux").is_err());
        assert!(validate_slug("nul").is_err());
        assert!(validate_slug("com1").is_err());
        assert!(validate_slug("lpt9").is_err());
    }

    #[test]
    fn slug_rejects_invalid_chars() {
        assert!(validate_slug("Has-Caps").is_err());
        assert!(validate_slug("-starts-dash").is_err());
        assert!(validate_slug("has space").is_err());
        assert!(validate_slug("has_underscore").is_err());
        assert!(validate_slug("").is_err());
    }

    #[test]
    fn slug_rejects_too_long() {
        let long = "a".repeat(101);
        assert!(validate_slug(&long).is_err());
    }

    #[test]
    fn free_text_valid() {
        assert!(validate_free_text("Hello world", "title", 200).is_ok());
        assert!(validate_free_text("Has \"quotes\" and \\backslash", "title", 200).is_ok());
    }

    #[test]
    fn free_text_rejects_newlines() {
        assert!(validate_free_text("line\nbreak", "title", 200).is_err());
        assert!(validate_free_text("cr\rhere", "title", 200).is_err());
        assert!(validate_free_text("null\0byte", "title", 200).is_err());
    }

    #[test]
    fn free_text_rejects_too_long() {
        let long = "a".repeat(201);
        assert!(validate_free_text(&long, "title", 200).is_err());
    }

    #[test]
    fn id_validation() {
        assert!(validate_id("01").is_ok());
        assert!(validate_id("123").is_ok());
        assert!(validate_id("abc").is_err());
        assert!(validate_id("01a").is_err());
        assert!(validate_id("").is_err());
    }

    #[test]
    fn self_dep_detection() {
        assert!(validate_no_self_dep("05", &["01", "03"]).is_ok());
        assert!(validate_no_self_dep("05", &["01", "05"]).is_err());
    }

    #[test]
    fn duplicate_slugs() {
        assert!(validate_no_duplicate_slugs(&["foo", "bar", "baz"]).is_ok());
        assert!(validate_no_duplicate_slugs(&["foo", "bar", "foo"]).is_err());
    }
}
