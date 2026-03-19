use std::fs;
use std::path::PathBuf;

use crate::registry::model::Registry;

pub struct RegistryStore {
    path: PathBuf,
}

impl RegistryStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Registry, RegistryStoreError> {
        if !self.path.exists() {
            return Ok(Registry::default());
        }

        let contents = fs::read_to_string(&self.path)?;
        let registry = toml::from_str(&contents)?;
        Ok(registry)
    }

    pub fn save(&self, registry: &Registry) -> Result<(), RegistryStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let contents = toml::to_string(registry)?;
        fs::write(&self.path, contents)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RegistryStoreError {
    Io(std::io::Error),
    SerializeToml(toml::ser::Error),
    Toml(toml::de::Error),
}

impl From<std::io::Error> for RegistryStoreError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for RegistryStoreError {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error)
    }
}

impl From<toml::ser::Error> for RegistryStoreError {
    fn from(error: toml::ser::Error) -> Self {
        Self::SerializeToml(error)
    }
}
