use aim_cli::config::{CliConfig, ConfigError, SearchConfig, load_from_path};
use tempfile::tempdir;

#[test]
fn missing_config_file_returns_defaults() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let config = load_from_path(&path).unwrap();

    assert_eq!(config, CliConfig::default());
    assert_eq!(config.search, SearchConfig::default());
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
        "[search]\nbottom_to_top = false\nskip_confirmation = true\n\n[theme]\naccent = \"#9f6bff\"\naccent_secondary = \"#efe7ff\"\ndim = \"#6b6480\"\n",
    )
    .unwrap();

    let config = load_from_path(&path).unwrap();

    assert_eq!(
        config,
        CliConfig {
            search: SearchConfig {
                bottom_to_top: false,
                skip_confirmation: true,
            },
            theme: aim_cli::config::ThemeConfig {
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
