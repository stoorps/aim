use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{NormalizedSourceKind, ResolvedRelease, SourceKind, SourceRef};

pub struct GitLabAdapter;

impl GitLabAdapter {
    pub fn artifact_name(source: &SourceRef) -> String {
        let slug = canonical_locator(source)
            .split('/')
            .next_back()
            .unwrap_or("app");
        format!("{slug}.AppImage")
    }

    pub fn artifact_url(source: &SourceRef) -> String {
        let repo = canonical_locator(source);
        let artifact_name = Self::artifact_name(source);

        match source.requested_tag.as_deref() {
            Some(tag) => {
                format!("https://gitlab.com/{repo}/-/releases/{tag}/downloads/{artifact_name}")
            }
            None => format!(
                "https://gitlab.com/{repo}/-/releases/permalink/latest/downloads/{artifact_name}"
            ),
        }
    }
}

impl SourceAdapter for GitLabAdapter {
    fn id(&self) -> &'static str {
        "gitlab"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_search: true,
            supports_exact_resolution: true,
        }
    }

    fn repository_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::GitLab)
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
        if source.kind != SourceKind::GitLab {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        if source.kind != SourceKind::GitLab {
            return Err(AdapterError::UnsupportedSource);
        }

        let resolved_source = resolved_source(source)?;

        let version = resolved_source
            .requested_tag
            .clone()
            .unwrap_or_else(|| "latest".to_owned());

        Ok(AdapterResolution {
            source: resolved_source,
            release: ResolvedRelease {
                version,
                prerelease: false,
            },
        })
    }

    fn resolve_supported_source(
        &self,
        source: &SourceRef,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        self.resolve(source).map(AdapterResolveOutcome::Resolved)
    }
}

fn canonical_locator(source: &SourceRef) -> &str {
    source
        .canonical_locator
        .as_deref()
        .unwrap_or(source.locator.as_str())
}

fn resolved_source(source: &SourceRef) -> Result<SourceRef, AdapterError> {
    if source.normalized_kind != NormalizedSourceKind::GitLabCandidate {
        return Ok(source.clone());
    }

    let canonical_locator = gitlab_locator_path(&source.locator).ok_or_else(|| {
        AdapterError::ResolutionFailed(
            "gitlab candidate source could not be reduced to a repository path".to_owned(),
        )
    })?;

    let mut resolved = source.clone();
    resolved.normalized_kind = NormalizedSourceKind::GitLab;
    resolved.canonical_locator = Some(canonical_locator);
    resolved.tracks_latest = resolved.requested_tag.is_none();
    Ok(resolved)
}

fn gitlab_locator_path(locator: &str) -> Option<String> {
    let trimmed = locator
        .trim_start_matches("https://gitlab.com/")
        .trim_start_matches("http://gitlab.com/");

    if trimmed == locator {
        return None;
    }

    let path = trimmed
        .split(['?', '#'])
        .next()
        .unwrap_or(trimmed)
        .trim_matches('/');

    if path.is_empty() {
        None
    } else {
        Some(path.to_owned())
    }
}
