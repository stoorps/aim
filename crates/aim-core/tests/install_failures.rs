use aim_core::app::add::{BuildAddPlanError, build_add_plan_with};
use aim_core::app::query::ResolveQueryError;
use aim_core::app::update::execute_updates;
use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::domain::source::SourceKind;
use aim_core::domain::source::{NormalizedSourceKind, SourceInputKind, SourceRef};
use aim_core::integration::install::{DesktopIntegrationRequest, InstallRequest, execute_install};
use aim_core::platform::DesktopHelpers;
use aim_core::source::github::FixtureGitHubTransport;
use std::fs;
use std::sync::Mutex;
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn integration_failure_removes_new_payload_and_generated_files() {
    let root = tempdir().unwrap();
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    let blocking_path = root.path().join("not-a-directory");

    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();
    fs::write(&blocking_path, "blocker").unwrap();
    let staged_path = staging_root.join("bat.download");
    fs::write(&staged_path, b"\x7fELFAppImage").unwrap();

    let final_payload_path = payload_root.join("bat.AppImage");
    let desktop_entry_path = blocking_path.join("aim-bat.desktop");
    let error = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: None,
        weak_checksum_md5: None,
        desktop: Some(DesktopIntegrationRequest {
            desktop_entry_path: &desktop_entry_path,
            desktop_entry_contents: "[Desktop Entry]\nName=bat\nExec=bat.AppImage\nType=Application\n",
            icon_path: None,
            icon_bytes: None,
        }),
        helpers: DesktopHelpers::default(),
    })
    .unwrap_err();

    assert!(error.to_string().contains("desktop integration failed"));
    assert!(!final_payload_path.exists());
    assert!(!desktop_entry_path.exists());
}

#[test]
fn unsupported_queries_remain_distinct_from_provider_resolution_failures() {
    let error =
        build_add_plan_with("https://gitlab.com/example", &FixtureGitHubTransport).unwrap_err();

    assert!(matches!(
        error,
        BuildAddPlanError::Query(ResolveQueryError::Unsupported)
    ));
}

#[test]
fn supported_sourceforge_project_without_latest_download_reports_no_installable_artifact() {
    let error = build_add_plan_with(
        "https://sourceforge.net/projects/team-app/",
        &FixtureGitHubTransport,
    )
    .unwrap_err();

    match error {
        BuildAddPlanError::NoInstallableArtifact { source } => {
            assert_eq!(source.kind, SourceKind::SourceForge);
            assert_eq!(source.locator, "https://sourceforge.net/projects/team-app/");
            assert_eq!(source.canonical_locator.as_deref(), Some("team-app"));
        }
        other => panic!("expected no-installable-artifact error, got {other:?}"),
    }
}

#[test]
fn failed_update_restores_tracked_desktop_and_icon_files() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let root = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
        std::env::set_var("DISPLAY", ":99");
        std::env::set_var("XDG_CURRENT_DESKTOP", "test");
    }

    let payload_path = root.path().join("tracked/team-app.AppImage");
    let desktop_path = root.path().join("tracked/aim-team-app.desktop");
    let icon_path = root.path().join("tracked/team-app.png");
    fs::create_dir_all(payload_path.parent().unwrap()).unwrap();
    fs::write(&payload_path, b"previous-payload").unwrap();
    fs::write(&desktop_path, b"previous-desktop").unwrap();
    fs::write(&icon_path, b"previous-icon").unwrap();

    let blocking_applications_root = root.path().join(".local/share/applications");
    fs::create_dir_all(blocking_applications_root.parent().unwrap()).unwrap();
    fs::write(&blocking_applications_root, b"blocker").unwrap();

    let previous = AppRecord {
        stable_id: "url-example.com-downloads-team-app.appimage".to_owned(),
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
            desktop_entry_path: Some(desktop_path.display().to_string()),
            icon_path: Some(icon_path.display().to_string()),
        }),
    };

    let result = execute_updates(std::slice::from_ref(&previous), root.path()).unwrap();

    assert_eq!(result.failed_count(), 1);
    assert_eq!(fs::read(&payload_path).unwrap(), b"previous-payload");
    assert_eq!(fs::read(&desktop_path).unwrap(), b"previous-desktop");
    assert_eq!(fs::read(&icon_path).unwrap(), b"previous-icon");
}
