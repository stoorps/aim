use crate::app::providers::ProviderRegistry;
use crate::domain::app::AppRecord;
use crate::domain::search::{
    InstalledSearchMatch, SearchInstallStatus, SearchQuery, SearchResult, SearchResults,
    SearchWarning,
};
use crate::source::github::{
    GitHubSearchError, GitHubTransport, TransportRelease, default_transport,
    search_github_repositories_with,
};
use std::collections::HashSet;

pub trait SearchProvider {
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchProviderError>;
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SearchError {
    ProviderFailures(Vec<SearchWarning>),
}

pub fn build_search_results(
    query: &SearchQuery,
    installed_apps: &[AppRecord],
) -> Result<SearchResults, SearchError> {
    build_search_results_with_registered_providers(
        query,
        installed_apps,
        &ProviderRegistry::default(),
    )
}

pub fn build_search_results_with_registered_providers(
    query: &SearchQuery,
    installed_apps: &[AppRecord],
    providers: &ProviderRegistry<'_>,
) -> Result<SearchResults, SearchError> {
    let github_transport = default_transport();
    let github_provider = GitHubSearchProvider::new(github_transport.as_ref());
    let mut resolved_providers = vec![&github_provider as &dyn SearchProvider];
    resolved_providers.extend(providers.search_providers.iter().copied());

    build_search_results_with(query, installed_apps, &resolved_providers)
}

pub fn build_search_results_with(
    query: &SearchQuery,
    installed_apps: &[AppRecord],
    providers: &[&dyn SearchProvider],
) -> Result<SearchResults, SearchError> {
    let installed_matches = collect_installed_matches(query, installed_apps);
    let mut remote_hits = Vec::new();
    let mut warnings = Vec::new();

    for provider in providers {
        match provider.search(query) {
            Ok(mut hits) => remote_hits.append(&mut hits),
            Err(error) => warnings.push(SearchWarning {
                provider_id: Some(error.provider_id),
                message: error.message,
            }),
        }
    }

    annotate_remote_hits_with_install_status(&mut remote_hits, installed_apps);

    if remote_hits.is_empty() && installed_matches.is_empty() && !warnings.is_empty() {
        return Err(SearchError::ProviderFailures(warnings));
    }

    Ok(SearchResults {
        query_text: query.text.clone(),
        remote_hits,
        installed_matches,
        warnings,
    })
}

pub struct GitHubSearchProvider<'a, T: GitHubTransport + ?Sized> {
    transport: &'a T,
}

impl<'a, T: GitHubTransport + ?Sized> GitHubSearchProvider<'a, T> {
    pub fn new(transport: &'a T) -> Self {
        Self { transport }
    }
}

impl<T: GitHubTransport + ?Sized> SearchProvider for GitHubSearchProvider<'_, T> {
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>, SearchProviderError> {
        let name_only_query = format!("{} in:name", query.text);
        let mut ranked_hits =
            search_github_repositories_with(&name_only_query, query.remote_limit, self.transport)
                .map_err(|error| {
                SearchProviderError::new("github", &render_github_search_error(&error))
            })?;

        if ranked_hits.len() < query.remote_limit {
            let mut seen = ranked_hits
                .iter()
                .map(|hit| hit.full_name.clone())
                .collect::<HashSet<_>>();
            let backfill =
                search_github_repositories_with(&query.text, query.remote_limit, self.transport)
                    .map_err(|error| {
                        SearchProviderError::new("github", &render_github_search_error(&error))
                    })?;

            for hit in backfill {
                if ranked_hits.len() >= query.remote_limit {
                    break;
                }

                if seen.insert(hit.full_name.clone()) {
                    ranked_hits.push(hit);
                }
            }
        }

        let normalized_query = normalize_lookup(&query.text);
        let mut ranked_hits = ranked_hits
            .into_iter()
            .enumerate()
            .map(|(index, hit)| {
                (
                    github_remote_match_rank(&normalized_query, &hit),
                    index,
                    hit,
                )
            })
            .collect::<Vec<_>>();

        ranked_hits.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

        Ok(ranked_hits
            .into_iter()
            .filter_map(|(_, _, hit)| {
                let full_name = hit.full_name;
                let release = latest_appimage_release(self.transport, &full_name)?;
                Some(SearchResult {
                    provider_id: "github".to_owned(),
                    display_name: full_name.clone(),
                    description: hit.description,
                    source_locator: hit.html_url,
                    install_query: full_name.clone(),
                    canonical_locator: full_name.clone(),
                    version: Some(release.tag.trim_start_matches('v').to_owned()),
                    install_status: SearchInstallStatus::Available,
                })
            })
            .collect())
    }
}

