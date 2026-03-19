use aim_cli::ui::prompt::render_interaction;
use aim_cli::ui::render::render_update_summary;
use aim_core::app::interaction::{InteractionKind, InteractionRequest};

#[test]
fn update_summary_mentions_selected_count() {
    let output = render_update_summary(3, 2, 1);
    assert!(output.contains("selected: 2"));
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

    assert!(output.contains("tracking preference required"));
    assert!(output.contains("v0.0.11"));
    assert!(output.contains("v0.0.12"));
}
