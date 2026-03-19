use aim_core::app::query::resolve_query;
use aim_core::source::github::{FixtureGitHubTransport, discover_github_candidates_with};

#[test]
fn discovery_reports_appimage_assets_and_latest_linux_yml() {
    let source = resolve_query("pingdotgg/t3code").unwrap();
    let discovery = discover_github_candidates_with(&source, &FixtureGitHubTransport).unwrap();

    assert!(
        discovery
            .assets
            .iter()
            .any(|asset| asset.name.ends_with(".AppImage"))
    );
    assert!(
        discovery
            .metadata_documents
            .iter()
            .any(|doc| doc.url.ends_with("latest-linux.yml"))
    );
}

#[test]
fn discovery_marks_explicit_older_release_against_latest_fixture_release() {
    let source = resolve_query(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
    )
    .unwrap();
    let discovery = discover_github_candidates_with(&source, &FixtureGitHubTransport).unwrap();

    assert_eq!(discovery.releases[0].tag, "v0.0.12");
    assert!(discovery.requested_is_older_release);
}
