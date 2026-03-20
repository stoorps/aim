use std::env;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct LoadedConfig {
    pub config: AppConfig,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct AppConfig {
    pub theme: ThemeConfig,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ThemeConfig {
    pub heading: Option<String>,
    pub accent: Option<String>,
    pub muted: Option<String>,
    pub label: Option<String>,
    pub bullet: Option<String>,
    pub success: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub progress_spinner: Option<String>,
    pub progress_bar: Option<String>,
    pub progress_bar_unfilled: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FileConfig {
    #[serde(default)]
    theme: FileThemeConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct FileThemeConfig {
    heading: Option<String>,
    accent: Option<String>,
    muted: Option<String>,
    label: Option<String>,
    bullet: Option<String>,
    success: Option<String>,
    warning: Option<String>,
    error: Option<String>,
    progress_spinner: Option<String>,
    progress_bar: Option<String>,
    progress_bar_unfilled: Option<String>,
}

impl AppConfig {
    pub fn load() -> LoadedConfig {
        let system_path = Some(PathBuf::from("/etc/aim/config.toml"));
        let user_path = env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".config/aim/config.toml"));
        Self::load_from_paths(system_path.as_deref(), user_path.as_deref())
    }

    pub fn load_from_paths(system_path: Option<&Path>, user_path: Option<&Path>) -> LoadedConfig {
        let mut loaded = LoadedConfig::default();

        if let Some(path) = system_path {
            merge_file(path, &mut loaded);
        }

        if let Some(path) = user_path {
            merge_file(path, &mut loaded);
        }

        loaded
    }
}

fn merge_file(path: &Path, loaded: &mut LoadedConfig) {
    if !path.exists() {
        return;
    }

    let contents = match std::fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(error) => {
            loaded
                .warnings
                .push(format!("failed to read {}: {error}", path.display()));
            return;
        }
    };

    let parsed: FileConfig = match toml::from_str(&contents) {
        Ok(parsed) => parsed,
        Err(error) => {
            loaded
                .warnings
                .push(format!("failed to parse {}: {error}", path.display()));
            return;
        }
    };

    merge_theme(&mut loaded.config.theme, parsed.theme);
}

fn merge_theme(theme: &mut ThemeConfig, update: FileThemeConfig) {
    merge_option(&mut theme.heading, update.heading);
    merge_option(&mut theme.accent, update.accent);
    merge_option(&mut theme.muted, update.muted);
    merge_option(&mut theme.label, update.label);
    merge_option(&mut theme.bullet, update.bullet);
    merge_option(&mut theme.success, update.success);
    merge_option(&mut theme.warning, update.warning);
    merge_option(&mut theme.error, update.error);
    merge_option(&mut theme.progress_spinner, update.progress_spinner);
    merge_option(&mut theme.progress_bar, update.progress_bar);
    merge_option(
        &mut theme.progress_bar_unfilled,
        update.progress_bar_unfilled,
    );
}

fn merge_option(target: &mut Option<String>, update: Option<String>) {
    if let Some(value) = update {
        *target = Some(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn user_config_overrides_system_theme_values() {
        let dir = tempdir().unwrap();
        let system_path = dir.path().join("system-config.toml");
        let user_path = dir.path().join("user-config.toml");

        std::fs::write(
            &system_path,
            "[theme]\nheading = \"amber\"\naccent = \"teal\"\n",
        )
        .unwrap();
        std::fs::write(&user_path, "[theme]\nheading = \"#d28b26\"\n").unwrap();

        let loaded = AppConfig::load_from_paths(Some(&system_path), Some(&user_path));

        assert_eq!(loaded.config.theme.heading.as_deref(), Some("#d28b26"));
        assert_eq!(loaded.config.theme.accent.as_deref(), Some("teal"));
        assert!(loaded.warnings.is_empty());
    }

    #[test]
    fn invalid_config_is_ignored_without_aborting_load() {
        let dir = tempdir().unwrap();
        let system_path = dir.path().join("system-config.toml");

        std::fs::write(&system_path, "[theme\nheading = \"amber\"\n").unwrap();

        let loaded = AppConfig::load_from_paths(Some(&system_path), None);

        assert_eq!(loaded.config.theme.heading, None);
        assert!(!loaded.warnings.is_empty());
    }
}
