use aim_cli::DispatchResult;
use aim_cli::ui::prompt::render_interaction;
use aim_cli::ui::render::{render_dispatch_result, render_update_summary};
use aim_core::app::add::InstalledApp;
use aim_core::app::interaction::{InteractionKind, InteractionRequest};
use aim_core::app::list::ListRow;
use aim_core::app::remove::{RemovalPlan, RemovalResult};
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::domain::update::ArtifactCandidate;
use aim_core::domain::update::{ChannelPreference, PlannedUpdate, UpdateChannelKind, UpdatePlan};
use aim_core::integration::install::InstallOutcome;

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
