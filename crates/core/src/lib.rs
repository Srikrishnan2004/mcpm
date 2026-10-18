//! Core library for mcpm.
//!
//! Everything that is shared between the desktop app and the `mcpm` CLI/shim lives here:
//! the domain model, persistence, the server catalog, keychain-backed secrets,
//! client config adapters and the sync engine.

pub mod adapters;
pub mod catalog;
pub mod error;
pub mod model;
pub mod render;
pub mod secrets;
pub mod service;
pub mod store;
pub mod sync;

pub use error::{Error, Result};
