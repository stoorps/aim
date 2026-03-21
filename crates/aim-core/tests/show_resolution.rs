use aim_core::app::show::{build_show_result, build_show_result_with};
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::show::{ShowResult, ShowResultError};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::domain::update::{
    ChannelPreference, MetadataHints, ParsedMetadata, ParsedMetadataKind, UpdateChannelKind,
    UpdateStrategy,
};
use aim_core::source::github::FixtureGitHubTransport;

#[test]
fn exact_installed_match_returns_installed_details() {
    let apps = vec![AppRecord {
        stable_id: "legacy-bat".to_owned(),
        display_name: "Legacy Bat".to_owned(),
        source_input: Some("sharkdp/bat".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::GitHub,
            locator: "https://github.com/sharkdp/bat".to_owned(),
            input_kind: SourceInputKind::RepoShorthand,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            canonical_locator: Some("sharkdp/bat".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("0.24.0".to_owned()),
        update_strategy: Some(UpdateStrategy {
            preferred: ChannelPreference {
                kind: UpdateChannelKind::GitHubReleases,
                locator: "sharkdp/bat".to_owned(),
                reason: "install-origin-match".to_owned(),
            },
            alternates: Vec::new(),
        }),
        metadata: vec![ParsedMetadata {
            kind: ParsedMetadataKind::ElectronBuilder,
            hints: MetadataHints {
                version: Some("0.24.0".to_owned()),
                primary_download: Some("https://example.test/bat.AppImage".to_owned()),
                checksum: Some("sha256:abcd".to_owned()),
                architecture: Some("x86_64".to_owned()),
                channel_label: None,
            },
            warnings: Vec::new(),
            confidence: 90,
        }],
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: Some("/tmp/bat.AppImage".to_owned()),
            desktop_entry_path: Some("/tmp/aim-bat.desktop".to_owned()),
            icon_path: Some("/tmp/aim-bat.png".to_owned()),
        }),
    }];

    let result = build_show_result("legacy-bat", &apps).unwrap();

    match result {
        ShowResult::Installed(installed) => {
            assert_eq!(installed.stable_id, "legacy-bat");
            assert_eq!(installed.display_name, "Legacy Bat");
            assert_eq!(installed.installed_version.as_deref(), Some("0.24.0"));
            assert_eq!(installed.install_scope, Some(InstallScope::User));
            assert_eq!(
                installed.source.as_ref().unwrap().locator,
                "https://github.com/sharkdp/bat"
            );
            assert_eq!(
                installed.tracked_paths.payload_path.as_deref(),
                Some("/tmp/bat.AppImage")
            );
            assert!(installed.update_strategy.is_some());
            assert_eq!(installed.metadata.len(), 1);
        }
        other => panic!("expected installed result, got {other:?}"),
    }
}

