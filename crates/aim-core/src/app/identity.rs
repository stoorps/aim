use crate::domain::app::{AppIdentity, IdentityConfidence};
use crate::source::input::classify_input;

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
        let stable_id = normalize_identifier(explicit_id)?;
        let display_name = explicit_name
            .filter(|value| !value.trim().is_empty())
            .map(sanitize_display_name)
            .unwrap_or_else(|| sanitize_display_name(explicit_id));

        return Ok(AppIdentity {
            stable_id,
            display_name,
            confidence: IdentityConfidence::Confident,
        });
    }

    if let Some(explicit_name) = explicit_name.filter(|value| !value.trim().is_empty()) {
        return Ok(AppIdentity {
            stable_id: normalize_identifier(explicit_name)?,
            display_name: sanitize_display_name(explicit_name),
            confidence: IdentityConfidence::NeedsConfirmation,
        });
    }

    if let Some(source_url) = source_url.filter(|value| !value.trim().is_empty())
        && let Ok(classified) = classify_input(source_url)
        && let Some(repo) = classified.canonical_locator
    {
        let display_name = repo.split('/').next_back().unwrap_or(&repo).to_owned();
        return Ok(AppIdentity {
            stable_id: normalize_identifier(&repo)?,
            display_name: sanitize_display_name(&display_name),
            confidence: IdentityConfidence::Confident,
        });
    }

    if let Some(source_url) = source_url.filter(|value| !value.trim().is_empty())
        && fallback == IdentityFallback::AllowRawUrl
    {
        return Ok(AppIdentity {
            stable_id: normalize_url_identifier(source_url)?,
            display_name: sanitize_display_name(source_url),
            confidence: IdentityConfidence::RawUrlFallback,
        });
    }

    Err(ResolveIdentityError::Unresolved)
}

#[derive(Debug, Eq, PartialEq)]
pub enum ResolveIdentityError {
    Unresolved,
    InvalidStableId,
}

fn normalize_identifier(value: &str) -> Result<String, ResolveIdentityError> {
    let normalized = value
        .trim()
        .chars()
        .map(|ch| match ch {
            'A'..='Z' => ch.to_ascii_lowercase(),
            'a'..='z' | '0'..='9' | '.' | '-' => ch,
            _ => '-',
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();

    if normalized.is_empty() || normalized.contains("..") {
        return Err(ResolveIdentityError::InvalidStableId);
    }

    Ok(normalized)
}

fn normalize_url_identifier(url: &str) -> Result<String, ResolveIdentityError> {
    let trimmed = url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("file://");

    Ok(format!("url-{}", normalize_identifier(trimmed)?))
}

fn sanitize_display_name(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if matches!(ch, '\n' | '\r') || ch.is_control() {
                ' '
            } else {
                ch
            }
        })
        .collect::<String>();
    let sanitized = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");

    if sanitized.is_empty() {
        "app".to_owned()
    } else {
        sanitized
    }
}
