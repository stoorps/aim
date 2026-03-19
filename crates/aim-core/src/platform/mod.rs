use std::path::{Path, PathBuf};

pub fn user_managed_appimages_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/lib/aim/appimages")
}

pub fn user_applications_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/share/applications")
}

pub fn user_icons_dir(home_dir: &Path) -> PathBuf {
    home_dir.join(".local/share/icons/hicolor/256x256/apps")
}

pub fn system_managed_appimages_dir() -> PathBuf {
    PathBuf::from("/opt/aim/appimages")
}

pub fn system_applications_dir() -> PathBuf {
    PathBuf::from("/usr/share/applications")
}

pub fn system_icons_dir() -> PathBuf {
    PathBuf::from("/usr/share/icons/hicolor/256x256/apps")
}
