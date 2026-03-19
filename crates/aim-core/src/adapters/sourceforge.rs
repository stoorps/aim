use crate::adapters::traits::{AdapterCapabilities, SourceAdapter};

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
}
