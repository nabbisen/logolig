//! # logolig (core)
//!
//! Pure logic layer for the local-first favicon generator.
//!
//! This crate has **no dependency on iced or snora**.
//! That is not a convention — it is a hard constraint enforced by the
//! dependency graph. This gives us:
//! - Image-processing logic that can be unit-tested without a GUI
//! - A reusable base for a future CLI binary or WASM front-end
//! - No rebuild of this crate during GUI iteration cycles
//!
//! (§3.1, §16)

pub mod domain;
pub mod error;
pub mod message_key;
pub mod services;
pub mod settings;

// Re-export the most commonly used types at the crate root so that
// upstream crates (logolig-app) can write `use logolig::SourceAsset`
// instead of the full module path.
pub use domain::{
    ExportPlan, ICO_SIZE_MAX, ICO_SIZE_MIN, PNG_SIZE_MAX, PNG_SIZE_MIN, PersistedSettings,
    PreviewContext, PreviewProfile, ResizeAlgorithm, Rgba8, SizeOverride, SourceAsset, SourceKind,
    ThemeMode, VtracerPreset, WebManifestSettings,
};
pub use error::AppError;
pub use message_key::MessageKey;
pub use services::exporter::{ExportReport, InMemoryArtifact};
pub use services::preview::PreviewCache;
pub use services::transparency_audit::TransparencyReport;
pub use settings::SettingsStore;
