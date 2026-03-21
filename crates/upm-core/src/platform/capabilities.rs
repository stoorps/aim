use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DesktopHelpers {
    pub update_desktop_database: bool,
    pub gtk_update_icon_cache: bool,
    pub update_desktop_database_path: Option<PathBuf>,
    pub gtk_update_icon_cache_path: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WritableRoots {
    pub payload: bool,
    pub desktop_entries: bool,
    pub icons: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostCapabilities {
    pub is_immutable: bool,
    pub is_nix: bool,
    pub has_desktop_session: bool,
    pub helpers: DesktopHelpers,
    pub writable_roots: WritableRoots,
}

impl HostCapabilities {
    pub fn immutable_user_only() -> Self {
        Self {
            is_immutable: true,
            has_desktop_session: true,
            ..Self::default()
        }
    }
}

pub fn probe_desktop_helpers(search_paths: &[&Path]) -> DesktopHelpers {
    let update_desktop_database_path = command_path(search_paths, "update-desktop-database");
    let gtk_update_icon_cache_path = command_path(search_paths, "gtk-update-icon-cache");

    DesktopHelpers {
        update_desktop_database: update_desktop_database_path.is_some(),
        gtk_update_icon_cache: gtk_update_icon_cache_path.is_some(),
        update_desktop_database_path,
        gtk_update_icon_cache_path,
    }
}

pub fn probe_writable_roots(payload: &Path, desktop_entries: &Path, icons: &Path) -> WritableRoots {
    WritableRoots {
        payload: is_writable_dir(payload),
        desktop_entries: is_writable_dir(desktop_entries),
        icons: is_writable_dir(icons),
    }
}

fn command_path(search_paths: &[&Path], executable: &str) -> Option<PathBuf> {
    search_paths
        .iter()
        .map(|path| path.join(executable))
        .find(|candidate| is_executable_file(candidate))
}

fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn is_writable_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }

    let probe_path = path.join(".upm-write-test");
    let result = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&probe_path);

    match result {
        Ok(_) => {
            let _ = fs::remove_file(&probe_path);
            true
        }
        Err(_) => false,
    }
}
