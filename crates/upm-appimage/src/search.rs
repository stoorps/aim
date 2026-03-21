use crate::source::appimagehub::{
    AppImageHubSearchError, AppImageHubTransport, search_appimagehub_with,
};
use upm_core::app::search::{SearchProvider, SearchProviderError};
use upm_core::domain::search::{SearchInstallStatus, SearchQuery, SearchResult};

pub struct AppImageHubSearchProvider<'a, T: AppImageHubTransport + ?Sized> {
    transport: &'a T,
}

impl<'a, T: AppImageHubTransport + ?Sized> AppImageHubSearchProvider<'a, T> {
    pub fn new(transport: &'a T) -> Self {
        Self { transport }
    }
}

impl<T: AppImageHubTransport + ?Sized> SearchProvider for AppImageHubSearchProvider<'_, T> {
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchProviderError> {
        let hits = search_appimagehub_with(&query.text, query.remote_limit, self.transport)
            .map_err(|error| {
                SearchProviderError::new("appimagehub", &render_appimagehub_search_error(&error))
            })?;

        let normalized_query = normalize_lookup(&query.text);
        let mut ranked_hits = hits
            .into_iter()
            .enumerate()
            .map(|(index, hit)| {
                (
                    appimagehub_remote_match_rank(
                        &normalized_query,
                        &hit.name,
                        hit.summary.as_deref(),
                    ),
                    index,
                    hit,
                )
            })
            .collect::<Vec<_>>();

        ranked_hits.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

        Ok(ranked_hits
            .into_iter()
            .map(|(_, _, hit)| SearchResult {
                provider_id: "appimagehub".to_owned(),
                display_name: hit.name,
                description: hit.summary,
                source_locator: hit.detail_page,
                install_query: format!("appimagehub/{}", hit.id),
                canonical_locator: hit.id,
                version: Some(hit.version),
                install_status: SearchInstallStatus::Available,
            })
            .collect())
    }
}

fn normalize_lookup(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn appimagehub_remote_match_rank(query: &str, name: &str, summary: Option<&str>) -> u8 {
    let name = normalize_lookup(name);
    let summary = summary.map(normalize_lookup);

    if name == query {
        return 0;
    }

    if name.starts_with(query) {
        return 1;
    }

    if name.contains(query) {
        return 2;
    }

    if summary
        .as_deref()
        .map(|summary| summary.starts_with(query))
        .unwrap_or(false)
    {
        return 3;
    }

    if summary
        .as_deref()
        .map(|summary| summary.contains(query))
        .unwrap_or(false)
    {
        return 4;
    }

    5
}

fn render_appimagehub_search_error(error: &AppImageHubSearchError) -> String {
    match error {
        AppImageHubSearchError::Parse(inner) => inner.to_string(),
        AppImageHubSearchError::Transport(inner) => inner.to_string(),
    }
}
