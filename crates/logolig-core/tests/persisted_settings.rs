//! Persisted-settings round-trip tests + `SettingsStore` trait behaviour (v1.4.0).
//!
//! Uses `InMemoryStore` as a fixture to test the trait without file I/O.
//! This also serves as a design test: any future `BrowserStore` must
//! satisfy the same trait contract.
//!
//! Test coverage:
//! - JSON serialise / deserialise round-trip (including missing-field tolerance)
//! - `load_or_default()` for first-run and existing-value cases
//! - `update()` read-modify-write correctness

use std::cell::RefCell;
use std::convert::Infallible;

use logolig_core::{ExportPlan, PersistedSettings, ResizeAlgorithm, SettingsStore, ThemeMode};
use serde::{Serialize, de::DeserializeOwned};

// ---------------------------------------------------------------------------
// Test fixture: in-memory store
// ---------------------------------------------------------------------------

/// Simple store that holds a single `Option<String>` (JSON).
/// Uses `RefCell` for interior mutability. Not thread-safe (test use only).
struct InMemoryStore {
    blob: RefCell<Option<String>>,
}

impl InMemoryStore {
    fn empty() -> Self {
        Self {
            blob: RefCell::new(None),
        }
    }
}

impl<T> SettingsStore<T> for InMemoryStore
where
    T: Serialize + DeserializeOwned + Default,
{
    type Error = Infallible;

    fn load_or_default(&self) -> Result<T, Self::Error> {
        // Keep the borrow short to avoid conflicting with borrow_mut for the None case;
        // clone before matching — the standard RefCell pattern.
        let snapshot = self.blob.borrow().clone();
        match snapshot {
            Some(s) => Ok(serde_json::from_str(&s).expect("test JSON should parse")),
            None => {
                let d = T::default();
                let s = serde_json::to_string(&d).expect("default should serialize");
                *self.blob.borrow_mut() = Some(s);
                Ok(d)
            }
        }
    }

    fn save(&self, config: &T) -> Result<(), Self::Error> {
        let s = serde_json::to_string(config).expect("config should serialize");
        *self.blob.borrow_mut() = Some(s);
        Ok(())
    }

    fn update<F>(&self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut T),
    {
        let mut cfg = self.load_or_default()?;
        f(&mut cfg);
        self.save(&cfg)?;
        Ok(cfg)
    }
}

// ---------------------------------------------------------------------------
// 1. round-trip
// ---------------------------------------------------------------------------

#[test]
fn persisted_settings_round_trip_through_json() {
    let original = PersistedSettings {
        export_plan: {
            let mut p = ExportPlan::default();
            p.png_sizes = vec![32, 64, 192];
            p.algorithm = ResizeAlgorithm::Bilinear;
            p.include_svg = false;
            p
        },
        theme: ThemeMode::Dark,
        locale: Some("ja".to_string()),
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: PersistedSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.export_plan.png_sizes, vec![32, 64, 192]);
    assert_eq!(restored.export_plan.algorithm, ResizeAlgorithm::Bilinear);
    assert!(!restored.export_plan.include_svg);
    assert_eq!(restored.theme, ThemeMode::Dark);
    assert_eq!(restored.locale.as_deref(), Some("ja"));
}

// ---------------------------------------------------------------------------
// 2. load_or_default()
// ---------------------------------------------------------------------------

#[test]
fn load_or_default_returns_default_on_empty_store() {
    let store: InMemoryStore = InMemoryStore::empty();
    let loaded: PersistedSettings = store.load_or_default().unwrap();
    // Verify default ExportPlan values are returned
    assert_eq!(loaded.export_plan.png_sizes, vec![32, 192, 512]);
    assert_eq!(loaded.theme, ThemeMode::System);
    assert!(loaded.locale.is_none());

    // store now contains the default (written by load_or_default as a side effect).
    // A subsequent load returns the same value.
    let again: PersistedSettings = store.load_or_default().unwrap();
    assert_eq!(again.export_plan.png_sizes, loaded.export_plan.png_sizes);
}

// ---------------------------------------------------------------------------
// 3. update()
// ---------------------------------------------------------------------------

#[test]
fn update_modifies_and_persists() {
    let store: InMemoryStore = InMemoryStore::empty();
    // Call update assuming the default was written by the initial load_or_default.
    let _: PersistedSettings = store.load_or_default().unwrap();

    let after: PersistedSettings = store
        .update(|s: &mut PersistedSettings| {
            s.theme = ThemeMode::Light;
            s.export_plan.include_apple_touch = false;
        })
        .unwrap();
    assert_eq!(after.theme, ThemeMode::Light);
    assert!(!after.export_plan.include_apple_touch);

    // Immediately after, load_or_default returns the updated value.
    let reloaded: PersistedSettings = store.load_or_default().unwrap();
    assert_eq!(reloaded.theme, ThemeMode::Light);
    assert!(!reloaded.export_plan.include_apple_touch);
}

// ---------------------------------------------------------------------------
// 4. Forward compatibility (serde(default))
// ---------------------------------------------------------------------------

#[test]
fn missing_fields_are_filled_from_default() {
    // Simulate an old v1.4.0-pre JSON where some export_plan sub-fields
    // are missing. E.g. before v1.2.0, include_svg / vectorize_on_raster
    // did not exist.
    let legacy_json = r#"{
        "export_plan": {
            "include_ico": true,
            "png_sizes": [32, 192, 512]
        },
        "theme": "Dark"
    }"#;

    let restored: PersistedSettings = serde_json::from_str(legacy_json).unwrap();
    // Missing fields are filled with their defaults
    assert!(restored.export_plan.include_svg); // default = true
    assert!(restored.export_plan.vectorize_on_raster);
    assert_eq!(restored.theme, ThemeMode::Dark);
    assert!(restored.locale.is_none()); // missing → None
}

#[test]
fn entirely_empty_json_object_yields_full_default() {
    let restored: PersistedSettings = serde_json::from_str("{}").unwrap();
    let default = PersistedSettings::default();
    assert_eq!(
        restored.export_plan.png_sizes,
        default.export_plan.png_sizes
    );
    assert_eq!(restored.theme, default.theme);
    assert_eq!(restored.locale, default.locale);
}
