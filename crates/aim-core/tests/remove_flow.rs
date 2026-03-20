use aim_core::app::interaction::{InteractionKind, InteractionRequest};
use aim_core::app::list::build_list_rows;
use aim_core::app::progress::{OperationEvent, OperationStage};
use aim_core::app::remove::{
    build_removal_plan, remove_registered_app_with_reporter, resolve_registered_app,
};
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceKind, SourceRef};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn remove_flow_rejects_unknown_app_names() {
    let result = resolve_registered_app("bat", &[]);

    assert!(result.is_err());
}

#[test]
fn list_flow_returns_display_rows_for_registered_apps() {
    let rows = build_list_rows(&[AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: Some("sharkdp/bat".to_owned()),
        source: Some(SourceRef {
            kind: SourceKind::GitHub,
            input_kind: SourceInputKind::RepoShorthand,
            normalized_kind: NormalizedSourceKind::GitHubRepository,
            locator: "sharkdp/bat".to_owned(),
            canonical_locator: Some("sharkdp/bat".to_owned()),
            requested_tag: None,
            requested_asset_name: None,
            tracks_latest: true,
        }),
        installed_version: Some("0.25.0".to_owned()),
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    }]);

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].stable_id, "bat");
    assert_eq!(rows[0].display_name, "Bat");
    assert_eq!(rows[0].version.as_deref(), Some("0.25.0"));
    assert_eq!(rows[0].source, "sharkdp/bat");
}

#[test]
fn ambiguous_remove_matches_include_stable_ids_for_client_choice() {
    let apps = [
        AppRecord {
            stable_id: "bat".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
        AppRecord {
            stable_id: "bat-nightly".to_owned(),
            display_name: "Bat".to_owned(),
            source_input: None,
            source: None,
            installed_version: None,
            update_strategy: None,
            metadata: Vec::new(),
            install: None,
        },
    ];

    let error = resolve_registered_app("Bat", &apps).unwrap_err();

    assert_eq!(
        error,
        aim_core::app::remove::ResolveRegisteredAppError::Ambiguous {
            request: InteractionRequest {
                key: "select-registered-app".to_owned(),
                kind: InteractionKind::SelectRegisteredApp {
                    query: "Bat".to_owned(),
                    matches: vec!["Bat (bat)".to_owned(), "Bat (bat-nightly)".to_owned()],
                },
            },
        }
    );
}

#[test]
fn removal_plan_prefers_persisted_install_metadata_paths() {
    let app = AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: None,
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::System,
            payload_path: Some("/opt/aim/appimages/bat.AppImage".to_owned()),
            desktop_entry_path: Some("/usr/share/applications/aim-bat.desktop".to_owned()),
            icon_path: Some("/usr/share/icons/hicolor/256x256/apps/bat.png".to_owned()),
        }),
    };

    let plan = build_removal_plan(&app, Path::new("/home/test"));

    assert_eq!(plan.stable_id, "bat");
    assert_eq!(
        plan.artifact_paths,
        vec![
            "/opt/aim/appimages/bat.AppImage".to_owned(),
            "/usr/share/applications/aim-bat.desktop".to_owned(),
            "/usr/share/icons/hicolor/256x256/apps/bat.png".to_owned(),
        ]
    );
}

#[test]
fn removal_plan_falls_back_to_derived_managed_user_paths() {
    let app = AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: None,
        update_strategy: None,
        metadata: Vec::new(),
        install: None,
    };

    let plan = build_removal_plan(&app, Path::new("/home/test"));

    assert_eq!(
        plan.artifact_paths,
        vec![
            "/home/test/.local/lib/aim/appimages/bat.AppImage".to_owned(),
            "/home/test/.local/share/applications/aim-bat.desktop".to_owned(),
            "/home/test/.local/share/icons/hicolor/256x256/apps/bat.png".to_owned(),
        ]
    );
}

#[test]
fn remove_flow_reports_resolution_and_cleanup_events() {
    let install_home = tempdir().unwrap();
    let app = AppRecord {
        stable_id: "bat".to_owned(),
        display_name: "Bat".to_owned(),
        source_input: None,
        source: None,
        installed_version: None,
        update_strategy: None,
        metadata: Vec::new(),
        install: Some(InstallMetadata {
            scope: InstallScope::User,
            payload_path: Some(
                install_home
                    .path()
                    .join(".local/lib/aim/appimages/bat.AppImage")
                    .display()
                    .to_string(),
            ),
            desktop_entry_path: None,
            icon_path: None,
        }),
    };
    let mut events: Vec<OperationEvent> = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let result =
        remove_registered_app_with_reporter("bat", &[app], install_home.path(), &mut reporter)
            .unwrap();

    assert_eq!(result.removed.stable_id, "bat");
    assert_eq!(result.removed_paths.len(), 0);
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
            OperationEvent::StageChanged {
                stage: OperationStage::Finalize,
                ..
            }
        )
    }));
}
