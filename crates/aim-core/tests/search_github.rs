use aim_core::app::search::{
    GitHubSearchProvider, SearchProvider, SearchProviderError, build_search_results_with,
};
use aim_core::domain::app::AppRecord;
use aim_core::domain::search::{SearchInstallStatus, SearchQuery};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::source::github::{FixtureGitHubTransport, search_github_repositories_with};

#[test]
fn github_fixtures_return_normalized_remote_hits() {
    let query = SearchQuery::new("bat");
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &[], &[&provider]).unwrap();

    assert_eq!(query.remote_limit, 10);
    assert!(results.installed_matches.is_empty());
    assert!(results.warnings.is_empty());
    assert_eq!(results.remote_hits.len(), 3);

    let first = &results.remote_hits[0];
    assert_eq!(first.provider_id, "github");
    assert_eq!(first.display_name, "sharkdp/bat");
    assert_eq!(
        first.description.as_deref(),
        Some("A cat(1) clone with wings.")
    );
    assert_eq!(first.source_locator, "https://github.com/sharkdp/bat");
    assert_eq!(first.install_query, "sharkdp/bat");
    assert_eq!(first.canonical_locator, "sharkdp/bat");
    assert_eq!(first.version.as_deref(), Some("1.0.0"));
    assert_eq!(first.install_status, SearchInstallStatus::Available);
}

#[test]
fn github_search_respects_limit_and_fixture_order() {
    let query = SearchQuery::with_remote_limit("bat", 2);
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &[], &[&provider]).unwrap();

    let locators = results
        .remote_hits
        .iter()
        .map(|hit| hit.canonical_locator.as_str())
        .collect::<Vec<_>>();

    assert_eq!(locators, vec!["sharkdp/bat", "astatine/bat"]);
}

#[test]
fn github_search_ranks_full_name_matches_above_description_only_matches() {
    let query = SearchQuery::new("pingdotgg");
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &[], &[&provider]).unwrap();

    let locators = results
        .remote_hits
        .iter()
        .map(|hit| hit.canonical_locator.as_str())
        .collect::<Vec<_>>();

    assert_eq!(locators[0], "pingdotgg/t3code");
    assert_eq!(locators, vec!["pingdotgg/t3code"]);
}

#[test]
fn github_search_backfills_description_matches_after_name_matches() {
    let query = SearchQuery::with_remote_limit("pingdotgg", 3);
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &[], &[&provider]).unwrap();

    let locators = results
        .remote_hits
        .iter()
        .map(|hit| hit.canonical_locator.as_str())
        .collect::<Vec<_>>();

    assert_eq!(locators, vec!["pingdotgg/t3code"]);
}

#[test]
fn github_search_only_returns_repositories_with_appimage_release_assets() {
    let query = SearchQuery::new("pingdotgg");
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &[], &[&provider]).unwrap();

    assert!(
        results
            .remote_hits
            .iter()
            .all(|hit| hit.canonical_locator == "pingdotgg/t3code")
    );
}

#[test]
fn github_name_only_search_excludes_description_only_matches() {
    let hits =
        search_github_repositories_with("pingdotgg in:name", 10, &FixtureGitHubTransport).unwrap();

    let locators = hits
        .iter()
        .map(|hit| hit.full_name.as_str())
        .collect::<Vec<_>>();

    assert_eq!(locators, vec!["pingdotgg/t3code"]);
}

#[test]
fn app_search_results_can_carry_local_matches_and_warnings() {
    let query = SearchQuery::new("bat");
    let installed = vec![AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: Some("1.0.0".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];
    let provider = FailingProvider;

    let results = build_search_results_with(&query, &installed, &[&provider]).unwrap();

    assert!(results.remote_hits.is_empty());
    assert_eq!(results.installed_matches.len(), 1);
    assert_eq!(results.installed_matches[0].stable_id, "bat");
    assert_eq!(results.installed_matches[0].display_name, "Bat");
    assert_eq!(results.warnings.len(), 1);
    assert_eq!(results.warnings[0].provider_id.as_deref(), Some("github"));
}

#[test]
fn github_search_marks_matching_current_install_as_installed() {
    let query = SearchQuery::new("bat");
    let installed = vec![installed_github_app("sharkdp/bat", "1.0.0")];
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &installed, &[&provider]).unwrap();
    let bat = results
        .remote_hits
        .iter()
        .find(|hit| hit.install_query == "sharkdp/bat")
        .unwrap();

    assert_eq!(
        bat.install_status,
        SearchInstallStatus::Installed {
            installed_version: Some("1.0.0".to_owned()),
        }
    );
}

#[test]
fn github_search_marks_older_install_as_update_available() {
    let query = SearchQuery::new("pingdotgg");
    let installed = vec![installed_github_app("pingdotgg/t3code", "0.0.11")];
    let provider = GitHubSearchProvider::new(&FixtureGitHubTransport);

    let results = build_search_results_with(&query, &installed, &[&provider]).unwrap();
    let t3code = results
        .remote_hits
        .iter()
        .find(|hit| hit.install_query == "pingdotgg/t3code")
        .unwrap();

    assert_eq!(t3code.version.as_deref(), Some("0.0.12"));
    assert_eq!(
        t3code.install_status,
        SearchInstallStatus::UpdateAvailable {
            installed_version: Some("0.0.11".to_owned()),
            latest_version: Some("0.0.12".to_owned()),
        }
    );
}

fn installed_github_app(locator: &str, installed_version: &str) -> AppRecord {
    AppRecord {
        stable_id: locator.replace('/', "-"),
        display_name: locator.split('/').next_back().unwrap().to_owned(),
        source_input: Some(locator.to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::GitHub,
            locator: locator.to_owned(),
            input_kind: SourceInputKind::RepoShorthand,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            canonical_locator: Some(locator.to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some(installed_version.to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }
}

struct FailingProvider;

impl SearchProvider for FailingProvider {
    fn search(
        &self,
        _query: &SearchQuery,
    ) -> Result<Vec<aim_core::domain::search::SearchResult>, SearchProviderError> {
        Err(SearchProviderError::new("github", "fixture rate limit"))
    }
}
