use upm_core::app::add::{build_add_plan_with, materialize_app_record, prefer_latest_tracking};
use upm_core::app::query::resolve_query;
use upm_core::source::github::FixtureGitHubTransport;

#[test]
fn github_adapter_can_normalize_owner_repo_source() {
    let source = resolve_query("sharkdp/bat").unwrap();

    assert_eq!(source.kind.as_str(), "github");
}

#[test]
fn add_flow_builds_github_plan_from_owner_repo_query() {
    let plan = build_add_plan_with("sharkdp/bat", &FixtureGitHubTransport).unwrap();

    assert_eq!(plan.resolution.source.kind.as_str(), "github");
    assert_eq!(plan.resolution.source.locator, "sharkdp/bat");
    assert_eq!(plan.selected_artifact.selection_reason, "metadata-guided");
}

#[test]
fn add_plan_prefers_metadata_guided_appimage_when_available() {
    let plan = build_add_plan_with("pingdotgg/t3code", &FixtureGitHubTransport).unwrap();

    assert_eq!(plan.selected_artifact.selection_reason, "metadata-guided");
    assert_eq!(
        plan.update_strategy.preferred.kind.as_str(),
        "electron-builder"
    );
}

#[test]
fn direct_old_release_url_requests_tracking_choice_prompt() {
    let plan = build_add_plan_with(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
        &FixtureGitHubTransport,
    )
    .unwrap();

    assert!(
        plan.interactions
            .iter()
            .any(|item| item.key == "tracking-preference")
    );
}

#[test]
fn materialized_record_preserves_source_and_strategy() {
    let query = "sharkdp/bat";
    let plan = build_add_plan_with(query, &FixtureGitHubTransport).unwrap();

    let record = materialize_app_record(query, &plan).unwrap();

    assert_eq!(record.stable_id, "sharkdp-bat");
    assert_eq!(record.display_name, "bat");
    assert_eq!(record.source_input.as_deref(), Some(query));
    assert_eq!(record.installed_version.as_deref(), Some("1.0.0"));
    assert_eq!(
        record
            .update_strategy
            .as_ref()
            .unwrap()
            .preferred
            .kind
            .as_str(),
        "electron-builder"
    );
    assert_eq!(record.source.as_ref().unwrap().locator, query);
}

#[test]
fn latest_tracking_choice_promotes_non_direct_update_channel() {
    let plan = build_add_plan_with(
        "https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage",
        &FixtureGitHubTransport,
    )
    .unwrap();

    let resolved = prefer_latest_tracking(plan);

    assert!(resolved.interactions.is_empty());
    assert_eq!(resolved.resolution.source.locator, "pingdotgg/t3code");
    assert!(resolved.resolution.source.tracks_latest);
    assert_ne!(
        resolved.update_strategy.preferred.kind.as_str(),
        "direct-asset-lineage"
    );
}
