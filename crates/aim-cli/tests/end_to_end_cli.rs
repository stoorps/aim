use aim_core::domain::app::{AppRecord, InstallMetadata, InstallScope};
use aim_core::registry::model::Registry;
use aim_core::registry::store::RegistryStore;
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::tempdir;

const FIXTURE_MODE_ENV: &str = "AIM_GITHUB_FIXTURE_MODE";

#[test]
fn list_command_runs_without_registry_entries() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("list")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout(contains("No installed apps yet"));
}

#[test]
fn list_command_reads_registered_apps_from_registry_file() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    std::fs::write(
        &registry_path,
        "version = 1\n[[apps]]\nstable_id = \"bat\"\ndisplay_name = \"Bat\"\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("list")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout(contains("Name"))
        .stdout(contains("Version"))
        .stdout(contains("Source"))
        .stdout(contains("Bat"))
        .stdout(contains("Bat (bat)").not());
}

#[test]
fn remove_command_removes_registered_app_from_registry_file() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    std::fs::write(
        &registry_path,
        "version = 1\n[[apps]]\nstable_id = \"bat\"\ndisplay_name = \"Bat\"\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["remove", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout(contains("Removed Bat"))
        .stdout(contains("Removal Summary").not())
        .stdout(contains("Removed app:").not());

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(!contents.contains("stable_id = \"bat\""));
}

#[test]
fn remove_command_uninstalls_managed_files() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let install_home = dir.path().join("install-home");
    let payload_path = install_home.join(".local/lib/aim/appimages/sharkdp-bat.AppImage");
    let desktop_path = install_home.join(".local/share/applications/aim-sharkdp-bat.desktop");
    let icon_path = install_home.join(".local/share/icons/hicolor/256x256/apps/sharkdp-bat.png");

    let mut add_cmd = Command::cargo_bin("aim").unwrap();
    add_cmd
        .arg("sharkdp/bat")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success();

    assert!(payload_path.exists());
    assert!(desktop_path.exists());
    assert!(icon_path.exists());

    let mut remove_cmd = Command::cargo_bin("aim").unwrap();
    remove_cmd
        .args(["remove", "sharkdp-bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout(contains("\nRemoved bat"))
        .stdout(contains("Removed bat"))
        .stdout(contains("Removal Summary").not())
        .stdout(contains("Removed app:").not())
        .stdout(contains("Removed files"))
        .stdout(contains("sharkdp-bat.AppImage"))
        .stdout(contains("aim-sharkdp-bat.desktop"))
        .stdout(contains("sharkdp-bat.png"));

    assert!(!payload_path.exists());
    assert!(!desktop_path.exists());
    assert!(!icon_path.exists());
}

#[test]
fn query_command_registers_unambiguous_app_in_registry_file() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("sharkdp/bat")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("\nInstalled bat (user)"))
        .stdout(contains("Installed bat (user)"))
        .stdout(contains("Installation Summary").not())
        .stdout(contains("Source: github sharkdp/bat"))
        .stdout(contains("Artifact:"))
        .stdout(contains("Selected artifact").not())
        .stdout(contains("metadata-guided").not())
        .stdout(contains("Installed files"))
        .stdout(contains("sharkdp-bat.AppImage"))
        .stdout(contains("Completed steps").not());

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains("stable_id = \"sharkdp-bat\""));
    assert!(contents.contains("source_input = \"sharkdp/bat\""));
}

#[test]
fn old_release_query_renders_tracking_prompt_without_writing_registry() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Choose update tracking"))
        .stdout(contains("v0.0.11"))
        .stdout(contains("v0.0.12"));

    assert!(!registry_path.exists());
}

#[test]
fn old_release_query_can_track_latest_and_register_app() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("https://github.com/pingdotgg/t3code/releases/download/v0.0.11/T3-Code-0.0.11-x86_64.AppImage")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .env("AIM_TRACKING_PREFERENCE", "latest")
        .assert()
        .success()
        .stdout(contains("\nInstalled t3code (user)"))
        .stdout(contains("Installed t3code (user)"))
        .stdout(contains("Installation Summary").not())
        .stdout(contains("Source: github pingdotgg/t3code"))
        .stdout(contains("Artifact: T3-Code-0.0.12-x86_64.AppImage"))
        .stdout(contains("Selected artifact").not())
        .stdout(contains("metadata-guided").not())
        .stdout(contains("Installed files"))
        .stdout(contains("pingdotgg-t3code.AppImage"))
        .stdout(contains("Completed steps").not());

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains("stable_id = \"pingdotgg-t3code\""));
    assert!(contents.contains("locator = \"pingdotgg/t3code\""));
}

#[test]
fn cli_add_installs_and_renders_resolved_mode() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("sharkdp/bat")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("\nInstalled bat (user)"))
        .stdout(contains("Installed bat (user)"))
        .stdout(contains("Artifact:"))
        .stdout(contains("Installed files"))
        .stdout(contains("Completed steps").not());
}

