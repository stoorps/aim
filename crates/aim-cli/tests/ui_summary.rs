use aim_cli::DispatchResult;
use aim_cli::ui::prompt::render_interaction;
use aim_cli::ui::render::{render_dispatch_result, render_update_summary};
use aim_cli::ui::search_browser::{SearchRow, format_search_row, render_confirmation_summary};
use aim_core::app::add::InstalledApp;
use aim_core::app::interaction::{InteractionKind, InteractionRequest};
use aim_core::app::list::ListRow;
use aim_core::app::remove::{RemovalPlan, RemovalResult};
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::search::SearchInstallStatus;
use aim_core::domain::show::{
    InstalledShow, MetadataSummary, RemoteArtifactSummary, RemoteShow, ShowResult, SourceSummary,
    TrackedInstallPaths, UpdateChannelSummary, UpdateStrategySummary,
};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::domain::update::ArtifactCandidate;
use aim_core::domain::update::{
    ChannelPreference, ParsedMetadataKind, PlannedUpdate, UpdateChannelKind, UpdatePlan,
};
use aim_core::integration::install::InstallOutcome;

fn muted_bold_label(title: &str) -> String {
    let mut style = aim_cli::ui::theme::current_theme().muted;
    style.bold = true;
    aim_cli::ui::theme::apply_style_spec(&format!("{title}:"), &style)
}

#[test]
fn update_summary_mentions_selected_count() {
    let output = render_update_summary(3, 2, 1);
    assert!(output.contains("selected: 2"));
}

#[test]
fn update_summary_uses_review_heading() {
    let output = render_update_summary(3, 2, 1);
    assert!(output.contains("Update Review"));
}

#[test]
fn list_empty_state_uses_friendlier_copy() {
    let output = render_dispatch_result(&DispatchResult::List(Vec::new()));
    assert!(output.contains("No installed apps yet"));
}

#[test]
fn list_renders_table_with_name_version_and_source() {
    let output = render_dispatch_result(&DispatchResult::List(vec![ListRow {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        version: Some("0.25.0".to_owned()),
        source: "sharkdp/bat".to_owned(),
    }]));

    assert!(output.contains("Name"));
    assert!(output.contains("Version"));
    assert!(output.contains("Source"));
    assert!(output.contains("Bat"));
    assert!(output.contains("0.25.0"));
    assert!(output.contains("sharkdp/bat"));
    assert!(!output.contains("Bat (bat)"));
}

#[test]
fn review_flow_uses_clearer_summary_labels() {
    let output = render_dispatch_result(&DispatchResult::UpdatePlan(UpdatePlan {
        items: vec![PlannedUpdate {
            stable_id: "bat".to_owned(),
            display_name: "Bat".to_owned(),
            selected_channel: ChannelPreference {
                kind: UpdateChannelKind::GitHubReleases,
                locator: "sharkdp/bat".to_owned(),
                reason: "install-origin-match".to_owned(),
            },
            selection_reason: "install-origin-match".to_owned(),
        }],
    }));

    assert!(output.contains("Update Review"));
    assert!(output.contains("apps with updates"));
}

#[test]
fn removal_summary_lists_removed_files() {
    let output = render_dispatch_result(&DispatchResult::Removed(Box::new(RemovalResult {
        removed: RemovalPlan {
            stable_id: "bat".to_owned(),
            display_name: "Bat".to_owned(),
            artifact_paths: vec![
                "/tmp/install-home/.local/lib/aim/appimages/bat.AppImage".to_owned(),
                "/tmp/install-home/.local/share/applications/aim-bat.desktop".to_owned(),
            ],
        },
        removed_paths: vec![
            "/tmp/install-home/.local/lib/aim/appimages/bat.AppImage".to_owned(),
            "/tmp/install-home/.local/share/applications/aim-bat.desktop".to_owned(),
        ],
        remaining_apps: Vec::new(),
        warnings: Vec::new(),
    })));

    assert!(output.contains("Removed files"));
    assert!(output.contains("bat.AppImage"));
    assert!(output.contains("aim-bat.desktop"));
}

