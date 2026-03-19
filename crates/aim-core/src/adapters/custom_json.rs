use crate::adapters::traits::{AdapterCapabilities, SourceAdapter};

pub struct CustomJsonAdapter;

impl SourceAdapter for CustomJsonAdapter {
    fn id(&self) -> &'static str {
        "custom-json"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }
}
