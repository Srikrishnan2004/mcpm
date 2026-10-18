//! Secret storage. Values live only in the OS keychain:
//! service `mcpm`, account `<instance-id>/<secret-key>`.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{Error, Result};

const SERVICE: &str = "mcpm";

pub trait SecretStore: Send + Sync {
    fn get(&self, instance: &str, key: &str) -> Result<Option<String>>;
    fn set(&self, instance: &str, key: &str, value: &str) -> Result<()>;
    fn delete(&self, instance: &str, key: &str) -> Result<()>;
}

fn account(instance: &str, key: &str) -> String {
    format!("{instance}/{key}")
}

/// Windows Credential Manager, macOS Keychain, or Linux Secret Service.
#[derive(Debug, Default)]
pub struct KeyringStore;

impl KeyringStore {
    fn entry(instance: &str, key: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(SERVICE, &account(instance, key))
            .map_err(|e| Error::Keychain(e.to_string()))
    }
}

impl SecretStore for KeyringStore {
    fn get(&self, instance: &str, key: &str) -> Result<Option<String>> {
        match Self::entry(instance, key)?.get_password() {
            Ok(v) => Ok(Some(v)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(Error::Keychain(e.to_string())),
        }
    }

    fn set(&self, instance: &str, key: &str, value: &str) -> Result<()> {
        Self::entry(instance, key)?
            .set_password(value)
            .map_err(|e| Error::Keychain(e.to_string()))
    }

    fn delete(&self, instance: &str, key: &str) -> Result<()> {
        match Self::entry(instance, key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Keychain(e.to_string())),
        }
    }
}

/// In-memory store for tests.
#[derive(Debug, Default)]
pub struct MemoryStore(Mutex<HashMap<String, String>>);

impl SecretStore for MemoryStore {
    fn get(&self, instance: &str, key: &str) -> Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(&account(instance, key)).cloned())
    }

    fn set(&self, instance: &str, key: &str, value: &str) -> Result<()> {
        self.0
            .lock()
            .unwrap()
            .insert(account(instance, key), value.to_string());
        Ok(())
    }

    fn delete(&self, instance: &str, key: &str) -> Result<()> {
        self.0.lock().unwrap().remove(&account(instance, key));
        Ok(())
    }
}
