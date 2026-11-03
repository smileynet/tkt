use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result};

pub(super) fn load(
    dir: &Path,
    body: Option<&str>,
    template: Option<&str>,
) -> Result<Option<String>> {
    if let Some(name) = template {
        crate::core::validate::validate_slug(name).map_err(|error| {
            crate::DomainError::new(
                crate::ErrorKind::Validation,
                format!("invalid template name: {error}"),
            )
        })?;
        return std::fs::read_to_string(dir.join("templates").join(format!("{name}.md")))
            .with_context(|| format!("cannot read ticket template {name:?}"))
            .map(Some);
    }
    match body {
        Some("-") => {
            let mut content = String::new();
            std::io::stdin().read_to_string(&mut content)?;
            Ok(Some(content))
        }
        Some(source) if source.starts_with('@') => std::fs::read_to_string(&source[1..])
            .with_context(|| format!("cannot read ticket body file {:?}", &source[1..]))
            .map(Some),
        Some(content) => Ok(Some(content.to_string())),
        None => Ok(None),
    }
}
