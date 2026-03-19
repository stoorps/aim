use aim_core::registry::store::RegistryStore;
use tempfile::tempdir;

#[test]
fn registry_round_trips_app_records() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    let loaded = store.load().unwrap();
    assert!(loaded.apps.is_empty());
}

#[test]
fn registry_round_trips_update_strategy_and_alternates() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    let registry = aim_core::registry::model::Registry {
        version: 1,
        apps: vec![aim_core::domain::app::AppRecord {
            stable_id: "t3code".to_owned(),
            display_name: "T3 Code".to_owned(),
            source_input: Some("pingdotgg/t3code".to_owned()),
            source: None,
            installed_version: Some("0.0.11".to_owned()),
            update_strategy: Some(aim_core::domain::update::UpdateStrategy {
                preferred: aim_core::domain::update::ChannelPreference {
                    kind: aim_core::domain::update::UpdateChannelKind::DirectAsset,
                    locator: "https://example.test/app.AppImage".to_owned(),
                    reason: "install-origin-match".to_owned(),
                },
                alternates: vec![
                    aim_core::domain::update::ChannelPreference {
                        kind: aim_core::domain::update::UpdateChannelKind::GitHubReleases,
                        locator: "pingdotgg/t3code".to_owned(),
                        reason: "heuristic-match".to_owned(),
                    },
                    aim_core::domain::update::ChannelPreference {
                        kind: aim_core::domain::update::UpdateChannelKind::ElectronBuilder,
                        locator: "https://example.test/latest-linux.yml".to_owned(),
                        reason: "metadata-guided".to_owned(),
                    },
                ],
            }),
            metadata: Vec::new(),
        }],
    };

    store.save(&registry).unwrap();
    let loaded = store.load().unwrap();

    let strategy = loaded.apps[0].update_strategy.as_ref().unwrap();
    assert_eq!(strategy.preferred.reason, "install-origin-match");
    assert_eq!(strategy.alternates.len(), 2);
}
