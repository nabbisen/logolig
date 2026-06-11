//! Raster-to-SVG vectorisation (v1.2.0; presets added in v1.4.1).
//!
//! Wraps `vtracer` 0.6 to convert an `Rgba8` bitmap into an SVG string.
//!
//! ## Design decisions
//!
//! - **Input at source resolution** — no resize before vectorisation.
//!   Preserving original detail produces better contour extraction.
//! - **Three presets** control behaviour (`VtracerPreset`). Raw vtracer
//!   parameters are not exposed to the user (§5).
//! - **Failures map to `AppError::export(_)`**. vtracer returns
//!   `Result<_, String>`; this module normalises that into `AppError`.
//! - **CPU-bound**: the caller dispatches via `iced::Task::perform`.

use vtracer::{ColorImage, ColorMode, Config, Hierarchical};

use crate::domain::{Rgba8, VtracerPreset};
use crate::error::AppError;

/// Vectorise an `Rgba8` bitmap and return an SVG string.
/// Assembles a vtracer `Config` for the given preset.
pub fn vectorize(rgba: &Rgba8, preset: VtracerPreset) -> Result<String, AppError> {
    if rgba.width == 0 || rgba.height == 0 {
        return Err(AppError::export("vectorize: zero-sized raster"));
    }

    // ColorImage is RGBA 4 bytes/pixel — fully compatible with Rgba8.
    // pixels takes ownership, so clone once.
    let color_image = ColorImage {
        pixels: rgba.as_bytes().to_vec(),
        width: rgba.width as usize,
        height: rgba.height as usize,
    };

    let config = config_for(preset);

    let svg_file = vtracer::convert(color_image, config)
        .map_err(|e| AppError::export(format!("vtracer: {e}")))?;

    // SvgFile yields the SVG string via its Display impl.
    Ok(format!("{svg_file}"))
}

/// Build a vtracer `Config` for the given preset.
///
/// Raw `Config` editing is not exposed to users (§5 "reduce decision fatigue").
/// The preset→Config mapping is centralised here. A future `Custom { … }` variant
/// could be added without changing call sites.
///
/// ## v1.4.2 Sharp calibration approach
///
/// v1.4.1 changed 4 parameters simultaneously (including `filter_speckle=2`,
/// `path_precision=Some(3)`). Testing found that `filter_speckle=2` and
/// `path_precision=3` likely degraded logo contour quality.

///
/// v1.4.2 reduces the diff to a single parameter (`corner_threshold`)
/// so the effect of "no corner rounding" can be observed in isolation.
/// Evidence-based preset tuning: one parameter change = clear causal attribution.
fn config_for(preset: VtracerPreset) -> Config {
    match preset {
        VtracerPreset::Sharp => Config {
            // Logo preset: no corner rounding. All other params at default.
            // v1.4.1 included filter_speckle=2 / path_precision=3, but they
            // likely degraded logo quality and were removed in v1.4.2.
            corner_threshold: 80, // default 60 → 80
            ..Config::default()
        },
        VtracerPreset::Default => {
            // Exact v1.2.0 behaviour (vtracer defaults respected)
            Config::default()
        }
        VtracerPreset::PhotoRich => Config {
            // Photo preset: fine colour gradations preserved; small noise ignored
            color_precision: 8, // maximum precision
            filter_speckle: 8,  // ignore small noise
            corner_threshold: 45,
            hierarchical: Hierarchical::Stacked,
            color_mode: ColorMode::Color,
            ..Config::default()
        },
    }
}
