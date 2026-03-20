use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::domain::source::SourceRef;

pub struct CustomJsonAdapter;

impl SourceAdapter for CustomJsonAdapter {
    fn id(&self) -> &'static str {
        "custom-json"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }

    fn normalize(&self, _query: &str) -> Result<SourceRef, AdapterError> {
        Err(AdapterError::UnsupportedQuery)
    }

    fn resolve(&self, _source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        Err(AdapterError::UnsupportedSource)
    }
}
