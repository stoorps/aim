use crate::adapters::traits::AdapterCapabilities;
use crate::adapters::traits::SourceAdapter;

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
}
