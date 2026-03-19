use crate::domain::app::{AppIdentity, IdentityConfidence};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityFallback {
    DisallowRawUrl,
    AllowRawUrl,
}

pub fn resolve_identity(
    explicit_name: Option<&str>,
    explicit_id: Option<&str>,
    source_url: Option<&str>,
    fallback: IdentityFallback,
) -> Result<AppIdentity, ResolveIdentityError> {
    if let Some(explicit_id) = explicit_id.filter(|value| !value.trim().is_empty()) {
        let stable_id = normalize_identifier(explicit_id);
        let display_name = explicit_name
            .filter(|value| !value.trim().is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| explicit_id.to_owned());

        return Ok(AppIdentity {
            stable_id,
            display_name,
            confidence: IdentityConfidence::Confident,
        });
    }

    if let Some(explicit_name) = explicit_name.filter(|value| !value.trim().is_empty()) {
        return Ok(AppIdentity {
            stable_id: normalize_identifier(explicit_name),
            display_name: explicit_name.to_owned(),
            confidence: IdentityConfidence::NeedsConfirmation,
        });
    }

    if let Some(source_url) = source_url.filter(|value| !value.trim().is_empty())
        && fallback == IdentityFallback::AllowRawUrl
    {
        return Ok(AppIdentity {
            stable_id: normalize_url_identifier(source_url),
            display_name: source_url.to_owned(),
            confidence: IdentityConfidence::RawUrlFallback,
        });
    }

    Err(ResolveIdentityError::Unresolved)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveIdentityError {
    Unresolved,
}

fn normalize_identifier(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| match ch {
            'A'..='Z' => ch.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '.' | '-' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn normalize_url_identifier(url: &str) -> String {
    let trimmed = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("file://");

    format!("url-{}", normalize_identifier(trimmed))
}
