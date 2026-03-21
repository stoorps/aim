use aim_core::app::search::{
    AppImageHubSearchProvider, GitHubSearchProvider, SearchProvider, SearchProviderError,
    build_search_results_with,
};
use aim_core::domain::app::AppRecord;
use aim_core::domain::search::{SearchInstallStatus, SearchQuery, SearchResult};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::source::appimagehub::FixtureAppImageHubTransport;
use aim_core::source::github::FixtureGitHubTransport;

struct StubProvider {
    hit: SearchResult,
}

impl SearchProvider for StubProvider {
    fn search(&self, _query: &SearchQuery) -> Result<Vec<SearchResult>, SearchProviderError> {
        Ok(vec![self.hit.clone()])
    }
}

#[test]
fn appimagehub_search_provider_maps_hits_to_install_ready_results() {
    let provider = AppImageHubSearchProvider::new(&FixtureAppImageHubTransport);

    let results = provider.search(&SearchQuery::new("firefox")).unwrap();

    assert!(results.iter().any(|hit| {
        hit.provider_id == "appimagehub"
            && hit.display_name == "Firefox by Mozilla - Official AppImage Edition"
            && hit.install_query == "appimagehub/2338455"
            && hit.canonical_locator == "2338455"
    }));
}

#[test]
fn appimagehub_hits_are_annotated_as_installed_by_canonical_id() {
    let provider = AppImageHubSearchProvider::new(&FixtureAppImageHubTransport);
    let installed = vec![AppRecord {
        stable_id: "firefox".to_owned(),
        display_name: "Firefox by Mozilla - Official AppImage Edition".to_owned(),
        source_input: Some("appimagehub/2338455".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::AppImageHub,
            locator: "https://www.appimagehub.com/p/2338455".to_owned(),
            input_kind: SourceInputKind::AppImageHubShorthand,
            normalized_kind: NormalizedSourceKind::AppImageHub,
            canonical_locator: Some("2338455".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("latest".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];

    let results =
        build_search_results_with(&SearchQuery::new("firefox"), &installed, &[&provider]).unwrap();

    assert!(results.remote_hits.iter().any(|hit| {
        hit.canonical_locator == "2338455"
            && matches!(
                hit.install_status,
                SearchInstallStatus::Installed {
                    installed_version: Some(ref version)
                } if version == "latest"
            )
    }));
}

#[test]
fn search_can_merge_github_and_appimagehub_providers() {
    let github = GitHubSearchProvider::new(&FixtureGitHubTransport);
    let appimagehub = AppImageHubSearchProvider::new(&FixtureAppImageHubTransport);
    let stub = StubProvider {
        hit: SearchResult {
            provider_id: "github".to_owned(),
            display_name: "firefox-tooling/firestarter".to_owned(),
            description: Some("Stub GitHub result".to_owned()),
            source_locator: "https://github.com/firefox-tooling/firestarter".to_owned(),
            install_query: "firefox-tooling/firestarter".to_owned(),
            canonical_locator: "firefox-tooling/firestarter".to_owned(),
            version: Some("1.0.0".to_owned()),
            install_status: SearchInstallStatus::Available,
        },
    };

    let results = build_search_results_with(
        &SearchQuery::new("firefox"),
        &[],
        &[&stub, &github, &appimagehub],
    )
    .unwrap();

    assert!(
        results
            .remote_hits
            .iter()
            .any(|hit| hit.provider_id == "github")
    );
    assert!(
        results
            .remote_hits
            .iter()
            .any(|hit| hit.provider_id == "appimagehub")
    );
}
