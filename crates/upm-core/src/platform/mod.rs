pub mod capabilities;
pub mod distro;

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use crate::domain::app::InstallScope;
pub use capabilities::{DesktopHelpers, HostCapabilities, WritableRoots};
pub use distro::{DistroFamily, detect_distro_family};

const OS_RELEASE_PATH_ENV: &str = "UPM_OS_RELEASE_PATH";
const HELPER_PATHS_ENV: &str = "UPM_HELPER_PATHS";

pub fn user_managed_appimages_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/lib/upm/appimages")
}

pub fn user_applications_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/share/applications")
}

pub fn user_icons_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/share/icons/hicolor/256x256/apps")
}

pub fn system_managed_appimages_dir() -> PathBuf {
    PathBuf::from("/opt/upm/appimages")
}

pub fn system_applications_dir() -> PathBuf {
    PathBuf::from("/usr/share/applications")
}

pub fn system_icons_dir() -> PathBuf {
    PathBuf::from("/usr/share/icons/hicolor/256x256/apps")
}

pub fn probe_live_host(
    home_dir: &Path,
    requested_scope: InstallScope,
) -> io::Result<(DistroFamily, HostCapabilities)> {
    let os_release = load_os_release()?;
    let family = detect_distro_family(&os_release);
    let helper_paths = helper_search_paths();
    let helper_refs = helper_paths
        .iter()
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();
    let helpers = capabilities::probe_desktop_helpers(&helper_refs);
    let (payload_root, desktop_root, icon_root) = match requested_scope {
        InstallScope::User => (
            user_managed_appimages_dir(home_dir),
            user_applications_dir(home_dir),
            user_icons_dir(home_dir),
        ),
        InstallScope::System => (
            system_managed_appimages_dir(),
            system_applications_dir(),
            system_icons_dir(),
        ),
    };

    Ok((
        family,
        HostCapabilities {
            is_immutable: family == DistroFamily::Immutable,
            is_nix: family == DistroFamily::Nix,
            has_desktop_session: env::var_os("DISPLAY").is_some()
                || env::var_os("WAYLAND_DISPLAY").is_some()
                || env::var_os("XDG_CURRENT_DESKTOP").is_some(),
            helpers,
            writable_roots: capabilities::probe_writable_roots(
                &payload_root,
                &desktop_root,
                &icon_root,
            ),
        },
    ))
}

fn load_os_release() -> io::Result<String> {
    let path = env::var_os(OS_RELEASE_PATH_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/os-release"));

    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error),
    }
}

fn helper_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(extra_paths) = env::var_os(HELPER_PATHS_ENV) {
        paths.extend(env::split_paths(&extra_paths));
    }
    if let Some(system_paths) = env::var_os("PATH") {
        paths.extend(env::split_paths(&system_paths));
    }

    paths
}