#[test]
fn installed_source_lineage_matches_before_remote_fallback() {
    let apps = vec![AppRecord {
        stable_id: "legacy-bat".to_owned(),
        display_name: "Legacy Bat".to_owned(),
        source_input: Some("sharkdp/bat".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::GitHub,
            locator: "https://github.com/sharkdp/bat".to_owned(),
            input_kind: SourceInputKind::RepoShorthand,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            canonical_locator: Some("sharkdp/bat".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("0.24.0".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];

    let result = build_show_result_with("sharkdp/bat", &apps, &FixtureGitHubTransport).unwrap();

    match result {
        ShowResult::Installed(installed) => {
            assert_eq!(installed.stable_id, "legacy-bat");
            assert_eq!(installed.source_input.as_deref(), Some("sharkdp/bat"));
        }
        other => panic!("expected installed result, got {other:?}"),
    }
}

#[test]
fn installed_direct_url_show_omits_unresolved_version() {
    let apps = vec![AppRecord {
        stable_id: "team-app".to_owned(),
        display_name: "team-app".to_owned(),
        source_input: Some("https://example.com/downloads/team-app.AppImage".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::DirectUrl,
            locator: "https://example.com/downloads/team-app.AppImage".to_owned(),
            input_kind: SourceInputKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        }),
        installed_version: Some("unresolved".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];

    let result = build_show_result("team-app", &apps).unwrap();

    match result {
        ShowResult::Installed(installed) => {
            assert_eq!(installed.installed_version, None);
            assert_eq!(
                installed.source.as_ref().unwrap().kind,
                SourceKind::DirectUrl
            );
        }
        other => panic!("expected installed result, got {other:?}"),
    }
}

#[test]
fn no_installed_match_falls_back_to_remote_resolution() {
    let result = build_show_result_with("sharkdp/bat", &[], &FixtureGitHubTransport).unwrap();

    match result {
        ShowResult::Remote(remote) => {
            assert_eq!(remote.source.kind, SourceKind::GitHub);
            assert_eq!(
                remote.source.canonical_locator.as_deref(),
                Some("sharkdp/bat")
            );
            assert!(remote.artifact.url.ends_with("Bat-1.0.0-x86_64.AppImage"));
            assert_eq!(remote.artifact.version.as_deref(), Some("1.0.0"));
            assert!(remote.artifact.trusted_checksum.is_some());
            assert!(!remote.artifact.selection_reason.is_empty());
            assert!(remote.interactions.is_empty());
            assert!(remote.warnings.is_empty());
        }
        other => panic!("expected remote result, got {other:?}"),
    }
}

#[test]
fn remote_show_projects_tracking_preference_interaction() {
    let result = build_show_result_with(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
        &[],
        &FixtureGitHubTransport,
    )
    .unwrap();

    match result {
        ShowResult::Remote(remote) => {
            assert!(remote.interactions.iter().any(|interaction| matches!(
                interaction,
                aim_core::domain::show::RemoteInteractionSummary::ChooseTrackingPreference { .. }
            )));
        }
        other => panic!("expected remote result, got {other:?}"),
    }
}

#[test]
fn direct_url_remote_show_omits_unresolved_version() {
    let result = build_show_result_with(
        "https://example.com/downloads/team-app.AppImage",
        &[],
        &FixtureGitHubTransport,
    )
    .unwrap();

    match result {
        ShowResult::Remote(remote) => {
            assert_eq!(remote.source.kind, SourceKind::DirectUrl);
            assert_eq!(remote.artifact.version, None);
            assert_eq!(
                remote.artifact.url,
                "https://example.com/downloads/team-app.AppImage"
            );
        }
        other => panic!("expected remote result, got {other:?}"),
    }
}

#[test]
fn ambiguous_installed_matches_return_dedicated_error() {
    let apps = vec![
        AppRecord {
            stable_id: "bat".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
        AppRecord {
            stable_id: "legacy-bat".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
    ];

    let error = build_show_result("bat", &apps).unwrap_err();

    match error {
        ShowResultError::AmbiguousInstalledMatch { matches, .. } => {
            assert_eq!(matches.len(), 2);
            assert!(matches.iter().any(|item: &String| item.contains("bat")));
            assert!(
                matches
                    .iter()
                    .any(|item: &String| item.contains("legacy-bat"))
            );
        }
        other => panic!("expected ambiguous installed match, got {other:?}"),
    }
}

#[test]
fn ambiguous_installed_match_blocks_valid_remote_fallback() {
    let apps = vec![
        AppRecord {
            stable_id: "bat-alpha".to_owned(),
            display_name: "sharkdp/bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
        AppRecord {
            stable_id: "bat-beta".to_owned(),
            display_name: "sharkdp/bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
    ];

    let error = build_show_result_with("sharkdp/bat", &apps, &FixtureGitHubTransport).unwrap_err();

    assert!(matches!(
        error,
        ShowResultError::AmbiguousInstalledMatch { .. }
    ));
}

#[test]
fn unsupported_query_stays_distinct_from_no_installable_artifact() {
    let unsupported =
        build_show_result_with("https://gitlab.com/example", &[], &FixtureGitHubTransport)
            .unwrap_err();
    let no_artifact = build_show_result_with(
        "https://sourceforge.net/projects/team-app/",
        &[],
        &FixtureGitHubTransport,
    )
    .unwrap_err();

    assert!(matches!(unsupported, ShowResultError::UnsupportedQuery));
    assert!(matches!(
        no_artifact,
        ShowResultError::NoInstallableArtifact { .. }
    ));
}
