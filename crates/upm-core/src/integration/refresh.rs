use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::platform::DesktopHelpers;

pub fn refresh_integration(
    helpers: &DesktopHelpers,
    desktop_entry_path: Option<&Path>,
    icon_path: Option<&Path>,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if let (Some(helper), Some(path)) = (
        helpers.update_desktop_database_path.as_ref(),
        desktop_entry_path.and_then(Path::parent),
    ) {
        audit_helper(helper, &[path]);
        if let Err(error) = Command::new(helper).arg(path).status().inspect(|status| {
            audit_helper_status(helper, status.code());
        }) {
            audit_helper_failure(helper, &error.to_string());
            warnings.push(format!("update-desktop-database failed: {error}"));
        }
    } else if !helpers.update_desktop_database {
        warnings.push(
            "update-desktop-database not available; desktop cache refresh skipped".to_owned(),
        );
    }

    if let (Some(helper), Some(path)) = (
        helpers.gtk_update_icon_cache_path.as_ref(),
        icon_path.map(icon_theme_root),
    ) {
        audit_helper(helper, &[Path::new("-f"), Path::new("-t"), path.as_path()]);
        if let Err(error) = Command::new(helper)
            .args(["-f", "-t"])
            .arg(path)
            .status()
            .inspect(|status| {
                audit_helper_status(helper, status.code());
            })
        {
            audit_helper_failure(helper, &error.to_string());
            warnings.push(format!("gtk-update-icon-cache failed: {error}"));
        }
    } else if !helpers.gtk_update_icon_cache {
        warnings.push("gtk-update-icon-cache not available; icon cache refresh skipped".to_owned());
    }

    warnings
}

fn icon_theme_root(icon_path: &Path) -> PathBuf {
    for ancestor in icon_path.ancestors() {
        if ancestor.file_name().and_then(|name| name.to_str()) == Some("hicolor") {
            return ancestor.to_path_buf();
        }
    }

    icon_path.parent().unwrap_or(icon_path).to_path_buf()
}

fn audit_helper(helper: &Path, args: &[&Path]) {
    if env::var("UPM_DEBUG_EXTERNAL_HELPERS").ok().as_deref() != Some("1") {
        return;
    }

    let rendered_args = args
        .iter()
        .map(|arg| arg.display().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!(
        "[upm] helper exec: {}{}{}",
        helper.display(),
        if rendered_args.is_empty() { "" } else { " " },
        rendered_args
    );
}

fn audit_helper_status(helper: &Path, code: Option<i32>) {
    if env::var("UPM_DEBUG_EXTERNAL_HELPERS").ok().as_deref() != Some("1") {
        return;
    }

    match code {
        Some(code) => eprintln!("[upm] helper exit: {} code={code}", helper.display()),
        None => eprintln!(
            "[upm] helper exit: {} terminated by signal",
            helper.display()
        ),
    }
}

fn audit_helper_failure(helper: &Path, error: &str) {
    if env::var("UPM_DEBUG_EXTERNAL_HELPERS").ok().as_deref() != Some("1") {
        return;
    }

    eprintln!("[upm] helper failure: {} error={error}", helper.display());
}
