use upm_core::app::query::resolve_query;
use upm_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind};

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

#[test]
fn classifies_appimagehub_item_url() {
    let source = resolve_query("https://www.appimagehub.com/p/2338455").unwrap();

    assert_eq!(source.kind, SourceKind::AppImageHub);
    assert_eq!(source.input_kind, SourceInputKind::AppImageHubUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::AppImageHub);
    assert_eq!(source.canonical_locator.as_deref(), Some("2338455"));
    assert!(source.tracks_latest);
}

#[test]
fn classifies_appimagehub_id_shorthand() {
    let source = resolve_query("appimagehub/2338455").unwrap();

    assert_eq!(source.kind, SourceKind::AppImageHub);
    assert_eq!(source.input_kind, SourceInputKind::AppImageHubShorthand);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::AppImageHub);
    assert_eq!(source.locator, "https://www.appimagehub.com/p/2338455");
    assert_eq!(source.canonical_locator.as_deref(), Some("2338455"));
    assert!(source.tracks_latest);
}

#[test]
fn classifies_gitlab_repository_url() {
    let source = resolve_query("https://gitlab.com/example/team-app").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(source.input_kind, SourceInputKind::GitLabUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::GitLab);
    assert_eq!(
        source.canonical_locator.as_deref(),
        Some("example/team-app")
    );
    assert!(source.tracks_latest);
}

#[test]
fn classifies_gitlab_release_like_url() {
    let source = resolve_query("https://gitlab.com/example/team-app/-/releases/v1.2.3").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(source.input_kind, SourceInputKind::GitLabUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::GitLab);
    assert_eq!(
        source.canonical_locator.as_deref(),
        Some("example/team-app")
    );
    assert_eq!(source.requested_tag.as_deref(), Some("v1.2.3"));
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_gitlab_subgroup_repository_url() {
    let source = resolve_query("https://gitlab.com/example/platform/team-app").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(
        source.canonical_locator.as_deref(),
        Some("example/platform/team-app")
    );
    assert!(source.tracks_latest);
}

#[test]
fn classifies_gitlab_deep_subgroup_repository_url() {
    let source = resolve_query("https://gitlab.com/example/platform/apps/team-app").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(
        source.canonical_locator.as_deref(),
        Some("example/platform/apps/team-app")
    );
    assert!(source.tracks_latest);
}

#[test]
fn classifies_gitlab_repository_with_reserved_namespace_segment() {
    let source = resolve_query("https://gitlab.com/example/releases/team-app").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(
        source.canonical_locator.as_deref(),
        Some("example/releases/team-app")
    );
}

#[test]
fn classifies_gitlab_two_segment_repository_with_reserved_slug() {
    let source = resolve_query("https://gitlab.com/example/issues").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(source.canonical_locator.as_deref(), Some("example/issues"));
    assert!(source.tracks_latest);
}

#[test]
fn classifies_sourceforge_project_url() {
    let source = resolve_query("https://sourceforge.net/projects/team-app/").unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::SourceForge);
}

#[test]
fn classifies_sourceforge_files_url() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/latest/download").unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::SourceForge);
}

#[test]
fn preserves_direct_url_classification() {
    let source = resolve_query("https://example.com/downloads/team-app.AppImage").unwrap();

    assert_eq!(source.kind, SourceKind::DirectUrl);
    assert_eq!(source.input_kind, SourceInputKind::DirectUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::DirectUrl);
}

#[test]
fn classifies_single_segment_sourceforge_release_download_as_candidate() {
    let source = resolve_query(
        "https://sourceforge.net/projects/team-app/files/releases/team-app-1.0.0.AppImage/download",
    )
    .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert_eq!(
        source.requested_asset_name.as_deref(),
        Some("team-app-1.0.0.AppImage")
    );
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_sourceforge_releases_root_as_provider_source() {
    let source = resolve_query("https://sourceforge.net/projects/team-app/files/releases").unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::SourceForge);
    assert_eq!(
        source.locator,
        "https://sourceforge.net/projects/team-app/files/releases"
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(source.tracks_latest);
}

#[test]
fn preserves_sourceforge_root_download_url_as_direct_url() {
    let source = resolve_query(
        "https://sourceforge.net/projects/team-app/files/team-app-1.0.0.AppImage/download",
    )
    .unwrap();

    assert_eq!(source.kind, SourceKind::DirectUrl);
    assert_eq!(source.input_kind, SourceInputKind::DirectUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::DirectUrl);
}

#[test]
fn preserves_sourceforge_extensionless_root_download_url_as_direct_url() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/team-app/download").unwrap();

    assert_eq!(source.kind, SourceKind::DirectUrl);
    assert_eq!(source.input_kind, SourceInputKind::DirectUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::DirectUrl);
}

