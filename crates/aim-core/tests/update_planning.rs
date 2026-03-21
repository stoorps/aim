use aim_core::app::progress::{OperationEvent, OperationStage};
use aim_core::app::update::{build_update_plan, execute_updates, execute_updates_with_reporter};
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use aim_core::domain::update::{ChannelPreference, UpdateChannelKind, UpdateStrategy};
use aim_core::integration::paths::managed_appimage_path;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

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

#[test]
fn failed_update_keeps_previous_app_record() {
    let install_home = tempdir().unwrap();
    let previous = AppRecord {
        stable_id: "legacy-bat".to_owned(),
        display_name: "Legacy Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: Some("0.9.0".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: None,
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.apps, vec![previous]);
    assert_eq!(result.updated_count(), 0);
    assert_eq!(result.failed_count(), 1);
}

#[test]
fn update_execution_reports_per_app_lifecycle_events() {
    let install_home = tempdir().unwrap();
    let app = AppRecord {
        stable_id: "legacy-bat".to_owned(),
        display_name: "Legacy Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: Some("0.9.0".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: None,
            desktop_entry_path: None,
            icon_path: None,
        }),
    };
    let mut events: Vec<OperationEvent> = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let result = execute_updates_with_reporter(
        std::slice::from_ref(&app),
        install_home.path(),
        &mut reporter,
    )
    .unwrap();

    assert_eq!(result.failed_count(), 1);
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OperationEvent::StageChanged {
                stage: OperationStage::ResolveQuery,
                ..
            }
        )
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OperationEvent::Failed {
                stage: OperationStage::ResolveQuery,
                ..
            }
        )
    }));
}

#[test]
fn update_plan_uses_direct_asset_fallback_for_direct_url_origin() {
    let apps = [AppRecord {
        stable_id: "team-app".to_owned(),
        display_name: "team-app".to_owned(),
        source_input: Some("https://example.com/downloads/team-app.AppImage".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::DirectUrl,
            locator: "https://example.com/downloads/team-app.AppImage".to_owned(),
            input_kind: SourceInputKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        }),
        installed_version: Some("unresolved".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }];

    let plan = build_update_plan(&apps).unwrap();

    assert_eq!(
        plan.items[0].selected_channel.kind,
        UpdateChannelKind::DirectAsset
    );
    assert_eq!(
        plan.items[0].selected_channel.locator,
        "https://example.com/downloads/team-app.AppImage"
    );
    assert_eq!(plan.items[0].selection_reason, "install-origin-match");
}

