use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

const FIXTURE_MODE_ENV: &str = "AIM_GITHUB_FIXTURE_MODE";

#[test]
fn list_command_runs_without_registry_entries() {
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("list")
        .assert()
        .success()
        .stdout(contains("installed"));
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
        .stdout(contains("Bat (bat)"));
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
        .stdout(contains("removed: Bat"));

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
        .stdout(contains("removed: bat"));

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
        .stdout(contains("installing as user"))
        .stdout(contains("installed app: bat (sharkdp-bat)"));

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
        .stdout(contains("tracking preference required"))
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
        .stdout(contains("installing as user"))
        .stdout(contains("installed app: t3code (pingdotgg-t3code)"));

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
        .stdout(contains("installing as user"))
        .stdout(contains("installed app:"));
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
        .stdout(contains("installing as user"))
        .stdout(contains("downgraded to user scope"));
}
