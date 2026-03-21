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
    pub search_providers: Vec<Box<dyn SearchProvider + 'a>>,
    pub external_add_providers: Vec<Box<dyn ExternalAddProvider + 'a>>,
}

impl<'a> ProviderRegistry<'a> {
    pub fn with_search_provider<P>(mut self, provider: P) -> Self
    where
        P: SearchProvider + 'a,
    {
        self.search_providers.push(Box::new(provider));
        self
    }

    pub fn with_external_add_provider<P>(mut self, provider: P) -> Self
    where
        P: ExternalAddProvider + 'a,
    {
        self.external_add_providers.push(Box::new(provider));
        self
    }
}
