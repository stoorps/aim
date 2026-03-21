use crate::domain::source::{ResolvedRelease, SourceKind, SourceRef};

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdapterResolution {
    pub source: SourceRef,
    pub release: ResolvedRelease,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterResolveOutcome {
    Resolved(AdapterResolution),
    NoInstallableArtifact { source: SourceRef },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    UnsupportedQuery,
    UnsupportedSource,
    ResolutionFailed(String),
}

pub trait SourceAdapter {
    fn id(&self) -> &'static str;

    fn capabilities(&self) -> AdapterCapabilities;

    fn repository_source_kind(&self) -> Option<SourceKind> {
        None
    }

    fn exact_source_kind(&self) -> Option<SourceKind> {
        None
    }

    fn normalize(&self, query: &str) -> Result<SourceRef, AdapterError>;

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError>;

    fn resolve_supported_source(
        &self,
        source: &SourceRef,
    ) -> Result<AdapterResolveOutcome, AdapterError> {
        self.resolve(source).map(AdapterResolveOutcome::Resolved)
    }

    fn supports_source(&self, source: &SourceRef) -> bool {
        self.repository_source_kind() == Some(source.kind)
            || self.exact_source_kind() == Some(source.kind)
    }

    fn resolve_source(&self, source: &SourceRef) -> Result<AdapterResolveOutcome, AdapterError> {
        if !self.supports_source(source) {
            return Err(AdapterError::UnsupportedSource);
        }

        self.resolve_supported_source(source)
    }
}
