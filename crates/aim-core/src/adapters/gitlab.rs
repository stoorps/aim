use crate::adapters::traits::{AdapterCapabilities, AdapterResolution, SourceAdapter};
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

pub struct GitLabAdapter;

impl GitLabAdapter {
    pub fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, GitLabAdapterError> {
        if source.kind != SourceKind::GitLab {
            return Err(GitLabAdapterError::UnsupportedSource);
        }

        Ok(AdapterResolution {
            source: SourceRef {
                kind: SourceKind::GitLab,
                locator: source.locator.clone(),
            },
            release: ResolvedRelease {
                version: "latest".to_owned(),
            },
        })
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
}

#[derive(Debug, Eq, PartialEq)]
pub enum GitLabAdapterError {
    UnsupportedSource,
}
