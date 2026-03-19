use crate::adapters::traits::{AdapterCapabilities, AdapterResolution, SourceAdapter};
use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

pub struct DirectUrlAdapter;

impl DirectUrlAdapter {
    pub fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, DirectUrlAdapterError> {
        if source.kind != SourceKind::DirectUrl {
            return Err(DirectUrlAdapterError::UnsupportedSource);
        }

        Ok(AdapterResolution {
            source: SourceRef {
                kind: SourceKind::DirectUrl,
                locator: source.locator.clone(),
            },
            release: ResolvedRelease {
                version: "unresolved".to_owned(),
            },
        })
    }
}

impl SourceAdapter for DirectUrlAdapter {
    fn id(&self) -> &'static str {
        "direct-url"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DirectUrlAdapterError {
    UnsupportedSource,
}