#[test]
fn tracking_prompt_mentions_requested_and_latest_versions() {
    let output = render_interaction(&InteractionRequest {
        key: "tracking-preference".to_owned(),
        kind: InteractionKind::ChooseTrackingPreference {
            requested_version: "v0.0.11".to_owned(),
            latest_version: "v0.0.12".to_owned(),
        },
    });

    assert!(output.contains("Choose update tracking"));
    assert!(output.contains("v0.0.11"));
    assert!(output.contains("v0.0.12"));
}

#[test]
fn tracking_prompt_uses_explicit_question_copy() {
    let output = render_interaction(&InteractionRequest {
        key: "tracking-preference".to_owned(),
        kind: InteractionKind::ChooseTrackingPreference {
            requested_version: "v0.0.11".to_owned(),
            latest_version: "v0.0.12".to_owned(),
        },
    });

    assert!(output.contains("Choose update tracking"));
}

#[test]
fn install_summary_omits_completed_steps_recap() {
    let output = render_dispatch_result(&DispatchResult::Added(Box::new(InstalledApp {
        record: AppRecord {
            stable_id: "bat".to_owned(),
            display_name: "bat".to_owned(),
            source_input: Some("sharkdp/bat".to_owned()),
            source: None,
            installed_version: Some("0.25.0".to_owned()),
            update_strategy: None,
            metadata: Vec::new(),
            install: Some(InstallMetadata {
                scope: InstallScope::User,
                payload_path: Some(
                    "/tmp/install-home/.local/lib/aim/appimages/sharkdp-bat.AppImage".to_owned(),
                ),
                desktop_entry_path: Some(
                    "/tmp/install-home/.local/share/applications/aim-sharkdp-bat.desktop"
                        .to_owned(),
                ),
                icon_path: None,
            }),
        },
        selected_artifact: ArtifactCandidate {
            url: "https://github.com/sharkdp/bat/releases/download/v0.25.0/bat-x86_64.AppImage"
                .to_owned(),
            version: "0.25.0".to_owned(),
            arch: Some("x86_64".to_owned()),
            trusted_checksum: None,
            selection_reason: "heuristic-match".to_owned(),
        },
        artifact_size_bytes: 173_015_040,
        source: SourceRef {
            kind: SourceKind::GitHub,
            input_kind: SourceInputKind::RepoShorthand,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            locator: "sharkdp/bat".to_owned(),
            canonical_locator: Some("sharkdp/bat".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        },
        install_scope: InstallScope::User,
        integration_mode: aim_core::integration::policy::IntegrationMode::Full,
        install_outcome: InstallOutcome {
            final_payload_path: "/tmp/install-home/.local/lib/aim/appimages/sharkdp-bat.AppImage"
                .into(),
            desktop_entry_path: Some(
                "/tmp/install-home/.local/share/applications/aim-sharkdp-bat.desktop".into(),
            ),
            icon_path: None,
            warnings: Vec::new(),
        },
        warnings: Vec::new(),
    })));

    assert!(output.contains("Installed bat (user)"));
    assert!(output.contains("Installed files"));
    assert!(!output.contains("Completed steps"));
}

#[test]
fn search_browser_row_uses_status_tag_version_and_description_layout() {
    let row = SearchRow {
        status: SearchInstallStatus::Installed {
            installed_version: Some("0.0.12".to_owned()),
        },
        provider_id: "github".to_owned(),
        display_name: "pingdotgg/t3code".to_owned(),
        description: Some("The T3 desktop app.".to_owned()),
        install_query: "pingdotgg/t3code".to_owned(),
        version: Some("0.0.12".to_owned()),
        selectable: false,
    };

    let output = format_search_row(1, &row, true, true, 120);

    assert!(output.contains('\n'));
    assert!(output.contains("[installed]"));
    assert!(output.contains("v0.0.12"));
    assert!(output.contains("pingdotgg/t3code"));
    assert!(output.contains("github - The T3 desktop app."));
}

#[test]
fn search_browser_row_without_description_shows_provider_only() {
    let row = SearchRow {
        status: SearchInstallStatus::Available,
        provider_id: "github".to_owned(),
        display_name: "pingdotgg/t3code".to_owned(),
        description: None,
        install_query: "pingdotgg/t3code".to_owned(),
        version: Some("0.0.12".to_owned()),
        selectable: true,
    };

    let output = format_search_row(1, &row, false, false, 120);

    assert!(output.contains("github"));
    assert!(!output.contains(" - "));
    assert!(!output.contains("No description available"));
}

#[test]
fn search_confirmation_summary_lists_selected_rows() {
    let rows = vec![
        SearchRow {
            status: SearchInstallStatus::UpdateAvailable {
                installed_version: Some("0.0.11".to_owned()),
                latest_version: Some("0.0.12".to_owned()),
            },
            provider_id: "github".to_owned(),
            display_name: "pingdotgg/t3code".to_owned(),
            description: Some("The T3 desktop app.".to_owned()),
            install_query: "pingdotgg/t3code".to_owned(),
            version: Some("0.0.12".to_owned()),
            selectable: true,
        },
        SearchRow {
            status: SearchInstallStatus::Available,
            provider_id: "github".to_owned(),
            display_name: "sharkdp/bat".to_owned(),
            description: Some("A cat(1) clone with wings.".to_owned()),
            install_query: "sharkdp/bat".to_owned(),
            version: Some("1.0.0".to_owned()),
            selectable: true,
        },
    ];

    let output = render_confirmation_summary(&rows);

    assert!(output.contains("Confirm Search Selection"));
    assert!(output.contains("pingdotgg/t3code"));
    assert!(output.contains("sharkdp/bat"));
}

#[test]
fn installed_show_summary_renders_source_scope_and_paths() {
    let output = render_dispatch_result(&DispatchResult::Show(Box::new(ShowResult::Installed(
        InstalledShow {
            stable_id: "legacy-bat".to_owned(),
            display_name: "Legacy Bat".to_owned(),
            installed_version: Some("0.24.0".to_owned()),
            source_input: Some("sharkdp/bat".to_owned()),
            source: Some(SourceSummary {
                kind: SourceKind::GitHub,
                locator: "https://github.com/sharkdp/bat".to_owned(),
                canonical_locator: Some("sharkdp/bat".to_owned()),
            }),
            install_scope: Some(InstallScope::User),
            tracked_paths: TrackedInstallPaths {
                payload_path: Some("/tmp/bat.AppImage".to_owned()),
                desktop_entry_path: Some("/tmp/aim-bat.desktop".to_owned()),
                icon_path: Some("/tmp/aim-bat.png".to_owned()),
            },
            update_strategy: Some(UpdateStrategySummary {
                preferred: UpdateChannelSummary {
                    kind: UpdateChannelKind::GitHubReleases,
                    locator: "sharkdp/bat".to_owned(),
                    reason: "install-origin-match".to_owned(),
                },
                alternates: Vec::new(),
            }),
            metadata: vec![
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.24.0".to_owned()),
                    primary_download: Some("https://example.test/bat.AppImage".to_owned()),
                    checksum: Some("sha256:abcdefghijklmnopqrstuvwxyz0123456789".to_owned()),
                    architecture: Some("x86_64".to_owned()),
                    channel_label: None,
                    warnings: Vec::new(),
                },
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.23.0".to_owned()),
                    primary_download: Some("https://example.test/bat-0.23.0.AppImage".to_owned()),
                    checksum: Some("sha256:efgh".to_owned()),
                    architecture: Some("x86_64".to_owned()),
                    channel_label: None,
                    warnings: Vec::new(),
                },
            ],
        },
    ))));

    assert!(output.contains("Legacy Bat (legacy-bat)"));
    assert!(output.contains("v0.24.0"));
    assert!(output.contains("[up to date]"));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Source"),
        aim_cli::ui::theme::muted("github - sharkdp/bat")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Update Mechanism"),
        aim_cli::ui::theme::muted("electron-builder")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Architecture"),
        aim_cli::ui::theme::muted("x86_64")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Checksum"),
        aim_cli::ui::theme::muted("sha256:abcdefg...456789")
    )));
    assert!(output.contains(&muted_bold_label("Installed as User")));
    assert!(output.contains("/tmp/bat.AppImage"));
    assert!(output.contains("/tmp/aim-bat.desktop"));
    assert!(!output.contains("[up to date]  User"));
    assert!(!output.contains("past version"));
    assert!(!output.contains(&aim_cli::ui::theme::label("Metadata")));
    assert!(!output.contains(&aim_cli::ui::theme::label("Files")));
    assert!(!output.contains("abcdefghijklmnopqrstuvwxyz0123456789"));
}

