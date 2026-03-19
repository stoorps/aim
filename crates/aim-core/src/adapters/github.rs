use crate::adapters::traits::{AdapterCapabilities, AdapterResolution, SourceAdapter};
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
            source: SourceRef {
                kind: SourceKind::GitHub,
                locator: source.locator.clone(),
            },
            release: ResolvedRelease {
                version: "latest".to_owned(),
            },
        })
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
