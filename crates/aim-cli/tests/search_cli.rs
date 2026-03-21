use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use tempfile::tempdir;

const FIXTURE_MODE_ENV: &str = "AIM_GITHUB_FIXTURE_MODE";

#[test]
fn search_command_renders_remote_github_results() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Search Results"))
        .stdout(contains("Remote Results"))
        .stdout(contains("[github] sharkdp/bat"))
        .stdout(contains("Install query: sharkdp/bat"));
}

#[test]
fn search_command_renders_local_matches_in_deterministic_order() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    std::fs::write(
        &registry_path,
        concat!(
            "version = 1\n",
            "[[apps]]\n",
            "stable_id = \"bat\"\n",
            "display_name = \"Bat\"\n",
            "[[apps]]\n",
            "stable_id = \"bat-tools\"\n",
            "display_name = \"Bat Tools\"\n",
            "[[apps]]\n",
            "stable_id = \"acrobat-reader\"\n",
            "display_name = \"Acrobat Reader\"\n",
            "[[apps]]\n",
            "stable_id = \"combat-viewer\"\n",
            "display_name = \"Combat Viewer\"\n"
        ),
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Installed Matches"))
        .stdout(
            contains("- Bat (bat)")
                .and(contains("- Bat Tools (bat-tools)"))
                .and(contains("- Acrobat Reader (acrobat-reader)"))
                .and(contains("- Combat Viewer (combat-viewer)")),
        );
}

#[test]
fn search_command_is_read_only_for_registry_contents() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let original = "version = 1\n[[apps]]\nstable_id = \"bat\"\ndisplay_name = \"Bat\"\n";
    std::fs::write(&registry_path, original).unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success();

    let persisted = std::fs::read_to_string(&registry_path).unwrap();
    assert_eq!(persisted, original);
}

#[test]
fn search_command_fails_fast_on_malformed_config() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let config_path = dir.path().join("config.toml");
    std::fs::write(&config_path, "[search\nskip_confirmation = true\n").unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env("AIM_CONFIG_PATH", &config_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .failure()
        .stderr(contains(config_path.to_string_lossy().as_ref()));
}

#[test]
fn search_command_uses_plain_text_output_when_not_on_a_tty() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let config_path = dir.path().join("config.toml");
    std::fs::write(
        &config_path,
        "[search]\nbottom_to_top = false\nskip_confirmation = true\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env("AIM_CONFIG_PATH", &config_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Search Results"))
        .stdout(contains("Remote Results"))
        .stdout(contains("[github] sharkdp/bat"));
}

#[test]
fn search_command_reports_loading_status_to_stderr() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "bat"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stderr(contains("Searching bat"));
}

#[test]
fn search_command_keeps_empty_results_in_plain_text_mode() {
    let dir = tempdir().unwrap();
    let registry_path = dir.path().join("registry.toml");
    let mut cmd = Command::cargo_bin("aim").unwrap();

    cmd.args(["search", "no-such-app-image-query"])
        .env("AIM_REGISTRY_PATH", &registry_path)
        .env(FIXTURE_MODE_ENV, "1")
        .assert()
        .success()
        .stdout(contains("Search Results"))
        .stdout(contains("No remote matches"));
}
