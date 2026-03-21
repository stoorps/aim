use assert_cmd::Command;
use predicates::str::contains;

use clap::Parser;
use upm::cli::args::Command as UpmCommand;
use upm::{Cli, DispatchError};
use upm_core::domain::show::{ShowResultError, SourceSummary};
use upm_core::domain::source::SourceKind;

#[test]
fn help_lists_expected_commands() {
    let mut cmd = Command::cargo_bin("upm").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(contains("search"))
        .stdout(contains("show"))
        .stdout(contains("remove"))
        .stdout(contains("list"))
        .stdout(contains("update"));
}

#[test]
fn cli_parses_show_subcommand() {
    let cli = Cli::try_parse_from(["upm", "show", "legacy-bat"]).unwrap();

    match cli.command {
        Some(UpmCommand::Show { value }) => assert_eq!(value.as_deref(), Some("legacy-bat")),
        other => panic!("expected show command, got {other:?}"),
    }
}

#[test]
fn cli_parses_bare_show_subcommand() {
    let cli = Cli::try_parse_from(["upm", "show"]).unwrap();

    match cli.command {
        Some(UpmCommand::Show { value }) => assert_eq!(value, None),
        other => panic!("expected bare show command, got {other:?}"),
    }
}

#[test]
fn show_ambiguity_error_is_readable() {
    let error = DispatchError::Show(ShowResultError::AmbiguousInstalledMatch {
        query: "bat".to_owned(),
        matches: vec![
            "Bat (bat)".to_owned(),
            "Bat Preview (legacy-bat)".to_owned(),
        ],
    });

    let rendered = error.to_string();

    assert!(rendered.contains("multiple installed apps match bat"));
    assert!(rendered.contains("Bat (bat)"));
    assert!(rendered.contains("Bat Preview (legacy-bat)"));
}

#[test]
fn show_no_installable_artifact_error_is_readable() {
    let error = DispatchError::Show(ShowResultError::NoInstallableArtifact {
        source: SourceSummary {
            kind: SourceKind::SourceForge,
            locator: "https://sourceforge.net/projects/team-app/".to_owned(),
            canonical_locator: Some("team-app".to_owned()),
        },
    });

    let rendered = error.to_string();

    assert!(rendered.contains("no installable artifact found"));
    assert!(rendered.contains("sourceforge"));
    assert!(rendered.contains("https://sourceforge.net/projects/team-app/"));
}
