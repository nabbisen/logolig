//! Native (file-system) implementation of `SettingsStore` (v1.4.0).
//!
//! Thin wrapper around `app-json-settings` v2's `ConfigManager<T>` that
//! satisfies the `SettingsStore` trait from logolig-core.
//!
//! ## Why a wrapper?
//!
//! `ConfigManager` could be used directly and it would work. However,
//! keeping persistence behind the trait means UI code never touches the
//! storage implementation. When v2 adds a `BrowserStore` (LocalStorage),
//! the change is contained to this one file.

use std::fmt;

use app_json_settings::{ConfigError, ConfigManager};

use logolig_core::{PersistedSettings, SettingsStore};

/// SettingsStore for the native (file-system) environment.
pub struct NativeStore {
    inner: ConfigManager<PersistedSettings>,
}

impl std::fmt::Debug for NativeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ConfigManager does not impl Debug; expose path to show
        // where the settings file is stored.
        f.debug_struct("NativeStore")
            .field("path", &self.inner.path())
            .finish()
    }
}

impl NativeStore {
    /// Initialise at the standard OS config dir path (…/logolig/settings.json).
    pub fn new() -> Self {
        Self {
            inner: ConfigManager::new().with_filename("settings.json"),
        }
    }
}

impl SettingsStore<PersistedSettings> for NativeStore {
    type Error = NativeStoreError;

    fn load_or_default(&self) -> Result<PersistedSettings, Self::Error> {
        self.inner.load_or_default().map_err(NativeStoreError)
    }

    fn save(&self, config: &PersistedSettings) -> Result<(), Self::Error> {
        self.inner.save(config).map_err(NativeStoreError)
    }

    fn update<F>(&self, f: F) -> Result<PersistedSettings, Self::Error>
    where
        F: FnOnce(&mut PersistedSettings),
    {
        self.inner.update(f).map_err(NativeStoreError)
    }
}

/// Wrapper around `app_json_settings::ConfigError` providing `Display` and `std::error::Error`
/// .
#[derive(Debug)]
pub struct NativeStoreError(pub ConfigError);

impl fmt::Display for NativeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ConfigError::Io(e) => write!(f, "settings I/O error: {e}"),
            ConfigError::Serialize(e) => write!(f, "settings serialize error: {e}"),
            ConfigError::Deserialize(e) => write!(f, "settings deserialize error: {e}"),
        }
    }
}

impl std::error::Error for NativeStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ConfigError::Io(e) => Some(e),
            ConfigError::Serialize(e) | ConfigError::Deserialize(e) => Some(e),
        }
    }
}