#[test]
fn cli_add_installs_gitlab_source_with_truthful_origin() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("https://gitlab.com/example/team-app")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed team-app (user)"))
        .stdout(contains("Source: gitlab https://gitlab.com/example/team-app"))
        .stdout(contains(
            "Artifact: https://gitlab.com/example/team-app/-/releases/permalink/latest/downloads/team-app.AppImage",
        ));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains("source_input = \"https://gitlab.com/example/team-app\""));
    assert!(contents.contains("kind = \"GitLab\""));
    assert!(contents.contains("locator = \"https://gitlab.com/example/team-app\""));
    assert!(contents.contains("canonical_locator = \"example/team-app\""));
}

#[test]
fn cli_add_preserves_direct_url_origin_for_provider_like_downloads() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let query = "https://sourceforge.net/projects/team-app/files/team-app-1.0.0.AppImage/download";
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg(query)
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed "))
        .stdout(contains(format!("Source: direct-url {query}")))
        .stdout(contains(format!("Artifact: {query}")));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains(&format!("source_input = \"{query}\"")));
    assert!(contents.contains("kind = \"DirectUrl\""));
    assert!(contents.contains(&format!("locator = \"{query}\"")));
    assert!(!contents.contains("kind = \"SourceForge\""));
}

#[test]
fn cli_add_installs_sourceforge_latest_download_with_truthful_origin() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let query = "https://sourceforge.net/projects/team-app/files/latest/download";
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg(query)
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed team-app (user)"))
        .stdout(contains(format!("Source: sourceforge {query}")))
        .stdout(contains(format!("Artifact: {query}")));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains(&format!("source_input = \"{query}\"")));
    assert!(contents.contains("kind = \"SourceForge\""));
    assert!(contents.contains(&format!("locator = \"{query}\"")));
    assert!(contents.contains("canonical_locator = \"team-app\""));
}

#[test]
fn cli_add_installs_sourceforge_release_folder_with_truthful_origin() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let query = "https://sourceforge.net/projects/team-app/files/releases/beta/download";
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg(query)
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed team-app (user)"))
        .stdout(contains(format!("Source: sourceforge {query}")))
        .stdout(contains(format!("Artifact: {query}")));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains(&format!("source_input = \"{query}\"")));
    assert!(contents.contains("kind = \"SourceForge\""));
    assert!(contents.contains(&format!("locator = \"{query}\"")));
    assert!(contents.contains("canonical_locator = \"team-app\""));
}

#[test]
fn cli_add_file_like_sourceforge_release_download_stores_releases_root_and_preserves_artifact() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let query =
        "https://sourceforge.net/projects/team-app/files/releases/team-app-1.0.0.AppImage/download";
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg(query)
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed team-app (user)"))
        .stdout(contains(
            "Source: sourceforge https://sourceforge.net/projects/team-app/files/releases",
        ))
        .stdout(contains(format!("Artifact: {query}")));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains(&format!("source_input = \"{query}\"")));
    assert!(contents.contains("kind = \"SourceForge\""));
    assert!(
        contents.contains("locator = \"https://sourceforge.net/projects/team-app/files/releases\"")
    );
    assert!(contents.contains("requested_asset_name = \"team-app-1.0.0.AppImage\""));
}

#[test]
fn cli_reports_unsupported_source_queries_distinctly() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("https://gitlab.com/example")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .failure()
        .stderr(contains("unsupported source query"));
}

#[test]
fn cli_reports_supported_sources_without_installable_artifacts_distinctly() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("https://sourceforge.net/projects/team-app/")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .failure()
        .stderr(contains("no installable artifact found"))
        .stderr(contains("sourceforge"));
}

#[test]
fn cli_add_emits_live_progress_to_stderr() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("sharkdp/bat")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stderr(contains("Installing sharkdp/bat"))
        .stderr(contains("Resolving source"))
        .stderr(contains("Discovering release"))
        .stderr(contains("Selecting artifact"))
        .stderr(contains("Downloading artifact"))
        .stderr(contains("Downloaded"))
        .stderr(contains("Payload Staged"))
        .stderr(contains("Desktop Entry Written"))
        .stderr(contains("Icon Extracted"))
        .stderr(contains("Desktop Integration Refreshed"))
        .stderr(contains("Registry Saved"));
}

#[test]
fn bare_aim_review_renders_review_heading() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let store = RegistryStore::new(registry_path.clone());
    store
        .save(&Registry {
            version: 1,
            apps: vec![AppRecord {
                stable_id: "pingdotgg-t3code".to_owned(),
                display_name: "t3code".to_owned(),
                source_input: Some("pingdotgg/t3code".to_owned()),
                source: None,
                installed_version: Some("0.0.11".to_owned()),
                update_strategy: None,
                metadata: Vec::new(),
                install: Some(InstallMetadata {
                    scope: InstallScope::User,
                    payload_path: None,
                    desktop_entry_path: None,
                    icon_path: None,
                }),
            }],
        })
        .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stdout(contains("Update Review"))
        .stdout(contains("apps with updates"));
}

