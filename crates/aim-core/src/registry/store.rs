use std::fs::{self, File, OpenOptions};
use std::path::PathBuf;

use fs2::FileExt;

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
        let temporary_path = self.temporary_path();
        fs::write(&temporary_path, contents)?;
        fs::rename(&temporary_path, &self.path).map_err(|error| {
            let _ = fs::remove_file(&temporary_path);
            RegistryStoreError::Io(error)
        })?;
        Ok(())
    }

    pub fn lock_exclusive(&self) -> Result<RegistryLock, RegistryStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let lock_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(self.lock_path())?;

        match lock_file.try_lock_exclusive() {
            Ok(()) => Ok(RegistryLock { file: lock_file }),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                Err(RegistryStoreError::LockUnavailable)
            }
            Err(error) => Err(RegistryStoreError::Io(error)),
        }
    }

    pub fn mutate_exclusive<F>(&self, apply: F) -> Result<Registry, RegistryStoreError>
    where
        F: FnOnce(&mut Registry),
    {
        let _lock = self.lock_exclusive()?;
        let mut registry = self.load()?;
        apply(&mut registry);
        self.save(&registry)?;
        Ok(registry)
    }

    fn lock_path(&self) -> PathBuf {
        self.path.with_extension("toml.lock")
    }

    fn temporary_path(&self) -> PathBuf {
        self.path.with_extension("toml.tmp")
    }
}

#[derive(Debug)]
pub struct RegistryLock {
    file: File,
}

impl Drop for RegistryLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

#[derive(Debug)]
pub enum RegistryStoreError {
    Io(std::io::Error),
    LockUnavailable,
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
