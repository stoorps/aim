use upm_core::adapters::direct_url::DirectUrlAdapter;
use upm_core::adapters::github::GitHubAdapter;
use upm_core::adapters::gitlab::GitLabAdapter;
use upm_core::adapters::sourceforge::SourceForgeAdapter;
use upm_core::adapters::traits::{
    AdapterCapabilities, AdapterError, AdapterResolution, AdapterResolveOutcome, SourceAdapter,
};
use upm_core::app::query::resolve_query;
use upm_core::domain::source::{
    NormalizedSourceKind, ResolvedRelease, SourceInputKind, SourceKind, SourceRef,
};

struct FileArtifactAdapter;

impl SourceAdapter for FileArtifactAdapter {
    fn id(&self) -> &'static str {
        "file"
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::exact_resolution_only()
    }

    fn exact_source_kind(&self) -> Option<SourceKind> {
        Some(SourceKind::File)
    }

    fn normalize(&self, _query: &str) -> Result<SourceRef, AdapterError> {
        Err(AdapterError::UnsupportedQuery)
    }

    fn resolve(&self, source: &SourceRef) -> Result<AdapterResolution, AdapterError> {
        Ok(AdapterResolution {
            source: source.clone(),
            release: ResolvedRelease {
                version: "file".to_owned(),
                prerelease: false,
            },
        })
    }
}

fn file_source() -> SourceRef {
    SourceRef {
        kind: SourceKind::File,
        locator: "/tmp/team-app.AppImage".to_owned(),
        input_kind: SourceInputKind::File,
        normalized_kind: NormalizedSourceKind::File,
        canonical_locator: None,
        requested_tag: None,
        requested_asset_name: None,
        tracks_latest: false,
    }
}

#[test]
fn adapter_capabilities_can_report_exact_resolution_only() {
    let capabilities = AdapterCapabilities::exact_resolution_only();
    assert!(!capabilities.supports_search);
}

#[test]
fn repository_backed_resolvers_accept_only_their_own_source_kind() {
    let github_source = resolve_query("sharkdp/bat").unwrap();
    let gitlab_source = resolve_query("https://gitlab.com/example/team/app").unwrap();

    let github_adapter: &dyn SourceAdapter = &GitHubAdapter;
    assert!(github_adapter.supports_source(&github_source));
    assert!(!github_adapter.supports_source(&gitlab_source));
    assert_eq!(
        github_adapter.resolve_source(&gitlab_source),
        Err(AdapterError::UnsupportedSource)
    );

    let gitlab_adapter: &dyn SourceAdapter = &GitLabAdapter;
    assert!(gitlab_adapter.supports_source(&gitlab_source));
    assert!(!gitlab_adapter.supports_source(&github_source));
    assert_eq!(
        gitlab_adapter.resolve_source(&github_source),
        Err(AdapterError::UnsupportedSource)
    );
}

#[test]
fn exact_resolution_resolvers_accept_only_exact_artifact_kinds() {
    let direct_url_adapter: &dyn SourceAdapter = &DirectUrlAdapter;
    let file_adapter: &dyn SourceAdapter = &FileArtifactAdapter;
    let direct_url_source = resolve_query("https://example.com/team-app.AppImage").unwrap();
    let github_source = resolve_query("sharkdp/bat").unwrap();
    let file_source = file_source();

    assert!(direct_url_adapter.supports_source(&direct_url_source));
    assert!(!direct_url_adapter.supports_source(&file_source));
    assert!(!direct_url_adapter.supports_source(&github_source));
    assert_eq!(
        direct_url_adapter.resolve_source(&github_source),
        Err(AdapterError::UnsupportedSource)
    );
    assert_eq!(
        direct_url_adapter.resolve_source(&file_source),
        Err(AdapterError::UnsupportedSource)
    );

    let direct_resolution = direct_url_adapter
        .resolve_source(&direct_url_source)
        .unwrap();
    assert!(matches!(
        direct_resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            release: ResolvedRelease { version, .. },
            ..
        }) if version == "unresolved"
    ));

    assert!(file_adapter.supports_source(&file_source));
    assert!(!file_adapter.supports_source(&direct_url_source));
    assert!(!file_adapter.supports_source(&github_source));
    assert_eq!(
        file_adapter.resolve_source(&direct_url_source),
        Err(AdapterError::UnsupportedSource)
    );

    let file_resolution = file_adapter.resolve_source(&file_source).unwrap();
    assert!(matches!(
        file_resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::File && version == "file"
    ));
}

#[test]
fn resolvers_can_return_no_installable_artifact_without_looking_unsupported() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;
    let source = resolve_query("https://sourceforge.net/projects/team-app/").unwrap();

    let resolution = adapter.resolve_source(&source).unwrap();

    assert_eq!(
        resolution,
        AdapterResolveOutcome::NoInstallableArtifact { source }
    );
}

