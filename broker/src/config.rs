use protocol::OpSecretReference;
use serde::Deserialize;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct BrokerConfig {
    pub socket_path: PathBuf,
    #[serde(default)]
    pub items: BTreeSet<OpSecretReference>,
}

#[derive(Debug, Clone)]
pub struct ItemLookup<'a> {
    pub reference: &'a OpSecretReference,
}

impl BrokerConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let raw = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let config: Self = serde_json::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        if config.socket_path.as_os_str().is_empty() {
            return Err(ConfigError::MissingSocket);
        }
        Ok(config)
    }

    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    pub fn resolve<'a>(&'a self, reference: &'a OpSecretReference) -> Option<ItemLookup<'a>> {
        self.items
            .get(reference)
            .map(|reference| ItemLookup { reference })
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config {path:?}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config {path:?}: {source}")]
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("socket_path must not be empty")]
    MissingSocket,
}
