use aim_core::app::update::build_update_plan;
use aim_core::domain::app::AppRecord;
use aim_core::domain::update::{ChannelPreference, UpdateChannelKind, UpdateStrategy};

#[test]
fn empty_registry_produces_empty_plan() {
    let plan = build_update_plan(&[]).unwrap();

    assert!(plan.items.is_empty());
}

#[test]
fn installed_apps_are_carried_into_review_plan() {
    let apps = [AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: None,
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];

    let plan = build_update_plan(&apps).unwrap();

    assert_eq!(plan.items.len(), 1);
    assert_eq!(plan.items[0].stable_id, "bat");
    assert_eq!(plan.items[0].selection_reason, "install-origin-match");
}

#[test]
fn update_plan_uses_alternate_channel_after_preferred_failure() {
    let apps = [AppRecord {
        stable_id: "t3code".to_owned(),
        display_name: "T3 Code".to_owned(),
        source_input: Some("pingdotgg/t3code".to_owned()),
        source: None,
        installed_version: Some("0.0.11".to_owned()),
        update_strategy: Some(UpdateStrategy {
            preferred: ChannelPreference {
                kind: UpdateChannelKind::GitHubReleases,
                locator: "fail://github".to_owned(),
                reason: "install-origin-match".to_owned(),
            },
            alternates: vec![ChannelPreference {
                kind: UpdateChannelKind::ElectronBuilder,
                locator: "https://example.test/latest-linux.yml".to_owned(),
                reason: "metadata-guided".to_owned(),
            }],
        }),
        metadata: Vec::new(),
        install: None,
    }];

    let plan = build_update_plan(&apps).unwrap();

    assert_eq!(
        plan.items[0].selected_channel.kind.as_str(),
        "electron-builder"
    );
    assert_eq!(plan.items[0].selection_reason, "preferred-channel-failed");
}