#[test]
fn no_installable_artifact_outcomes_still_reject_unsupported_source_kinds() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;
    let unsupported_source = resolve_query("sharkdp/bat").unwrap();

    assert_eq!(
        adapter.resolve_source(&unsupported_source),
        Err(AdapterError::UnsupportedSource)
    );
}

#[test]
fn sourceforge_latest_download_sources_resolve_through_trait() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize("https://sourceforge.net/projects/team-app/files/latest/download")
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator == "https://sourceforge.net/projects/team-app/files/latest/download"
            && version == "latest"
    ));
}

#[test]
fn gitlab_candidate_sources_can_resolve_to_repository_semantics() {
    let adapter: &dyn SourceAdapter = &GitLabAdapter;

    let result = adapter
        .normalize("https://gitlab.com/acme/platform/releases/team-app")
        .unwrap();

    assert_eq!(result.kind, SourceKind::GitLab);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::GitLabCandidate
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::GitLab
            && source.locator == "https://gitlab.com/acme/platform/releases/team-app"
            && source.canonical_locator.as_deref() == Some("acme/platform/releases/team-app")
            && source.normalized_kind == NormalizedSourceKind::GitLab
            && source.tracks_latest
            && version == "latest"
    ));
}

#[test]
fn sourceforge_candidate_sources_can_resolve_to_latest_download() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize("https://sourceforge.net/projects/team-app/files/releases/stable/download")
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator
                == "https://sourceforge.net/projects/team-app/files/releases/stable/download"
            && version == "latest"
    ));
}

#[test]
fn sourceforge_version_folder_candidates_can_resolve_to_latest_download() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize("https://sourceforge.net/projects/team-app/files/releases/v1-0/download")
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator
                == "https://sourceforge.net/projects/team-app/files/releases/v1-0/download"
            && source.normalized_kind == NormalizedSourceKind::SourceForge
            && source.tracks_latest
            && version == "latest"
    ));
}

#[test]
fn sourceforge_prerelease_folder_candidates_can_resolve_to_latest_download() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize("https://sourceforge.net/projects/team-app/files/releases/beta/download")
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator
                == "https://sourceforge.net/projects/team-app/files/releases/beta/download"
            && source.normalized_kind == NormalizedSourceKind::SourceForge
            && source.tracks_latest
            && version == "latest"
    ));
}

#[test]
fn sourceforge_dotted_release_folder_candidates_can_resolve_to_latest_download() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize("https://sourceforge.net/projects/team-app/files/releases/2026.03/download")
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator
                == "https://sourceforge.net/projects/team-app/files/releases/2026.03/download"
            && source.normalized_kind == NormalizedSourceKind::SourceForge
            && source.tracks_latest
            && version == "latest"
    ));
}

#[test]
fn sourceforge_file_like_release_candidates_resolve_to_releases_root() {
    let adapter: &dyn SourceAdapter = &SourceForgeAdapter;

    let result = adapter
        .normalize(
            "https://sourceforge.net/projects/team-app/files/releases/team-app-1.0.0.AppImage/download",
        )
        .unwrap();

    assert_eq!(result.kind, SourceKind::SourceForge);
    assert_eq!(
        result.normalized_kind,
        NormalizedSourceKind::SourceForgeCandidate
    );
    assert_eq!(
        result.requested_asset_name.as_deref(),
        Some("team-app-1.0.0.AppImage")
    );

    let resolution = adapter.resolve_source(&result).unwrap();
    assert!(matches!(
        resolution,
        AdapterResolveOutcome::Resolved(AdapterResolution {
            source,
            release: ResolvedRelease { version, .. },
        }) if source.kind == SourceKind::SourceForge
            && source.locator
                == "https://sourceforge.net/projects/team-app/files/releases"
            && source.normalized_kind == NormalizedSourceKind::SourceForge
            && source.tracks_latest
            && source.requested_asset_name.as_deref() == Some("team-app-1.0.0.AppImage")
            && version == "latest"
    ));
}

#[test]
fn legacy_github_adapter_delegates_to_source_pipeline() {
    let adapter: &dyn SourceAdapter = &GitHubAdapter;

    let result = adapter.normalize("sharkdp/bat").unwrap();

    assert_eq!(result.normalized_kind.as_str(), "github-repository");
    assert_eq!(result.canonical_locator.as_deref(), Some("sharkdp/bat"));

    let resolution = adapter.resolve(&result).unwrap();
    assert_eq!(resolution.release.version, "latest");
}

#[test]
fn gitlab_adapter_normalizes_and_resolves_through_trait() {
    let adapter: &dyn SourceAdapter = &GitLabAdapter;

    let result = adapter
        .normalize("https://gitlab.com/example/team/app")
        .unwrap();

    assert_eq!(result.kind.as_str(), "gitlab");

    let resolution = adapter.resolve(&result).unwrap();
    assert_eq!(resolution.release.version, "latest");
}
