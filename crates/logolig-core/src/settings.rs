//! Settings persistence abstraction (v1.4.0).
//!
//! `SettingsStore<T>` is a trait for persisting a single value of type `T`.
//! Its API shape is intentionally aligned with `ConfigManager<T>` from
//! `app-json-settings` v2:
//!
//! - `load_or_default()`: standard load on startup; saves and returns
//!   `T::default()` when the file / storage is empty
//! - `save()`:            full-replacement write
//! - `update()`:          safe read-modify-write
//!
//! ## Why a trait?
//!
//! v1 (native iced) persists to a JSON file in the OS config directory.
//! v2 (future WASM browser build) would persist to LocalStorage.
//! The storage medium differs but the API shape is identical.
//!
//! Defining this trait in `logolig-core` lets us abstract the persistence
//! mechanism in one place. Migration to v2 is an implementation swap only.
//!
//! ## Dependency direction
//!
//! ```text
//!         logolig-app          (v1)
//!          ↓
//!   ┌────────────┐
//!   │ logolig-   │  ← trait defined here
//!   │  core      │    (depends on serde, not on file I/O)
//!   └────────────┘
//!          ↑
//!         logolig-web          (v2, future)
//! ```
//!
//! `logolig-core` has **no file I/O and no localStorage abstraction**.
//! The concrete implementation is provided by the calling crate
//! (logolig-app or logolig-web).

use std::error::Error as StdError;

use serde::{Serialize, de::DeserializeOwned};

/// A store that persists exactly one value of type `T`.
///
/// `T` must satisfy `Serialize + DeserializeOwned + Default`. The `Default`
/// bound lets `load_or_default()` handle the "first run, no file yet" case
/// cleanly. In logolig, `T` is always `PersistedSettings`.
///
/// # Implementation notes
///
/// - `Error` varies per implementation (file I/O errors, serde errors,
///   localStorage permission errors, …). The trait requires only
///   `StdError + Send + Sync`; the concrete type is left to the implementor.
/// - `update()` is a composition of `load_or_default()` → apply closure →
///   `save()`. Atomicity across concurrent updates is not guaranteed
///   (logolig uses sequential single-process updates only).
pub trait SettingsStore<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// Implementation-specific error type.
    type Error: StdError + Send + Sync + 'static;

    /// Load the existing value, or save and return the default if absent.
    fn load_or_default(&self) -> Result<T, Self::Error>;

    /// Write a new value, replacing whatever was stored.
    fn save(&self, config: &T) -> Result<(), Self::Error>;

    /// Load → mutate via closure → save. Returns the updated `T`.
    /// The UI uses the return value to get the latest saved state.
    fn update<F>(&self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut T);
}
