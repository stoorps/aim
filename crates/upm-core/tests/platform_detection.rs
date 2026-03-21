use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::tempdir;
use upm_core::platform::capabilities::{probe_desktop_helpers, probe_writable_roots};
use upm_core::platform::distro::{DistroFamily, detect_distro_family};

#[test]
fn detects_fedora_family_from_os_release() {
    let distro = detect_distro_family("ID=fedora\nID_LIKE=rhel centos\n");
    assert_eq!(distro, DistroFamily::Fedora);
}

#[test]
fn detects_immutable_family_from_variant_id() {
    let distro = detect_distro_family("ID=fedora\nVARIANT_ID=silverblue\n");
    assert_eq!(distro, DistroFamily::Immutable);
}

#[test]
fn probes_desktop_helpers_from_search_paths() {
    let helper_dir = tempdir().unwrap();
    let update_desktop_database = helper_dir.path().join("update-desktop-database");
    let gtk_update_icon_cache = helper_dir.path().join("gtk-update-icon-cache");

    fs::write(&update_desktop_database, "#!/bin/sh\n").unwrap();
    fs::write(&gtk_update_icon_cache, "#!/bin/sh\n").unwrap();
    fs::set_permissions(&update_desktop_database, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(&gtk_update_icon_cache, fs::Permissions::from_mode(0o755)).unwrap();

    let helpers = probe_desktop_helpers(&[helper_dir.path()]);

    assert!(helpers.update_desktop_database);
    assert!(helpers.gtk_update_icon_cache);
}

#[test]
fn probes_writable_roots_from_candidate_directories() {
    let root = tempdir().unwrap();
    let payload = root.path().join("payload");
    let desktop_entries = root.path().join("applications");
    let icons = root.path().join("icons");

    fs::create_dir(&payload).unwrap();
    fs::create_dir(&desktop_entries).unwrap();
    fs::create_dir(&icons).unwrap();

    let writable = probe_writable_roots(&payload, &desktop_entries, &icons);

    assert!(writable.payload);
    assert!(writable.desktop_entries);
    assert!(writable.icons);
}