fn latest_appimage_release<T: GitHubTransport + ?Sized>(
    transport: &T,
    repo: &str,
) -> Option<TransportRelease> {
    transport.fetch_releases(repo).ok().and_then(|releases| {
        releases.into_iter().find(|release| {
            release
                .assets
                .iter()
                .any(|asset| asset.name.ends_with(".AppImage"))
        })
    })
}

fn collect_installed_matches(
    query: &SearchQuery,
    installed_apps: &[AppRecord],
) -> Vec<InstalledSearchMatch> {
    let normalized_query = normalize_lookup(&query.text);
    let mut matches = installed_apps
        .iter()
        .filter_map(|app| {
            match_rank(&normalized_query, &app.stable_id, &app.display_name).map(|rank| {
                (
                    rank,
                    normalize_lookup(&app.stable_id),
                    InstalledSearchMatch {
                        stable_id: app.stable_id.clone(),
                        display_name: app.display_name.clone(),
                        installed_version: app.installed_version.clone(),
                    },
                )
            })
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

    matches
        .into_iter()
        .map(|(_, _, installed_match)| installed_match)
        .collect()
}

fn match_rank(query: &str, stable_id: &str, display_name: &str) -> Option<u8> {
    let stable_id = normalize_lookup(stable_id);
    let display_name = normalize_lookup(display_name);

    [stable_id, display_name]
        .into_iter()
        .filter_map(|candidate| {
            if candidate == query {
                Some(0)
            } else if candidate.starts_with(query) {
                Some(1)
            } else if candidate.contains(query) {
                Some(2)
            } else {
                None
            }
        })
        .min()
}

fn normalize_lookup(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn annotate_remote_hits_with_install_status(
    remote_hits: &mut [SearchResult],
    installed_apps: &[AppRecord],
) {
    for hit in remote_hits.iter_mut() {
        if let Some(installed) = installed_apps
            .iter()
            .find(|app| app_matches_remote_hit(app, hit))
        {
            if installed.installed_version == hit.version {
                hit.install_status = SearchInstallStatus::Installed {
                    installed_version: installed.installed_version.clone(),
                };
            } else {
                hit.install_status = SearchInstallStatus::UpdateAvailable {
                    installed_version: installed.installed_version.clone(),
                    latest_version: hit.version.clone(),
                };
            }
        }
    }
}

fn app_matches_remote_hit(app: &AppRecord, hit: &SearchResult) -> bool {
    let Some(locator) = app_search_locator(app) else {
        return false;
    };

    locator == normalize_lookup(&hit.install_query)
        || locator == normalize_lookup(&hit.canonical_locator)
}

fn app_search_locator(app: &AppRecord) -> Option<String> {
    if let Some(source) = &app.source {
        match source.kind {
            crate::domain::source::SourceKind::GitHub
            | crate::domain::source::SourceKind::AppImageHub => {
                if let Some(locator) = source.canonical_locator.as_deref() {
                    return Some(normalize_lookup(locator));
                }
                return Some(normalize_lookup(&source.locator));
            }
            _ => {}
        }
    }

    app.source_input.as_deref().and_then(|input| {
        if input.contains('/') && !input.contains("://") {
            if let Some((provider, id)) = input.split_once('/')
                && provider.eq_ignore_ascii_case("appimagehub")
                && !id.is_empty()
            {
                return Some(normalize_lookup(id));
            }

            Some(normalize_lookup(input))
        } else {
            None
        }
    })
}

fn github_remote_match_rank(
    query: &str,
    repository: &crate::source::github::TransportRepository,
) -> u8 {
    let full_name = normalize_lookup(&repository.full_name);
    let description = repository.description.as_deref().map(normalize_lookup);
    let mut parts = full_name.split('/');
    let owner = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default();

    if full_name == query {
        return 0;
    }

    if owner == query || repo == query {
        return 1;
    }

    if full_name.starts_with(query) || owner.starts_with(query) || repo.starts_with(query) {
        return 2;
    }

    if full_name.contains(query) || owner.contains(query) || repo.contains(query) {
        return 3;
    }

    if description
        .as_deref()
        .map(|description| description.starts_with(query))
        .unwrap_or(false)
    {
        return 4;
    }

    if description
        .as_deref()
        .map(|description| description.contains(query))
        .unwrap_or(false)
    {
        return 5;
    }

    6
}

fn render_github_search_error(error: &GitHubSearchError) -> String {
    match error {
        GitHubSearchError::Transport(inner) => inner.to_string(),
    }
}