#[test]
fn installed_show_summary_reports_when_newer_versions_are_available() {
    let output = render_dispatch_result(&DispatchResult::Show(Box::new(ShowResult::Installed(
        InstalledShow {
            stable_id: "t3code".to_owned(),
            display_name: "t3code".to_owned(),
            installed_version: Some("0.0.13".to_owned()),
            source_input: Some("pingdotgg/t3code".to_owned()),
            source: Some(SourceSummary {
                kind: SourceKind::GitHub,
                locator: "pingdotgg/t3code".to_owned(),
                canonical_locator: Some("pingdotgg/t3code".to_owned()),
            }),
            install_scope: Some(InstallScope::User),
            tracked_paths: TrackedInstallPaths {
                payload_path: Some("/tmp/t3code.AppImage".to_owned()),
                desktop_entry_path: None,
                icon_path: None,
            },
            update_strategy: Some(UpdateStrategySummary {
                preferred: UpdateChannelSummary {
                    kind: UpdateChannelKind::ElectronBuilder,
                    locator: "https://github.com/pingdotgg/t3code/releases/download/v0.0.16/latest-linux.yml"
                        .to_owned(),
                    reason: "install-origin-match".to_owned(),
                },
                alternates: Vec::new(),
            }),
            metadata: vec![
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.0.16".to_owned()),
                    primary_download: None,
                    checksum: None,
                    architecture: Some("x86_64".to_owned()),
                    channel_label: Some("latest".to_owned()),
                    warnings: Vec::new(),
                },
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.0.15".to_owned()),
                    primary_download: None,
                    checksum: None,
                    architecture: Some("x86_64".to_owned()),
                    channel_label: Some("latest".to_owned()),
                    warnings: Vec::new(),
                },
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.0.14".to_owned()),
                    primary_download: None,
                    checksum: None,
                    architecture: Some("x86_64".to_owned()),
                    channel_label: Some("latest".to_owned()),
                    warnings: Vec::new(),
                },
                MetadataSummary {
                    kind: ParsedMetadataKind::ElectronBuilder,
                    version: Some("0.0.13".to_owned()),
                    primary_download: None,
                    checksum: None,
                    architecture: Some("x86_64".to_owned()),
                    channel_label: Some("latest".to_owned()),
                    warnings: Vec::new(),
                },
            ],
        },
    ))));

    assert!(output.contains("t3code (t3code)"));
    assert!(output.contains("v0.0.13"));
    assert!(output.contains("[update available]"));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Source"),
        aim_cli::ui::theme::muted("github - pingdotgg/t3code")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Update Mechanism"),
        aim_cli::ui::theme::muted("electron-builder")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Architecture"),
        aim_cli::ui::theme::muted("x86_64")
    )));
    assert!(output.contains(&muted_bold_label("Installed as User")));
    assert!(!output.contains("[update available]  User"));
    assert!(!output.contains("past versions"));
    assert!(!output.contains("latest v0.0.16"));
    assert!(!output.contains(&aim_cli::ui::theme::label("Metadata")));
    assert!(!output.contains(&aim_cli::ui::theme::label("Files")));
}

