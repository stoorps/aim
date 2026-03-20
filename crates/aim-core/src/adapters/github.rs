use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

pub struct GitHubAdapter;

impl Default for GitHubAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl SourceAdapter for GitHubAdapter {
    fn id(&self) -> &'static str {
        "github"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities {
            supports_search: true,
            supports_exact_resolution: true,
        }
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
        if source.kind != SourceKind::GitHub {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        if source.kind != SourceKind::GitHub {
            return Err(AdapterError::UnsupportedSource);
        }

        Ok(AdapterResolution {
            source: source.clone(),
            release: ResolvedRelease {
                version: "latest".to_owned(),
                prerelease: false,
            },
        })
    }
}
