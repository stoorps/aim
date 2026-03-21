use crate::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, SourceAdapter,
};
use crate::domain::source::SourceRef;

pub struct ZsyncAdapter;

impl SourceAdapter for ZsyncAdapter {
    fn id(&self) -> &'static str {
        "zsync"
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
