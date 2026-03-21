use crate::domain::search::SearchResult;

pub trait SearchProvider {
    fn search(
        &self,
        query: &crate::domain::search::SearchQuery,
    ) -> Result<Vec<SearchResult>, SearchProviderError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchProviderError {
    pub provider_id: String,
    pub message: String,
}

impl SearchProviderError {
    pub fn new(provider_id: &str, message: &str) -> Self {
        Self {
            provider_id: provider_id.to_owned(),
            message: message.to_owned(),
        }
    }
}
