use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn help_lists_expected_commands() {
    let mut cmd = Command::cargo_bin("aim").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("remove"))
        .stdout(contains("list"))
        .stdout(contains("update"));
}
