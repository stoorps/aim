use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;

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
