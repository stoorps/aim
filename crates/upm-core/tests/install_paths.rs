use std::path::Path;

use upm_core::domain::app::InstallScope;
use upm_core::integration::paths::{desktop_entry_path, managed_appimage_path};

#[test]
fn user_scope_path_lands_under_home_managed_dir() {
    let path = managed_appimage_path(Path::new("/home/test"), InstallScope::User, "bat");

    assert_eq!(
        path,
        Path::new("/home/test/.local/lib/upm/appimages/bat.AppImage")
    );
}

#[test]
fn system_scope_path_lands_under_opt_upm_dir() {
    let path = managed_appimage_path(Path::new("/home/test"), InstallScope::System, "bat");

    assert_eq!(path, Path::new("/opt/upm/appimages/bat.AppImage"));
}

#[test]
fn system_scope_desktop_entry_uses_upm_prefix() {
    let path = desktop_entry_path(Path::new("/home/test"), InstallScope::System, "bat");

    assert_eq!(path, Path::new("/usr/share/applications/upm-bat.desktop"));
}
