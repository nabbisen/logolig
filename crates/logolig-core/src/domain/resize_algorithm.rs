//! Resize algorithm (§6.2).
//! Default is quality-first (Lanczos3).

use std::fmt;

use fast_image_resize::{FilterType, ResizeAlg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ResizeAlgorithm {
    #[default]
    Lanczos3,
    MitchellNetravali,
    CatmullRom,
    Bilinear,
    /// Pixel-art / hard-edge sources (no interpolation).
    Nearest,
}

impl ResizeAlgorithm {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lanczos3 => "Lanczos3 (default, high quality)",
            Self::MitchellNetravali => "Mitchell–Netravali",
            Self::CatmullRom => "Catmull–Rom",
            Self::Bilinear => "Bilinear (fast)",
            Self::Nearest => "Nearest (pixel art)",
        }
    }

    /// Ordered list for cycling through options in the settings UI.
    pub fn all() -> [Self; 5] {
        [
            Self::Lanczos3,
            Self::MitchellNetravali,
            Self::CatmullRom,
            Self::Bilinear,
            Self::Nearest,
        ]
    }

    /// Convert to `fast_image_resize`'s `ResizeAlg`.
    ///
    /// Returns `ResizeAlg` rather than `FilterType` because
    /// `Nearest` is a standalone variant (`ResizeAlg::Nearest`)
    /// rather than a convolution filter.
    pub fn to_resize_alg(self) -> ResizeAlg {
        match self {
            Self::Lanczos3 => ResizeAlg::Convolution(FilterType::Lanczos3),
            Self::MitchellNetravali => ResizeAlg::Convolution(FilterType::Mitchell),
            Self::CatmullRom => ResizeAlg::Convolution(FilterType::CatmullRom),
            Self::Bilinear => ResizeAlg::Convolution(FilterType::Bilinear),
            Self::Nearest => ResizeAlg::Nearest,
        }
    }
}

/// `Display` implementation required by iced's `pick_list` widget (`T: ToString`).
/// Returns the same string as `label()`.
impl fmt::Display for ResizeAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
