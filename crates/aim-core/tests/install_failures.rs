use aim_core::app::add::{BuildAddPlanError, build_add_plan_with};
use aim_core::app::query::ResolveQueryError;
use aim_core::domain::source::SourceKind;
use aim_core::integration::install::{DesktopIntegrationRequest, InstallRequest, execute_install};
use aim_core::platform::DesktopHelpers;
use aim_core::source::github::FixtureGitHubTransport;
use std::fs;
use tempfile::tempdir;

#[test]
fn integration_failure_removes_new_payload_and_generated_files() {
    let root = tempdir().unwrap();
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    let blocking_path = root.path().join("not-a-directory");

    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();
    fs::write(&blocking_path, "blocker").unwrap();

    let final_payload_path = payload_root.join("bat.AppImage");
    let desktop_entry_path = blocking_path.join("aim-bat.desktop");
    let error = execute_install(&InstallRequest {
        staging_root: &staging_root,
        final_payload_path: &final_payload_path,
        artifact_bytes: b"\x7fELFAppImage",
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
