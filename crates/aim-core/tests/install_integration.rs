use aim_core::app::add::{build_add_plan_with_reporter, install_app_with_reporter};
use aim_core::app::progress::{OperationEvent, OperationStage};
use aim_core::domain::app::InstallScope;
use aim_core::domain::source::{NormalizedSourceKind, SourceKind};
use aim_core::integration::install::{DesktopIntegrationRequest, InstallRequest, execute_install};
use aim_core::platform::DesktopHelpers;
use aim_core::source::github::FixtureGitHubTransport;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;

#[test]
fn install_writes_desktop_entry_and_reports_refresh_warning_only() {
    let root = tempdir().unwrap();
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");

    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();

    let outcome = execute_install(&InstallRequest {
        staging_root: &staging_root,
        final_payload_path: &payload_root.join("bat.AppImage"),
        artifact_bytes: b"\x7fELFAppImage",
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
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");
    let helper_root = root.path().join("helpers");
    let log_path = root.path().join("helpers.log");

    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();
    fs::create_dir(&helper_root).unwrap();

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
        staging_root: &staging_root,
        final_payload_path: &payload_root.join("bat.AppImage"),
        artifact_bytes: b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82",
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
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    let desktop_root = root.path().join("applications");
    let icon_root = root.path().join("icons/hicolor/256x256/apps");

    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();
    fs::create_dir(&desktop_root).unwrap();
    fs::create_dir_all(&icon_root).unwrap();

    let outcome = execute_install(&InstallRequest {
        staging_root: &staging_root,
        final_payload_path: &payload_root.join("bat.AppImage"),
        artifact_bytes: b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82",
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
    let mut events: Vec<OperationEvent> = Vec::new();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let plan = build_add_plan_with_reporter("sharkdp/bat", &FixtureGitHubTransport, &mut reporter)
        .unwrap();

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
        stage: OperationStage::ResolveQuery,
        message: "resolving source".to_owned(),
    }));
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::DiscoverRelease,
        message: "discovering release".to_owned(),
    }));
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::SelectArtifact,
        message: "selecting artifact".to_owned(),
    }));
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

    let stage_order = events
        .iter()
        .filter_map(|event| match event {
            OperationEvent::StageChanged { stage, .. } => Some(*stage),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(stage_order.windows(2).any(|window| {
        window
            == [
                OperationStage::ResolveQuery,
                OperationStage::DiscoverRelease,
            ]
    }));
    assert!(stage_order.windows(2).any(|window| {
        window
            == [
                OperationStage::DiscoverRelease,
                OperationStage::SelectArtifact,
            ]
    }));
    assert!(stage_order.windows(2).any(|window| {
        window
            == [
                OperationStage::SelectArtifact,
                OperationStage::DownloadArtifact,
            ]
    }));
}

#[test]
fn gitlab_source_builds_concrete_install_candidate() {
    let mut events: Vec<OperationEvent> = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let plan = build_add_plan_with_reporter(
        "https://gitlab.com/example/team-app",
        &FixtureGitHubTransport,
        &mut reporter,
    )
    .unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::GitLab);
    assert_eq!(
        plan.resolution.source.locator,
        "https://gitlab.com/example/team-app"
    );
    assert_eq!(plan.resolution.release.version, "latest");
    assert_eq!(
        plan.selected_artifact.url,
        "https://gitlab.com/example/team-app/-/releases/permalink/latest/downloads/team-app.AppImage"
    );
    assert_eq!(plan.selected_artifact.version, "latest");
    assert_eq!(plan.selected_artifact.selection_reason, "provider-release");
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::DiscoverRelease,
        message: "discovering release".to_owned(),
    }));
}

#[test]
fn gitlab_candidate_builds_concrete_install_candidate() {
    let mut events: Vec<OperationEvent> = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let query = "https://gitlab.com/acme/platform/releases/team-app";
    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::GitLab);
    assert_eq!(plan.resolution.source.locator, query);
    assert_eq!(
        plan.resolution.source.canonical_locator.as_deref(),
        Some("acme/platform/releases/team-app")
    );
    assert_eq!(
        plan.resolution.source.normalized_kind,
        NormalizedSourceKind::GitLab
    );
    assert_eq!(plan.resolution.release.version, "latest");
    assert_eq!(
        plan.selected_artifact.url,
        "https://gitlab.com/acme/platform/releases/team-app/-/releases/permalink/latest/downloads/team-app.AppImage"
    );
    assert_eq!(plan.selected_artifact.version, "latest");
    assert_eq!(plan.selected_artifact.selection_reason, "provider-release");
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::DiscoverRelease,
        message: "discovering release".to_owned(),
    }));
}

