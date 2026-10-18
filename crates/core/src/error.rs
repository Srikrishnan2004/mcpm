use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("could not parse {path}: {source} (files with comments are not supported yet)")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("{0}")]
    Invalid(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error(
        "entry `{entry}` already exists in {path} and is not managed by mcpm; \
         rename the binding or remove the existing entry"
    )]
    Collision { entry: String, path: PathBuf },

    #[error("keychain error: {0}")]
    Keychain(String),
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn json(path: impl Into<PathBuf>, source: serde_json::Error) -> Self {
        Self::Json {
            path: path.into(),
            source,
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