#[test]
fn remove_command_emits_live_progress_to_stderr() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    std::fs::write(
        &registry_path,
        "version = 1\n[[apps]]\nstable_id = \"bat\"\ndisplay_name = \"Bat\"\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["remove", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .assert()
        .success()
        .stderr(contains("Removing bat"))
        .stderr(contains("Resolving source: resolving bat"))
        .stderr(contains("Saving registry"));
}

#[test]
fn system_request_on_immutable_host_falls_back_to_user_install() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let os_release_path = dir.path().join("os-release");
    std::fs::write(&os_release_path, "ID=fedora\nVARIANT_ID=silverblue\n").unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["--system", "sharkdp/bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env("AIM_OS_RELEASE_PATH", &os_release_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed bat (user)"))
        .stdout(contains("downgraded to user scope"));
}

#[test]
fn update_command_applies_updates() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let payload_path = dir
        .path()
        .join("install-home/.local/lib/aim/appimages/pingdotgg-t3code.AppImage");
    let store = RegistryStore::new(registry_path.clone());
    store
        .save(&Registry {
            version: 1,
            apps: vec![AppRecord {
                stable_id: "pingdotgg-t3code".to_owned(),
                display_name: "t3code".to_owned(),
                source_input: Some("pingdotgg/t3code".to_owned()),
                source: None,
                installed_version: Some("0.0.11".to_owned()),
                update_strategy: None,
                metadata: Vec::new(),
                install: Some(InstallMetadata {
                    scope: InstallScope::User,
                    payload_path: None,
                    desktop_entry_path: None,
                    icon_path: None,
                }),
            }],
        })
        .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("update")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("updated apps: 1"))
        .stdout(contains("updates found:").not());

    let updated = store.load().unwrap();
    assert_eq!(updated.apps.len(), 1);
    assert_eq!(updated.apps[0].stable_id, "pingdotgg-t3code");
    assert_eq!(updated.apps[0].installed_version.as_deref(), Some("0.0.12"));
    assert!(payload_path.exists());
}

#[test]
fn update_command_emits_live_progress_to_stderr() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let store = RegistryStore::new(registry_path.clone());
    store
        .save(&Registry {
            version: 1,
            apps: vec![AppRecord {
                stable_id: "pingdotgg-t3code".to_owned(),
                display_name: "t3code".to_owned(),
                source_input: Some("pingdotgg/t3code".to_owned()),
                source: None,
                installed_version: Some("0.0.11".to_owned()),
                update_strategy: None,
                metadata: Vec::new(),
                install: Some(InstallMetadata {
                    scope: InstallScope::User,
                    payload_path: None,
                    desktop_entry_path: None,
                    icon_path: None,
                }),
            }],
        })
        .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("update")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stderr(contains("Updating 1 apps"))
        .stderr(contains("Resolving source: resolving pingdotgg-t3code"))
        .stderr(contains("Saving registry"));
}

#[test]
fn update_command_reports_when_previous_installation_is_restored() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let install_home = dir.path().join("install-home");
    let store = RegistryStore::new(registry_path.clone());
    let stable_id = "url-example.com-downloads-team-app.appimage";
    let payload_path = install_home.join(format!(".local/lib/aim/appimages/{stable_id}.AppImage"));

    std::fs::create_dir_all(payload_path.parent().unwrap()).unwrap();
    std::fs::write(&payload_path, b"previous-payload").unwrap();
    std::fs::create_dir_all(install_home.join(".local/share")).unwrap();
    std::fs::write(install_home.join(".local/share/applications"), b"blocker").unwrap();

    store
        .save(&Registry {
            version: 1,
            apps: vec![AppRecord {
                stable_id: stable_id.to_owned(),
                display_name: "https://example.com/downloads/team-app.AppImage".to_owned(),
                source_input: Some("https://example.com/downloads/team-app.AppImage".to_owned()),
                source: Some(aim_core::domain::source::SourceRef {
                    kind: aim_core::domain::source::SourceKind::DirectUrl,
                    locator: "https://example.com/downloads/team-app.AppImage".to_owned(),
                    input_kind: aim_core::domain::source::SourceInputKind::DirectUrl,
                    normalized_kind: aim_core::domain::source::NormalizedSourceKind::DirectUrl,
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
            }],
        })
        .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("update")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .env("DISPLAY", ":99")
        .env("XDG_CURRENT_DESKTOP", "test")
        .assert()
        .success()
        .stdout(contains("Failed:"))
        .stdout(contains("restored previous installation"));

    assert_eq!(std::fs::read(&payload_path).unwrap(), b"previous-payload");
}