#[test]
fn gitlab_install_preserves_truthful_gitlab_origin() {
    let root = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let mut reporter = |_event: &OperationEvent| {};
    let query = "https://gitlab.com/example/team-app";
    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    let installed =
        install_app_with_reporter(query, &plan, root.path(), InstallScope::User, &mut reporter)
            .unwrap();

    assert_eq!(installed.record.source_input.as_deref(), Some(query));
    assert_eq!(
        installed.record.installed_version.as_deref(),
        Some("latest")
    );
    assert_eq!(installed.source.kind, SourceKind::GitLab);
    assert_eq!(installed.source.locator, query);
    assert_eq!(
        installed.source.canonical_locator.as_deref(),
        Some("example/team-app")
    );
    assert_eq!(
        installed.selected_artifact.url,
        "https://gitlab.com/example/team-app/-/releases/permalink/latest/downloads/team-app.AppImage"
    );
}

#[test]
fn direct_url_source_uses_exact_input_resolution() {
    let mut reporter = |_event: &OperationEvent| {};
    let query = "https://example.com/downloads/team-app.AppImage";

    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::DirectUrl);
    assert_eq!(plan.resolution.source.locator, query);
    assert_eq!(plan.resolution.release.version, "unresolved");
    assert_eq!(plan.selected_artifact.url, query);
    assert_eq!(plan.selected_artifact.version, "unresolved");
    assert_eq!(plan.selected_artifact.selection_reason, "exact-input");
    assert_eq!(plan.update_strategy.preferred.locator, query);
    assert_eq!(plan.update_strategy.preferred.reason, "exact-input");
}

#[test]
fn direct_url_install_preserves_truthful_direct_url_origin() {
    let root = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let mut reporter = |_event: &OperationEvent| {};
    let query = "https://sourceforge.net/projects/team-app/files/team-app-1.0.0.AppImage/download";
    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    let installed =
        install_app_with_reporter(query, &plan, root.path(), InstallScope::User, &mut reporter)
            .unwrap();

    assert_eq!(installed.record.source_input.as_deref(), Some(query));
    assert_eq!(
        installed.record.installed_version.as_deref(),
        Some("unresolved")
    );
    assert_eq!(installed.source.kind, SourceKind::DirectUrl);
    assert_eq!(installed.source.locator, query);
    assert_eq!(installed.selected_artifact.url, query);
}

#[test]
fn sourceforge_candidate_builds_concrete_install_candidate() {
    let mut events: Vec<OperationEvent> = Vec::new();
    let mut reporter = |event: &OperationEvent| events.push(event.clone());

    let query = "https://sourceforge.net/projects/team-app/files/releases/stable/download";
    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::SourceForge);
    assert_eq!(plan.resolution.source.locator, query);
    assert_eq!(plan.resolution.release.version, "latest");
    assert_eq!(plan.selected_artifact.url, query);
    assert_eq!(plan.selected_artifact.version, "latest");
    assert_eq!(plan.selected_artifact.selection_reason, "provider-release");
    assert_eq!(plan.update_strategy.preferred.locator, query);
    assert_eq!(plan.update_strategy.preferred.reason, "provider-release");
    assert!(events.contains(&OperationEvent::StageChanged {
        stage: OperationStage::DiscoverRelease,
        message: "discovering release".to_owned(),
    }));
}

#[test]
fn sourceforge_latest_download_builds_concrete_install_candidate() {
    let mut reporter = |_event: &OperationEvent| {};
    let query = "https://sourceforge.net/projects/team-app/files/latest/download";

    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    assert_eq!(plan.resolution.source.kind, SourceKind::SourceForge);
    assert_eq!(plan.resolution.source.locator, query);
    assert_eq!(plan.resolution.release.version, "latest");
    assert_eq!(plan.selected_artifact.url, query);
    assert_eq!(plan.selected_artifact.version, "latest");
    assert_eq!(plan.selected_artifact.selection_reason, "provider-release");
}

#[test]
fn sourceforge_latest_download_install_preserves_truthful_origin() {
    let root = tempdir().unwrap();

    unsafe {
        std::env::set_var("AIM_GITHUB_FIXTURE_MODE", "1");
    }

    let mut reporter = |_event: &OperationEvent| {};
    let query = "https://sourceforge.net/projects/team-app/files/latest/download";
    let plan = build_add_plan_with_reporter(query, &FixtureGitHubTransport, &mut reporter).unwrap();

    let installed =
        install_app_with_reporter(query, &plan, root.path(), InstallScope::User, &mut reporter)
            .unwrap();

    assert_eq!(installed.record.source_input.as_deref(), Some(query));
    assert_eq!(
        installed.record.installed_version.as_deref(),
        Some("latest")
    );
    assert_eq!(installed.source.kind, SourceKind::SourceForge);
    assert_eq!(installed.source.locator, query);
    assert_eq!(
        installed.source.canonical_locator.as_deref(),
        Some("team-app")
    );
    assert_eq!(installed.selected_artifact.url, query);
}
