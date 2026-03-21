use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{NormalizedSourceKind, ResolvedRelease, SourceKind, SourceRef};

pub struct SourceForgeAdapter;

impl SourceForgeAdapter {
    pub fn artifact_url(source: &SourceRef) -> Option<String> {
        if is_resolved_download_locator(&source.locator) {
            Some(source.locator.clone())
        } else {
            None
        }
    }
}

impl SourceAdapter for SourceForgeAdapter {
    fn id(&self) -> &'static str {
        "sourceforge"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_search: true,
            supports_exact_resolution: true,
        }
    }

    fn repository_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::SourceForge)
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
        if source.kind != SourceKind::SourceForge {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        if source.kind != SourceKind::SourceForge {
            return Err(AdapterError::UnsupportedSource);
        }
        if !is_resolved_download_locator(&source.locator) {
            return Err(AdapterError::ResolutionFailed(
                "sourceforge source has no concrete latest-download artifact".to_owned(),
            ));
        }

        Ok(AdapterResolution {
            source: resolved_source(source),
            release: ResolvedRelease {
                version: "latest".to_owned(),
                prerelease: false,
            },
        })
    }

    fn resolve_supported_source(
        &self,
        source: &SourceRef,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        if Self::artifact_url(source).is_some() {
            return self.resolve(source).map(AdapterResolveOutcome::Resolved);
        }

        if matches!(
            source.normalized_kind,
            NormalizedSourceKind::SourceForge | NormalizedSourceKind::SourceForgeCandidate
        ) {
            return Ok(AdapterResolveOutcome::NoInstallableArtifact {
                source: source.clone(),
            });
        }

        Ok(AdapterResolveOutcome::NoInstallableArtifact {
            source: source.clone(),
        })
    }
}

fn resolved_source(source: &SourceRef) -> SourceRef {
    let mut resolved = source.clone();
    if is_sourceforge_stable_download_locator(&resolved.locator) {
        resolved.normalized_kind = NormalizedSourceKind::SourceForge;
        resolved.tracks_latest = true;
    }

    resolved
}

fn is_resolved_download_locator(locator: &str) -> bool {
    is_latest_download_locator(locator) || is_sourceforge_stable_download_locator(locator)
}

fn is_latest_download_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');
    trimmed.ends_with("/files/latest/download")
}

fn is_sourceforge_stable_download_locator(locator: &str) -> bool {
    let trimmed = locator
        .split(['?', '#'])
        .next()
        .unwrap_or(locator)
        .trim_end_matches('/');
    trimmed.ends_with("/files/releases/stable/download")
}
