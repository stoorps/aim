use aim_core::app::query::resolve_query;
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind};

#[test]
fn owner_repo_defaults_to_github() {
    let source = resolve_query("sharkdp/bat").unwrap();
    assert_eq!(source.kind, SourceKind::GitHub);
    assert_eq!(source.input_kind, SourceInputKind::RepoShorthand);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::GitHubRepository
    );
}

#[test]
fn classifies_github_release_asset_url() {
    let source = resolve_query(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
    )
    .unwrap();

    assert_eq!(source.input_kind, SourceInputKind::GitHubReleaseAssetUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::GitHubReleaseAsset
    );
}
