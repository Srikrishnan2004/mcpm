//! Persistence of mcpm's own state (no secrets) in the OS config directory.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::model::{ClientId, Instance, Project};
use crate::{Error, Result};

pub const REGISTRY: &str = "registry.json";
pub const PROJECTS: &str = "projects.json";
pub const MANIFEST: &str = "manifest.json";

/// Root of mcpm's data: `%APPDATA%\mcpm`, `~/Library/Application Support/mcpm`, `~/.config/mcpm`.
/// Overridable with `MCPM_HOME`.
#[derive(Debug, Clone)]
pub struct DataDir {
    root: PathBuf,
}

impl DataDir {
    pub fn from_env() -> Result<Self> {
        let root = match std::env::var_os("MCPM_HOME") {
            Some(p) => PathBuf::from(p),
            None => dirs::config_dir()
                .ok_or_else(|| Error::Invalid("cannot determine the config directory".into()))?
                .join("mcpm"),
        };
        Ok(Self { root })
    }

    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    pub fn load<T: DeserializeOwned + Default>(&self, name: &str) -> Result<T> {
        read_json_or_default(&self.path(name))
    }

    pub fn save<T: Serialize>(&self, name: &str, value: &T) -> Result<()> {
        write_json_atomic(&self.path(name), value)
    }
}

pub fn read_json_or_default<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    match fs::read_to_string(path) {
        Ok(text) if text.trim().is_empty() => Ok(T::default()),
        Ok(text) => serde_json::from_str(&text).map_err(|e| Error::json(path, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(Error::io(path, e)),
    }
}

/// Write to a sibling temp file, then rename over the target.
pub fn write_json_atomic<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|e| Error::json(path, e))?;
    text.push('\n');
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!("{file_name}.mcpm-tmp"));
    fs::write(&tmp, text).map_err(|e| Error::io(&tmp, e))?;
    fs::rename(&tmp, path).map_err(|e| Error::io(path, e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    #[serde(default)]
    pub instances: Vec<Instance>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            version: 1,
            instances: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projects {
    pub version: u32,
    #[serde(default)]
    pub projects: Vec<Project>,
}

impl Default for Projects {
    fn default() -> Self {
        Self {
            version: 1,
            projects: Vec::new(),
        }
    }
}

/// What mcpm wrote where. Entries not listed here are user-owned and never touched.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u32,
    #[serde(default)]
    pub files: BTreeMap<String, FileRecord>,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            version: 1,
            files: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub client: ClientId,
    #[serde(default)]
    pub entries: BTreeMap<String, EntryRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryRecord {
    pub instance: String,
    /// The exact value mcpm wrote; used to detect hand edits (drift).
    pub value: Value,
}
