use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::domain::source::SourceRef;

#[derive(Debug)]
pub struct MockAdapter {
    id: &'static str,
    capabilities: AdapterCapabilities,
}

impl MockAdapter {
    pub fn exact_resolution_only() -> Self {
        Self {
            id: "mock",
            capabilities: AdapterCapabilities::exact_resolution_only(),
        }
    }
}

impl SourceAdapter for MockAdapter {
    fn id(&self) -> &'static str {
        self.id
    }

    fn capabilities(&self) -> AdapterCapabilities {
        self.capabilities
    }

    fn normalize(&self, _query: &str) -> Result<SourceRef, AdapterError> {
        Err(AdapterError::UnsupportedQuery)
    }

    fn resolve(&self, _source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        Err(AdapterError::UnsupportedSource)
    }
}
