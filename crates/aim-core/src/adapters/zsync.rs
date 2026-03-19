use crate::adapters::traits::{AdapterCapabilities, SourceAdapter};

pub struct ZsyncAdapter;

impl SourceAdapter for ZsyncAdapter {
    fn id(&self) -> &'static str {
        "zsync"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }
}
