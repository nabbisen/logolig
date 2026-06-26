//! Domain model layer. No side effects; pure types only.

pub mod export_plan;
pub mod persisted_settings;
pub mod preview_profile;
pub mod raster;
pub mod resize_algorithm;
pub mod source_asset;
pub mod theme_mode;
pub mod vtracer_preset;
pub mod web_manifest;

pub use export_plan::{
    ExportPlan, ICO_SIZE_MAX, ICO_SIZE_MIN, PNG_SIZE_MAX, PNG_SIZE_MIN, SizeOverride,
};
pub use persisted_settings::PersistedSettings;
pub use preview_profile::{PreviewContext, PreviewProfile};
pub use raster::Rgba8;
pub use resize_algorithm::ResizeAlgorithm;
pub use source_asset::{SourceAsset, SourceKind};
pub use theme_mode::ThemeMode;
pub use vtracer_preset::VtracerPreset;
pub use web_manifest::WebManifestSettings;
