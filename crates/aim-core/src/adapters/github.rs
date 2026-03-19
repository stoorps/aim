use crate::adapters::traits::{AdapterCapabilities, AdapterResolution, SourceAdapter};
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

    pub fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, GitHubAdapterError> {
        if source.kind != SourceKind::GitHub {
            return Err(GitHubAdapterError::UnsupportedSource);
        }

        Ok(AdapterResolution {
            source: source.clone(),
            release: ResolvedRelease {
                version: "latest".to_owned(),
                prerelease: false,
            },
        })
    }

    pub fn normalize(&self, query: &str) -> Result<SourceRef, GitHubAdapterError> {
        resolve_query(query).map_err(|_| GitHubAdapterError::UnsupportedSource)
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
}

#[derive(Debug, Eq, PartialEq)]
pub enum GitHubAdapterError {
    UnsupportedSource,
}
