use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use upm_core::integration::install::stage_and_commit_payload;

#[test]
fn payload_commit_moves_staged_appimage_into_final_location() {
    let root = tempdir().unwrap();
    let staging_root = root.path().join("staging");
    let payload_root = root.path().join("payloads");
    fs::create_dir(&staging_root).unwrap();
    fs::create_dir(&payload_root).unwrap();

    let staged_path = staging_root.join("bat.download");
    fs::write(&staged_path, b"\x7fELFAppImage").unwrap();
    let final_payload_path = payload_root.join("bat.AppImage");
    let outcome = stage_and_commit_payload(&staged_path, &final_payload_path).unwrap();

    assert_eq!(
        outcome
            .final_payload_path
            .extension()
            .and_then(|ext| ext.to_str()),
        Some("AppImage")
    );
    assert!(outcome.final_payload_path.exists());

    let mode = fs::metadata(&outcome.final_payload_path)
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111);
}
