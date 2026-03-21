use aim_core::app::add::{build_add_plan_with, install_app_with_reporter};
use aim_core::app::progress::{OperationEvent, OperationStage};
use aim_core::domain::app::InstallScope;
use aim_core::integration::install::{DesktopIntegrationRequest, InstallRequest, execute_install};
use aim_core::platform::DesktopHelpers;
use aim_core::source::github::FixtureGitHubTransport;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

fn write_staged_payload(root: &std::path::Path, name: &str, bytes: &[u8]) -> std::path::PathBuf {
    let staged_path = root.join("staging").join(format!("{name}.download"));
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&staged_path, bytes).unwrap();
    staged_path
}

#[test]
fn install_writes_desktop_entry_and_reports_refresh_warning_only() {
    let root = tempdir().unwrap();
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");

    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();
    let staged_path = write_staged_payload(root.path(), "bat", b"\x7fELFAppImage");

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &payload_root.join("bat.AppImage"),
        trusted_checksum: None,
        desktop: Some(DesktopIntegrationRequest {
            desktop_entry_path: &desktop_root.join("aim-bat.desktop"),
            desktop_entry_contents: "[Desktop Entry]\nName=bat\nExec=bat.AppImage\nType=Application\n",
            icon_path: None,
            icon_bytes: None,
        }),
        helpers: DesktopHelpers::default(),
    })
    .unwrap();

    assert!(outcome.desktop_entry_path.unwrap().exists());
    assert!(!outcome.warnings.is_empty());
}

#[test]
fn install_executes_refresh_helpers_when_available() {
    let root = tempdir().unwrap();
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");
    let helper_root = root.path().join("helpers");
    let log_path = root.path().join("helpers.log");

    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();
    fs::create_dir(&helper_root).unwrap();
    let staged_path = write_staged_payload(
        root.path(),
        "bat",
        b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82",
    );

    let update_helper = helper_root.join("update-desktop-database");
    let icon_helper = helper_root.join("gtk-update-icon-cache");
    fs::write(
        &update_helper,
        format!("#!/bin/sh\necho desktop:$1 >> {}\n", log_path.display()),
    )
    .unwrap();
    fs::write(
        &icon_helper,
        format!("#!/bin/sh\necho icon:$3 >> {}\n", log_path.display()),
    )
    .unwrap();
    fs::set_permissions(&update_helper, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&icon_helper, fs::Permissions::from_mode(0o755)).unwrap();

    let icon_root = root.path().join("icons/hicolor/256x256/apps");
    fs::create_dir_all(&icon_root).unwrap();

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &payload_root.join("bat.AppImage"),
        trusted_checksum: None,
        desktop: Some(DesktopIntegrationRequest {
            desktop_entry_path: &desktop_root.join("aim-bat.desktop"),
            desktop_entry_contents: "[Desktop Entry]\nName=bat\nExec=bat.AppImage\nType=Application\n",
            icon_path: Some(&icon_root.join("bat.png")),
            icon_bytes: None,
        }),
        helpers: DesktopHelpers {
            update_desktop_database: true,
            gtk_update_icon_cache: true,
            update_desktop_database_path: Some(update_helper),
            gtk_update_icon_cache_path: Some(icon_helper),
        },
    })
    .unwrap();

    assert!(outcome.warnings.is_empty());
    let log = fs::read_to_string(&log_path).unwrap();
    assert!(log.contains("desktop:"));
    assert!(log.contains("icon:"));
}

#[test]
fn install_extracts_icon_from_appimage_payload_when_icon_path_is_requested() {
    let root = tempdir().unwrap();
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");
    let icon_root = root.path().join("icons/hicolor/256x256/apps");

    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();
    fs::create_dir_all(&icon_root).unwrap();
    let staged_path = write_staged_payload(
        root.path(),
        "bat",
        b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82",
    );

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &payload_root.join("bat.AppImage"),
        trusted_checksum: None,
        desktop: Some(DesktopIntegrationRequest {
            desktop_entry_path: &desktop_root.join("aim-bat.desktop"),
            desktop_entry_contents: "[Desktop Entry]\nName=bat\nExec=bat.AppImage\nType=Application\n",
            icon_path: Some(&icon_root.join("bat.png")),
            icon_bytes: None,
        }),
        helpers: DesktopHelpers::default(),
    })
    .unwrap();

    let icon_path = outcome.icon_path.unwrap();
    assert!(icon_path.exists());
    assert!(
        fs::read(&icon_path)
            .unwrap()
            .starts_with(b"\x89PNG\r\n\x1a\n")
    );
}

#[test]
fn install_app_reports_operation_stages_in_order() {
    let root = tempdir().unwrap();
    let plan = build_add_plan_with("sharkdp/bat", &FixtureGitHubTransport).unwrap();
    let mut events: Vec<OperationEvent> = Vec::new();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let installed = install_app_with_reporter(
        "sharkdp/bat",
        &plan,
        root.path(),
        InstallScope::User,
        &mut reporter,
    )
    .unwrap();

    assert_eq!(installed.record.stable_id, "sharkdp-bat");
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::DownloadArtifact,
        message: "downloading artifact".to_owned(),
    }));
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::StagePayload,
        message: "staging payload".to_owned(),
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OperationEvent::Progress {
                current,
                total: Some(total)
            } if *current == *total
        )
    }));
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::WriteDesktopEntry,
        message: "writing desktop entry".to_owned(),
    }));
    assert!(events.iter().any(|event| {
        matches!(
            event,
            OperationEvent::StageChanged {
                stage: OperationStage::RefreshIntegration,
                ..
            }
        )
    }));
}
