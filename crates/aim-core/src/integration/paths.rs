use std::path::{Path, PathBuf};

use crate::domain::app::InstallScope;
use crate::platform::{
    system_applications_dir, system_icons_dir, system_managed_appimages_dir, user_applications_dir,
    user_icons_dir, user_managed_appimages_dir,
};

pub fn managed_appimage_path(home_dir: &Path, scope: InstallScope, app_id: &str) -> PathBuf {
    scope_managed_dir(home_dir, scope).join(format!("{app_id}.AppImage"))
}

pub fn desktop_entry_path(home_dir: &Path, scope: InstallScope, app_id: &str) -> PathBuf {
    scope_applications_dir(home_dir, scope).join(format!("aim-{app_id}.desktop"))
}

pub fn icon_path(home_dir: &Path, scope: InstallScope, app_id: &str) -> PathBuf {
    scope_icons_dir(home_dir, scope).join(format!("{app_id}.png"))
}

fn scope_managed_dir(home_dir: &Path, scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => user_managed_appimages_dir(home_dir),
        InstallScope::System => system_managed_appimages_dir(),
    }
}

fn scope_applications_dir(home_dir: &Path, scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => user_applications_dir(home_dir),
        InstallScope::System => system_applications_dir(),
    }
}

fn scope_icons_dir(home_dir: &Path, scope: InstallScope) -> PathBuf {
    match scope {
        InstallScope::User => user_icons_dir(home_dir),
        InstallScope::System => system_icons_dir(),
    }
}
