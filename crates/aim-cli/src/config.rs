use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize)]
pub struct CliConfig {
    #[serde(default)]
    pub allow_http: bool,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct SearchConfig {
    #[serde(default = "default_true")]
    pub bottom_to_top: bool,
    #[serde(default)]
    pub skip_confirmation: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            bottom_to_top: true,
            skip_confirmation: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_accent")]
    pub accent: String,
    #[serde(default = "default_accent_secondary")]
    pub accent_secondary: String,
    #[serde(default = "default_dim")]
    pub dim: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            accent: default_accent(),
            accent_secondary: default_accent_secondary(),
            dim: default_dim(),
        }
    }
}

pub fn load() -> Result<CliConfig, ConfigError> {
    load_from_path(&default_path())
}

pub fn load_from_path(path: &Path) -> Result<CliConfig, ConfigError> {
    match fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(CliConfig::default()),
        Err(source) => Err(ConfigError::Read {
            path: path.to_path_buf(),
            source,
        }),
    }
}

pub fn default_path() -> PathBuf {
    if let Some(path) = env::var_os("AIM_CONFIG_PATH") {
        return PathBuf::from(path);
    }

    if let Some(config_home) = env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(config_home).join("aim/config.toml");
    }

    let home = env::var_os("HOME").unwrap_or_else(|| ".".into());
    PathBuf::from(home).join(".config/aim/config.toml")
}

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

fn default_true() -> bool {
    true
}

fn default_accent() -> String {
    "#b388ff".to_owned()
}

fn default_accent_secondary() -> String {
    "#d5c2ff".to_owned()
}

fn default_dim() -> String {
    "#7f7396".to_owned()
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(
                    formatter,
                    "failed to read config {}: {source}",
                    path.display()
                )
            }
            Self::Parse { path, source } => {
                write!(
                    formatter,
                    "failed to parse config {}: {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}
