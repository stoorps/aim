use std::sync::Mutex;

use tempfile::tempdir;
use upm::config::{
    CliConfig, ConfigError, SearchConfig, ThemeConfig, default_path, load_from_path,
};
use upm::default_registry_path;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    key: &'static str,
    original: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, original }
    }

    fn remove(key: &'static str) -> Self {
        let original = std::env::var_os(key);
        unsafe {
            std::env::remove_var(key);
        }
        Self { key, original }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

#[test]
fn missing_config_file_returns_defaults() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let config = load_from_path(&path).unwrap();

    assert_eq!(config, CliConfig::default());
    assert_eq!(config.search, SearchConfig::default());
    assert!(!config.allow_http);
    assert!(config.search.bottom_to_top);
    assert!(!config.search.skip_confirmation);
    assert_eq!(config.theme.accent, "#b388ff");
    assert_eq!(config.theme.accent_secondary, "#d5c2ff");
    assert_eq!(config.theme.dim, "#7f7396");
}

#[test]
fn search_section_overrides_defaults() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(
        &path,
        "allow_http = true\n\n[search]\nbottom_to_top = false\nskip_confirmation = true\n\n[theme]\naccent = \"#9f6bff\"\naccent_secondary = \"#efe7ff\"\ndim = \"#6b6480\"\n",
    )
    .unwrap();

    let config = load_from_path(&path).unwrap();

    assert_eq!(
        config,
        CliConfig {
            allow_http: true,
            search: SearchConfig {
                bottom_to_top: false,
                skip_confirmation: true,
            },
            theme: ThemeConfig {
                accent: "#9f6bff".to_owned(),
                accent_secondary: "#efe7ff".to_owned(),
                dim: "#6b6480".to_owned(),
            },
        }
    );
}

#[test]
fn malformed_toml_returns_path_aware_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");
    std::fs::write(&path, "[search\nskip_confirmation = true\n").unwrap();

    let error = load_from_path(&path).unwrap_err();

    match error {
        ConfigError::Parse {
            path: error_path, ..
        } => {
            assert_eq!(error_path, path);
        }
        other => panic!("expected parse error, got {other:?}"),
    }
}

#[test]
fn default_config_path_uses_upm_directory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();

    let _config_path = EnvGuard::remove("UPM_CONFIG_PATH");
    let _xdg_config_home = EnvGuard::remove("XDG_CONFIG_HOME");
    let _home = EnvGuard::set("HOME", dir.path());

    let path = default_path();

    assert_eq!(path, dir.path().join(".config/upm/config.toml"));
}

#[test]
fn default_config_path_ignores_legacy_aim_override() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let legacy_path = dir.path().join("aim-config.toml");

    let _legacy_config_path = EnvGuard::set("AIM_CONFIG_PATH", &legacy_path);
    let _config_path = EnvGuard::remove("UPM_CONFIG_PATH");
    let _xdg_config_home = EnvGuard::remove("XDG_CONFIG_HOME");
    let _home = EnvGuard::set("HOME", dir.path());

    let path = default_path();

    assert_eq!(path, dir.path().join(".config/upm/config.toml"));
}

#[test]
fn default_registry_path_uses_upm_directory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();

    let _registry_path = EnvGuard::remove("UPM_REGISTRY_PATH");
    let _home = EnvGuard::set("HOME", dir.path());

    let path = default_registry_path();

    assert_eq!(path, dir.path().join(".local/share/upm/registry.toml"));
}

#[test]
fn default_registry_path_ignores_legacy_aim_override() {
    let _guard = ENV_LOCK.lock().unwrap();
    let dir = tempdir().unwrap();
    let legacy_path = dir.path().join("aim-registry.toml");

    let _legacy_registry_path = EnvGuard::set("AIM_REGISTRY_PATH", &legacy_path);
    let _registry_path = EnvGuard::remove("UPM_REGISTRY_PATH");
    let _home = EnvGuard::set("HOME", dir.path());

    let path = default_registry_path();

    assert_eq!(path, dir.path().join(".local/share/upm/registry.toml"));
}
