use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

pub struct GitLabAdapter;

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

        Ok(AdapterResolution {
            source: source.clone(),
            release: ResolvedRelease {
                version: "latest".to_owned(),
                prerelease: false,
            },
        })
    }
}
