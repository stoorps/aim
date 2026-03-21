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

#[test]
fn registry_save_is_atomic_and_cleans_up_temp_file() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let store = RegistryStore::new(registry_path.clone());

    store
        .save(&aim_core::registry::model::Registry {
            version: 1,
            apps: vec![aim_core::domain::app::AppRecord {
                stable_id: "bat".to_owned(),
                display_name: "Bat".to_owned(),
                source_input: None,
                source: None,
                installed_version: None,
                update_strategy: None,
                metadata: Vec::new(),
                install: None,
            }],
        })
        .unwrap();

    assert!(registry_path.exists());
    assert!(!dir.path().join("registry.toml.tmp").exists());
}

#[test]
fn registry_exclusive_lock_rejects_second_mutator() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    let _guard = store.lock_exclusive().unwrap();

    let error = store.lock_exclusive().unwrap_err();

    assert!(matches!(
        error,
        aim_core::registry::store::RegistryStoreError::LockUnavailable
    ));
}

#[test]
fn registry_mutate_exclusive_reloads_and_writes_latest_state() {
    let dir = tempdir().unwrap();
    let store = RegistryStore::new(dir.path().join("registry.toml"));
    store
        .save(&aim_core::registry::model::Registry {
            version: 1,
            apps: vec![aim_core::domain::app::AppRecord {
                stable_id: "bat".to_owned(),
                display_name: "Bat".to_owned(),
                source_input: None,
                source: None,
                installed_version: None,
                update_strategy: None,
                metadata: Vec::new(),
                install: None,
            }],
        })
        .unwrap();

    store
        .mutate_exclusive(|registry| {
            registry.apps.push(aim_core::domain::app::AppRecord {
                stable_id: "t3code".to_owned(),
                display_name: "T3 Code".to_owned(),
                source_input: None,
                source: None,
                installed_version: None,
                update_strategy: None,
                metadata: Vec::new(),
                install: None,
            });
        })
        .unwrap();

    let loaded = store.load().unwrap();
    assert_eq!(loaded.apps.len(), 2);
    assert_eq!(loaded.apps[0].stable_id, "bat");
    assert_eq!(loaded.apps[1].stable_id, "t3code");
}
