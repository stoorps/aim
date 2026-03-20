use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::domain::source::SourceRef;

pub struct SourceForgeAdapter;

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

    fn normalize(&self, _query: &str) -> Result<SourceRef, AdapterError> {
        Err(AdapterError::UnsupportedQuery)
    }

    fn resolve(&self, _source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        Err(AdapterError::UnsupportedSource)
    }
}
