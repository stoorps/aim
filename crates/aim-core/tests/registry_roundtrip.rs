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
            install: None,
        }],
    };

    store.save(&registry).unwrap();
    let loaded = store.load().unwrap();

    let strategy = loaded.apps[0].update_strategy.as_ref().unwrap();
    assert_eq!(strategy.preferred.reason, "install-origin-match");
    assert_eq!(strategy.alternates.len(), 2);
}

#[test]
fn registry_round_trips_install_metadata() {
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
            update_strategy: None,
            metadata: Vec::new(),
            install: Some(aim_core::domain::app::InstallMetadata {
                scope: aim_core::domain::app::InstallScope::User,
                payload_path: Some(
                    "/tmp/install-home/.local/lib/aim/appimages/t3code.AppImage".to_owned(),
                ),
                desktop_entry_path: Some(
                    "/tmp/install-home/.local/share/applications/aim-t3code.desktop".to_owned(),
                ),
                icon_path: Some(
                    "/tmp/install-home/.local/share/icons/hicolor/256x256/apps/t3code.png"
                        .to_owned(),
                ),
            }),
        }],
    };

    store.save(&registry).unwrap();
    let loaded = store.load().unwrap();

    let install = loaded.apps[0].install.as_ref().unwrap();
    assert_eq!(install.scope, aim_core::domain::app::InstallScope::User);
    assert_eq!(
        install.payload_path.as_deref(),
        Some("/tmp/install-home/.local/lib/aim/appimages/t3code.AppImage")
    );
    assert_eq!(
        install.desktop_entry_path.as_deref(),
        Some("/tmp/install-home/.local/share/applications/aim-t3code.desktop")
    );
    assert_eq!(
        install.icon_path.as_deref(),
        Some("/tmp/install-home/.local/share/icons/hicolor/256x256/apps/t3code.png")
    );
}