#[test]
fn installed_show_list_renders_each_app_using_singular_show_format() {
    let output = render_dispatch_result(&DispatchResult::ShowAll(vec![
        InstalledShow {
            stable_id: "legacy-bat".to_owned(),
            display_name: "Legacy Bat".to_owned(),
            installed_version: Some("0.24.0".to_owned()),
            source_input: Some("sharkdp/bat".to_owned()),
            source: Some(SourceSummary {
                kind: SourceKind::GitHub,
                locator: "https://github.com/sharkdp/bat".to_owned(),
                canonical_locator: Some("sharkdp/bat".to_owned()),
            }),
            install_scope: Some(InstallScope::User),
            tracked_paths: TrackedInstallPaths {
                payload_path: Some("/tmp/bat.AppImage".to_owned()),
                desktop_entry_path: Some("/tmp/aim-bat.desktop".to_owned()),
                icon_path: None,
            },
            update_strategy: None,
            metadata: vec![MetadataSummary {
                kind: ParsedMetadataKind::ElectronBuilder,
                version: Some("0.24.0".to_owned()),
                primary_download: None,
                checksum: Some("sha256:abcdefghijklmnopqrstuvwxyz0123456789".to_owned()),
                architecture: Some("x86_64".to_owned()),
                channel_label: None,
                warnings: Vec::new(),
            }],
        },
        InstalledShow {
            stable_id: "t3code".to_owned(),
            display_name: "t3code".to_owned(),
            installed_version: Some("0.0.13".to_owned()),
            source_input: Some("pingdotgg/t3code".to_owned()),
            source: Some(SourceSummary {
                kind: SourceKind::GitHub,
                locator: "pingdotgg/t3code".to_owned(),
                canonical_locator: Some("pingdotgg/t3code".to_owned()),
            }),
            install_scope: Some(InstallScope::User),
            tracked_paths: TrackedInstallPaths {
                payload_path: Some("/tmp/t3code.AppImage".to_owned()),
                desktop_entry_path: None,
                icon_path: None,
            },
            update_strategy: None,
            metadata: vec![MetadataSummary {
                kind: ParsedMetadataKind::ElectronBuilder,
                version: Some("0.0.16".to_owned()),
                primary_download: None,
                checksum: None,
                architecture: Some("x86_64".to_owned()),
                channel_label: None,
                warnings: Vec::new(),
            }],
        },
    ]));

    assert!(output.contains("Legacy Bat (legacy-bat)"));
    assert!(output.contains("t3code (t3code)"));
    assert!(output.contains("\n\n"));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Source"),
        aim_cli::ui::theme::muted("github - sharkdp/bat")
    )));
    assert!(output.contains(&format!(
        "{} {}",
        muted_bold_label("Source"),
        aim_cli::ui::theme::muted("github - pingdotgg/t3code")
    )));
}

#[test]
fn remote_show_summary_renders_source_artifact_and_reason() {
    let output = render_dispatch_result(&DispatchResult::Show(Box::new(ShowResult::Remote(
        RemoteShow {
            source: SourceSummary {
                kind: SourceKind::GitHub,
                locator: "sharkdp/bat".to_owned(),
                canonical_locator: Some("sharkdp/bat".to_owned()),
            },
            artifact: RemoteArtifactSummary {
                url: "https://github.com/sharkdp/bat/releases/download/v1.0.0/Bat-1.0.0-x86_64.AppImage"
                    .to_owned(),
                version: Some("1.0.0".to_owned()),
                arch: Some("x86_64".to_owned()),
                trusted_checksum: Some("sha512:abcd".to_owned()),
                selection_reason: "metadata-guided".to_owned(),
            },
            interactions: Vec::new(),
            warnings: Vec::new(),
        },
    ))));

    assert!(output.contains("Resolved Source"));
    assert!(output.contains("github"));
    assert!(output.contains("sharkdp/bat"));
    assert!(output.contains("Bat-1.0.0-x86_64.AppImage"));
    assert!(output.contains("1.0.0"));
    assert!(output.contains("metadata-guided"));
    assert!(output.contains("sha512:abcd"));
}
