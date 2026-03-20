use aim_cli::DispatchResult;
use aim_cli::ui::prompt::render_interaction;
use aim_cli::ui::render::{render_dispatch_result, render_update_summary};
use aim_core::app::interaction::{InteractionKind, InteractionRequest};
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
