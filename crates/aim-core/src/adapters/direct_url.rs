use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::app::query::resolve_query;
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

pub struct DirectUrlAdapter;

impl SourceAdapter for DirectUrlAdapter {
    fn id(&self) -> &'static str {
        "direct-url"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }

    fn exact_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::DirectUrl)
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError> {
        let source = resolve_query(query).map_err(|_| AdapterError::UnsupportedQuery)?;
        if source.kind != SourceKind::DirectUrl {
            return Err(AdapterError::UnsupportedQuery);
        }

        Ok(source)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        if source.kind != SourceKind::DirectUrl {
            return Err(AdapterError::UnsupportedSource);
        }

        Ok(AdapterResolution {
            source: source.clone(),
            release: ResolvedRelease {
                version: "unresolved".to_owned(),
                prerelease: false,
            },
        })
    }
}
