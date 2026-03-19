use crate::domain::source::ResolvedRelease;
use crate::domain::source::SourceRef;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterCapabilities {
    pub supports_search: bool,
    pub supports_exact_resolution: bool,
}

impl AdapterCapabilities {
    pub fn exact_resolution_only() -> Self {
        Self {
            supports_search: false,
            supports_exact_resolution: true,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct AdapterResolution {
    pub source: SourceRef,
    pub release: ResolvedRelease,
}

pub trait SourceAdapter {
    fn id(&self) -> &'static str;

    fn capabilities(&self) -> AdapterCapabilities;
}
