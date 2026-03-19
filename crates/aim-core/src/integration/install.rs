use std::path::{Path, PathBuf};

pub fn staged_appimage_path(staging_root: &Path, app_id: &str) -> PathBuf {
    staging_root.join(format!("{app_id}.download"))
}

pub fn replacement_path(target: &Path) -> PathBuf {
    let mut file_name = target
        .file_name()
        .map(|name| name.to_os_string())
        .unwrap_or_default();
    file_name.push(".new");
    target.with_file_name(file_name)
}
