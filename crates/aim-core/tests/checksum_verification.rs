use std::fs;

use aim_core::integration::install::{InstallRequest, PayloadInstallError, execute_install};
use aim_core::platform::DesktopHelpers;
use tempfile::tempdir;

const VALID_FIXTURE_SHA512: &str =
    "ZZma4ZD+9XB4GGTHCNZu8I92OY02YrEvIG89ZtRNi99W8SZKwWkmGZz/QyNBxqAt0XeiKtcR80/dMnKlwpcIWw==";

#[test]
fn install_succeeds_with_valid_trusted_checksum() {
    let root = tempdir().unwrap();
    let staged_path = write_staged_payload(
        root.path(),
        b"\x7fELFAppImage\x89PNG\r\n\x1a\nicondataIEND\xaeB`\x82",
    );
    let final_payload_path = root.path().join("payloads/bat.AppImage");

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: Some(VALID_FIXTURE_SHA512),
        desktop: None,
        helpers: DesktopHelpers::default(),
    })
    .unwrap();

    assert_eq!(outcome.final_payload_path, final_payload_path);
    assert!(outcome.final_payload_path.exists());
}

#[test]
fn install_succeeds_without_trusted_checksum() {
    let root = tempdir().unwrap();
    let staged_path = write_staged_payload(root.path(), b"\x7fELFAppImage");
    let final_payload_path = root.path().join("payloads/bat.AppImage");

    let outcome = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: None,
        desktop: None,
        helpers: DesktopHelpers::default(),
    })
    .unwrap();

    assert!(outcome.final_payload_path.exists());
}

#[test]
fn install_fails_before_commit_when_trusted_checksum_mismatches() {
    let root = tempdir().unwrap();
    let staged_path = write_staged_payload(root.path(), b"\x7fELFAppImage");
    let final_payload_path = root.path().join("payloads/bat.AppImage");

    let error = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: Some(VALID_FIXTURE_SHA512),
        desktop: None,
        helpers: DesktopHelpers::default(),
    })
    .unwrap_err();

    assert!(matches!(error, PayloadInstallError::ChecksumMismatch));
    assert!(!final_payload_path.exists());
    assert!(!staged_path.exists());
}

#[test]
fn malformed_trusted_checksum_fails_before_commit() {
    let root = tempdir().unwrap();
    let staged_path = write_staged_payload(root.path(), b"\x7fELFAppImage");
    let final_payload_path = root.path().join("payloads/bat.AppImage");

    let error = execute_install(&InstallRequest {
        staged_payload_path: &staged_path,
        final_payload_path: &final_payload_path,
        trusted_checksum: Some("not-base64"),
        desktop: None,
        helpers: DesktopHelpers::default(),
    })
    .unwrap_err();

    assert!(matches!(error, PayloadInstallError::InvalidTrustedChecksum));
    assert!(!final_payload_path.exists());
    assert!(!staged_path.exists());
}

fn write_staged_payload(root: &std::path::Path, bytes: &[u8]) -> std::path::PathBuf {
    let staged_path = root.join("staging/bat.download");
    fs::create_dir_all(staged_path.parent().unwrap()).unwrap();
    fs::write(&staged_path, bytes).unwrap();
    staged_path
}
