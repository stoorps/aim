use crate::adapters::traits::{AdapterError, AdapterResolution};
use crate::app::search::SearchProvider;
use crate::domain::source::SourceRef;
use crate::domain::update::{ArtifactCandidate, UpdateStrategy};

pub trait ExternalAddProvider {
    fn id(&self) -> &'static str;

    fn resolve(&self, source: &SourceRef) -> Result<Option<ExternalAddResolution>, AdapterError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalAddResolution {
    pub resolution: AdapterResolution,
    pub selected_artifact: ArtifactCandidate,
    pub update_strategy: UpdateStrategy,
    pub display_name_hint: Option<String>,
}

#[derive(Default)]
pub struct ProviderRegistry<'a> {
    pub search_providers: Vec<&'a dyn SearchProvider>,
    pub external_add_providers: Vec<&'a dyn ExternalAddProvider>,
}