#[test]
fn classifies_single_segment_sourceforge_release_download_with_query_as_candidate() {
    let source = resolve_query(
        "https://sourceforge.net/projects/team-app/files/releases/team-app-1.0.0.AppImage/download?use_mirror=pilotfiber",
    )
    .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert_eq!(
        source.requested_asset_name.as_deref(),
        Some("team-app-1.0.0.AppImage")
    );
    assert!(!source.tracks_latest);
}

#[test]
fn rejects_malformed_gitlab_url() {
    let error = resolve_query("https://gitlab.com/example").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_gitlab_url_shape() {
    let error = resolve_query("https://gitlab.com/example/team-app/-/issues").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_gitlab_nested_resource_url() {
    let error = resolve_query("https://gitlab.com/example/team-app/issues").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_gitlab_release_permalink_url() {
    let error = resolve_query("https://gitlab.com/example/team-app/-/releases/permalink/latest")
        .unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_gitlab_issue_detail_url() {
    let error = resolve_query("https://gitlab.com/example/team-app/issues/1").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_gitlab_blob_url() {
    let error =
        resolve_query("https://gitlab.com/example/team-app/blob/main/README.md").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn classifies_ambiguous_gitlab_deep_reserved_segment_as_candidate() {
    let source = resolve_query("https://gitlab.com/acme/platform/releases/team-app").unwrap();

    assert_eq!(source.kind, SourceKind::GitLab);
    assert_eq!(source.input_kind, SourceInputKind::GitLabUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::GitLabCandidate
    );
    assert_eq!(source.canonical_locator, None);
    assert!(!source.tracks_latest);
}

#[test]
fn rejects_unsupported_gitlab_packages_url() {
    let error = resolve_query("https://gitlab.com/example/team-app/packages").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_malformed_sourceforge_url() {
    let error = resolve_query("https://sourceforge.net/projects/").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_malformed_appimagehub_shorthand() {
    let error = resolve_query("appimagehub/firefox").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn rejects_unsupported_sourceforge_url_shape() {
    let error = resolve_query("https://sourceforge.net/projects/team-app/rss").unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn classifies_sourceforge_files_releases_shape_as_provider_source() {
    let source = resolve_query("https://sourceforge.net/projects/team-app/files/releases").unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(source.normalized_kind, NormalizedSourceKind::SourceForge);
    assert_eq!(
        source.locator,
        "https://sourceforge.net/projects/team-app/files/releases"
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(source.tracks_latest);
}

#[test]
fn rejects_unsupported_sourceforge_folder_download_shape() {
    let error = resolve_query("https://sourceforge.net/projects/team-app/files/releases/download")
        .unwrap_err();

    assert_eq!(error, upm_core::app::query::ResolveQueryError::Unsupported);
}

#[test]
fn classifies_ambiguous_sourceforge_nested_folder_download_as_candidate() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/releases/stable/download")
            .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_extensionless_sourceforge_release_folder_download_as_candidate() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/releases/team-app/download")
            .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_ambiguous_sourceforge_version_folder_download_as_candidate() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/releases/v1-0/download")
            .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_prerelease_named_sourceforge_release_folder_download_as_candidate() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/releases/beta/download")
            .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(!source.tracks_latest);
}

#[test]
fn classifies_dotted_sourceforge_release_folder_download_as_candidate() {
    let source =
        resolve_query("https://sourceforge.net/projects/team-app/files/releases/2026.03/download")
            .unwrap();

    assert_eq!(source.kind, SourceKind::SourceForge);
    assert_eq!(source.input_kind, SourceInputKind::SourceForgeUrl);
    assert_eq!(
        source.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
    assert!(!source.tracks_latest);
}
