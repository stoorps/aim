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
fn query_command_registers_unambiguous_app_in_registry_file() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.arg("sharkdp/bat")
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("tracked app: bat (sharkdp-bat)"));

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
        .stdout(contains("tracked app: t3code (pingdotgg-t3code)"));

    let contents = std::fs::read_to_string(&registry_path).unwrap();
    assert!(contents.contains("stable_id = \"pingdotgg-t3code\""));
    assert!(contents.contains("locator = \"pingdotgg/t3code\""));
}
