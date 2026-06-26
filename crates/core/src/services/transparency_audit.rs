//! Transparency auditor (v1.7.0).
//!
//! Analyses the alpha channel of a loaded image and reports cases where
//! the transparency state is likely unintentional. The result is shown
//! as a one-time informational toast right after the file is loaded.
//!
//! ## Detected cases
//!
//! - **`FullyOpaque`** — every pixel has alpha=255. The favicon appears
//!   as "logo inside a white square" on dark browser tabs.
//! - **`FullyTransparent`** — every pixel has alpha=0. The image is
//!   invisible; the user likely dropped the wrong file.
//! - **`HasTransparency`** — mix of opaque and transparent pixels.
//!   This is the expected state for a proper favicon source.
//!
//! `needs_warning()` returns `true` for `FullyOpaque` and
//! `FullyTransparent` only.

use crate::domain::Rgba8;

/// Classification of an image's alpha-channel state.
///
/// Three cases relevant to favicon use. There is no "Indeterminate" state —
/// `audit()` always returns one of these three.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransparencyReport {
    /// Fully opaque (every pixel has alpha=255). Exporting as a transparent PNG serves no purpose.
    /// On dark browser tabs this produces the classic "logo in a white box" mistake.
    FullyOpaque,
    /// Fully transparent (every pixel has alpha=0). Likely an empty layer or wrong file.
    FullyTransparent,
    /// Mix of transparent and opaque pixels — the expected state for a favicon.
    HasTransparency,
}

impl TransparencyReport {
    /// Whether to show a warning toast (used by the UI layer).
    /// `HasTransparency` never triggers a warning.
    pub fn needs_warning(self) -> bool {
        matches!(self, Self::FullyOpaque | Self::FullyTransparent)
    }
}

/// Analyse the transparency state of an image.
///
/// Scans all pixels to find the min and max alpha, then classifies into 3 cases.
/// An empty image (width=0 or height=0) returns `FullyTransparent` conservatively.
pub fn audit(image: &Rgba8) -> TransparencyReport {
    if image.width == 0 || image.height == 0 || image.pixels.is_empty() {
        // Zero pixels → equivalent to "nothing drawn"
        return TransparencyReport::FullyTransparent;
    }

    // RGBA: 4 bytes per pixel; alpha is the 4th byte.
    debug_assert!(
        image.pixels.len() == (image.width as usize) * (image.height as usize) * 4,
        "Rgba8 pixel buffer length mismatch"
    );

    let mut min_alpha: u8 = 255;
    let mut max_alpha: u8 = 0;
    for chunk in image.pixels.chunks_exact(4) {
        // chunk[3] is the alpha byte
        let a = chunk[3];
        if a < min_alpha {
            min_alpha = a;
        }
        if a > max_alpha {
            max_alpha = a;
        }
        // Early exit: once we confirm a mix, no further scanning is needed
        if min_alpha == 0 && max_alpha == 255 {
            return TransparencyReport::HasTransparency;
        }
    }

    match (min_alpha, max_alpha) {
        (255, 255) => TransparencyReport::FullyOpaque,
        (0, 0) => TransparencyReport::FullyTransparent,
        // Other cases (e.g. uniformly half-transparent alpha=128)
        // are not "mixed" but are harmless for favicons. No warning;
        // classify as HasTransparency. May be revisited in v1.7.x.
        _ => TransparencyReport::HasTransparency,
    }
}