#[test]
fn update_execution_rebuilds_gitlab_source_without_rewriting_origin() {
    let install_home = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let previous = AppRecord {
        stable_id: "example-team-app".to_owned(),
        display_name: "team-app".to_owned(),
        source_input: Some("https://gitlab.com/example/team-app".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::GitLab,
            locator: "https://gitlab.com/example/team-app".to_owned(),
            input_kind: SourceInputKind::GitLabUrl,
            normalized_kind: NormalizedSourceKind::GitLab,
            canonical_locator: Some("example/team-app".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("latest".to_owned()),
        update_strategy: Some(UpdateStrategy {
            preferred: ChannelPreference {
                kind: UpdateChannelKind::DirectAsset,
                locator: "https://gitlab.com/example/team-app/-/releases/permalink/latest/downloads/team-app.AppImage"
                    .to_owned(),
                reason: "provider-release".to_owned(),
            },
            alternates: Vec::new(),
        }),
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: None,
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.updated_count(), 1);
    assert_eq!(result.failed_count(), 0);
    assert_eq!(
        result.apps[0].source.as_ref().unwrap().kind,
        SourceKind::GitLab
    );
    assert_eq!(
        result.apps[0].source.as_ref().unwrap().locator,
        "https://gitlab.com/example/team-app"
    );
    assert_eq!(
        result.apps[0]
            .source
            .as_ref()
            .unwrap()
            .canonical_locator
            .as_deref(),
        Some("example/team-app")
    );
}

#[test]
fn update_execution_rebuilds_sourceforge_release_folder_without_rewriting_origin() {
    let install_home = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let previous = AppRecord {
        stable_id: "team-app".to_owned(),
        display_name: "team-app".to_owned(),
        source_input: Some(
            "https://sourceforge.net/projects/team-app/files/releases/beta/download".to_owned(),
        ),
        source: Some(SourceRef {
            kind: SourceKind::SourceForge,
            locator: "https://sourceforge.net/projects/team-app/files/releases/beta/download"
                .to_owned(),
            input_kind: SourceInputKind::SourceForgeUrl,
            normalized_kind: NormalizedSourceKind::SourceForge,
            canonical_locator: Some("team-app".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("latest".to_owned()),
        update_strategy: Some(UpdateStrategy {
            preferred: ChannelPreference {
                kind: UpdateChannelKind::DirectAsset,
                locator: "https://sourceforge.net/projects/team-app/files/releases/beta/download"
                    .to_owned(),
                reason: "provider-release".to_owned(),
            },
            alternates: Vec::new(),
        }),
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: None,
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.updated_count(), 1);
    assert_eq!(result.failed_count(), 0);
    assert_eq!(
        result.apps[0].source.as_ref().unwrap().kind,
        SourceKind::SourceForge
    );
    assert_eq!(
        result.apps[0].source.as_ref().unwrap().locator,
        "https://sourceforge.net/projects/team-app/files/releases/beta/download"
    );
    assert_eq!(
        result.apps[0]
            .source
            .as_ref()
            .unwrap()
            .canonical_locator
            .as_deref(),
        Some("team-app")
    );
}

#[test]
fn update_execution_uses_stored_sourceforge_releases_root_for_file_like_inputs() {
    let install_home = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let previous = AppRecord {
        stable_id: "team-app".to_owned(),
        display_name: "team-app".to_owned(),
        source_input: Some(
            "https://sourceforge.net/projects/team-app/files/releases/team-app-1.0.0.AppImage/download"
                .to_owned(),
        ),
        source: Some(SourceRef {
            kind: SourceKind::SourceForge,
            locator: "https://sourceforge.net/projects/team-app/files/releases".to_owned(),
            input_kind: SourceInputKind::SourceForgeUrl,
            normalized_kind: NormalizedSourceKind::SourceForge,
            canonical_locator: Some("team-app".to_owned()),
            requested_tag: None,
            requested_asset_name: Some("team-app-1.0.0.AppImage".to_owned()),
            tracks_latest: true,
        }),
        installed_version: Some("latest".to_owned()),
        update_strategy: Some(UpdateStrategy {
            preferred: ChannelPreference {
                kind: UpdateChannelKind::DirectAsset,
                locator: "https://sourceforge.net/projects/team-app/files/releases".to_owned(),
                reason: "provider-release".to_owned(),
            },
            alternates: Vec::new(),
        }),
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: None,
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.updated_count(), 1);
    assert_eq!(result.failed_count(), 0);
    assert_eq!(
        result.apps[0].source.as_ref().unwrap().locator,
        "https://sourceforge.net/projects/team-app/files/releases"
    );
    assert_eq!(
        result.apps[0]
            .source
            .as_ref()
            .unwrap()
            .requested_asset_name
            .as_deref(),
        None
    );
}

#[test]
fn failed_update_restores_previous_payload_contents() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let install_home = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
        std::env::set_var("DISPLAY", ":99");
        std::env::set_var("XDG_CURRENT_DESKTOP", "test");
    }

    let stable_id = "url-example.com-downloads-team-app.appimage";
    let payload_path = managed_appimage_path(install_home.path(), InstallScope::User, stable_id);
    fs::create_dir_all(payload_path.parent().unwrap()).unwrap();
    fs::write(&payload_path, b"previous-payload").unwrap();

    let desktop_root = install_home.path().join(".local/share/applications");
    fs::create_dir_all(desktop_root.parent().unwrap()).unwrap();
    fs::write(&desktop_root, b"blocker").unwrap();

    let previous = AppRecord {
        stable_id: stable_id.to_owned(),
        display_name: "https://example.com/downloads/team-app.AppImage".to_owned(),
        source_input: Some("https://example.com/downloads/team-app.AppImage".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::DirectUrl,
            locator: "https://example.com/downloads/team-app.AppImage".to_owned(),
            input_kind: SourceInputKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        }),
        installed_version: Some("unresolved".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: Some(payload_path.display().to_string()),
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.failed_count(), 1);
    assert_eq!(result.apps, vec![previous]);
    assert_eq!(fs::read(&payload_path).unwrap(), b"previous-payload");
}

#[test]
fn successful_update_removes_rollback_staging_directory() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let install_home = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
        std::env::remove_var("DISPLAY");
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::remove_var("XDG_CURRENT_DESKTOP");
    }

    let stable_id = "url-example.com-downloads-team-app.appimage";
    let payload_path = managed_appimage_path(install_home.path(), InstallScope::User, stable_id);
    fs::create_dir_all(payload_path.parent().unwrap()).unwrap();
    fs::write(&payload_path, b"previous-payload").unwrap();

    let previous = AppRecord {
        stable_id: stable_id.to_owned(),
        display_name: "https://example.com/downloads/team-app.AppImage".to_owned(),
        source_input: Some("https://example.com/downloads/team-app.AppImage".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::DirectUrl,
            locator: "https://example.com/downloads/team-app.AppImage".to_owned(),
            input_kind: SourceInputKind::DirectUrl,
            normalized_kind: NormalizedSourceKind::DirectUrl,
            canonical_locator: None,
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: false,
        }),
        installed_version: Some("unresolved".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: Some(payload_path.display().to_string()),
            desktop_entry_path: None,
            icon_path: None,
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), install_home.path()).unwrap();

    assert_eq!(result.updated_count(), 1);
    assert!(
        !install_home
            .path()
            .join(".local/share/aim/rollback")
            .exists()
    );
}
