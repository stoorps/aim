use assert_cmd::Command;

#[test]
fn cli_shows_help() {
    let mut cmd = Command::cargo_bin("upm").unwrap();
    cmd.arg("--help").assert().success();
}
