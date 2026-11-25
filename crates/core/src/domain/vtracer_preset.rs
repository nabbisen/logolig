//! vtracer presets (v1.4.1).
//!
//! Provides three presets to tune the raster→SVG vectorisation quality
//! introduced in v1.2.0.
//!
//! ## Why presets?
//!
//! vtracer's raw `Config` has many parameters (color_precision,
//! filter_speckle, corner_threshold, layer_difference, …). Exposing them
//! directly puts the user in a "I don't know what this does" position.
//! Following spec §5 "reduce decision fatigue", we offer three named
//! operating points instead.

use serde::{Deserialize, Serialize};

/// vtracer tuning preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VtracerPreset {
    /// Logos, icons, line art. Keeps corners sharp: `corner_threshold` 45, where 80 (before RFC 022)
    /// rounded corners of about 100 to 120 degrees more than the default does.
    Sharp,
    /// Same vtracer defaults as v1.2.0. Used as the default.
    #[default]
    Default,
    /// Photo-like and colourful images: a little more colour detail than the default (precision 8,
    /// small noise ignored). The gain is small, and a smooth gradient is flattened by every preset.
    PhotoRich,
}

impl VtracerPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sharp => "Sharp (logos, icons)",
            Self::Default => "Default (balanced)",
            Self::PhotoRich => "Photo-rich (gradients)",
        }
    }

    /// All variants in order, for pick_list display.
    pub fn all() -> [Self; 3] {
        [Self::Sharp, Self::Default, Self::PhotoRich]
    }
}

/// `Display` implementation required by iced's `pick_list` widget (`T: ToString`).
impl std::fmt::Display for VtracerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
