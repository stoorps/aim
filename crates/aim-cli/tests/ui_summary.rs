use aim_cli::DispatchResult;
use aim_cli::ui::prompt::render_interaction;
use aim_cli::ui::render::{render_dispatch_result, render_update_summary};
use aim_cli::ui::search_browser::{SearchRow, format_search_row, render_confirmation_summary};
use aim_core::app::interaction::{InteractionKind, InteractionRequest};
use aim_core::domain::search::SearchInstallStatus;
use aim_core::domain::update::{ChannelPreference, PlannedUpdate, UpdateChannelKind, UpdatePlan};

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
