//! Raster-to-SVG vectorisation (v1.2.0; presets added in v1.4.1).
//!
//! Wraps `vtracer` 0.6 to convert an `Rgba8` bitmap into an SVG string.
//!
//! ## Design decisions
//!
//! - **Input capped at `VECTORIZE_MAX_SIDE`** (RFC 030) — a raster whose longest side
//!   is longer is scaled down, aspect ratio kept, before tracing: the tracer's cost grows
//!   faster than the pixel count (a 1400 px photograph took 10 s and gave a 25 MB SVG),
//!   while the SVG is resolution independent. Below the cap the source is traced as is.
//!   Raster outputs never go through this: they are rendered from the full source.
//! - **Three presets** control behaviour (`VtracerPreset`). Raw vtracer
//!   parameters are not exposed to the user (§5).
//! - **Failures map to `AppError::export(_)`**. vtracer returns
//!   `Result<_, String>`; this module normalises that into `AppError`.
//! - **CPU-bound**: the caller runs it on a blocking thread (the application uses
//!   `spawn_blocking`, RFC 030), never on the async executor.

use vtracer::{ColorImage, ColorMode, Config, Hierarchical};

use crate::domain::{ResizeAlgorithm, Rgba8, VtracerPreset};
use crate::error::AppError;
use crate::services::resize;

/// Longest side, in pixels, of the raster handed to the tracer (RFC 030).
///
/// 512 px is the largest default favicon size; the cost at this size is under a second even
/// for noise, and the SVG stays near 3 MB at worst.
pub const VECTORIZE_MAX_SIDE: u32 = 512;

/// Vectorise an `Rgba8` bitmap (capped at [`VECTORIZE_MAX_SIDE`], resized with Lanczos3
/// when it has to be) and return an SVG string.
pub fn vectorize(rgba: &Rgba8, preset: VtracerPreset) -> Result<String, AppError> {
    vectorize_with_cap(rgba, preset, VECTORIZE_MAX_SIDE, ResizeAlgorithm::Lanczos3)
}

/// Size a `width` x `height` raster is traced at: unchanged when its longest side is within
/// `max_side`, otherwise scaled uniformly (aspect ratio kept, each side at least 1) so that the
/// longest side is `max_side`. A `max_side` of 0 is treated as 1.
pub fn trace_dimensions(width: u32, height: u32, max_side: u32) -> (u32, u32) {
    let max_side = max_side.max(1);
    let longest = width.max(height);
    if longest <= max_side {
        return (width, height);
    }
    // Rounded integer scaling: side * max_side / longest, never below 1.
    let scale = |side: u32| -> u32 {
        let scaled =
            (u64::from(side) * u64::from(max_side) + u64::from(longest) / 2) / u64::from(longest);
        u32::try_from(scaled).unwrap_or(max_side).max(1)
    };
    (scale(width), scale(height))
}

/// Vectorise with an explicit cap on the traced size; `algorithm` resizes when the raster is
/// longer than `max_side`. A zero-sized raster or a `max_side` of 0 is an error.
pub fn vectorize_with_cap(
    rgba: &Rgba8,
    preset: VtracerPreset,
    max_side: u32,
    algorithm: ResizeAlgorithm,
) -> Result<String, AppError> {
    if rgba.width == 0 || rgba.height == 0 {
        return Err(AppError::export("vectorize: zero-sized raster"));
    }
    if max_side == 0 {
        return Err(AppError::export(
            "vectorize: the size cap must be at least 1",
        ));
    }

    let (width, height) = trace_dimensions(rgba.width, rgba.height, max_side);
    if (width, height) == (rgba.width, rgba.height) {
        trace(rgba, preset)
    } else {
        let scaled = resize::resize(rgba, width, height, algorithm)?;
        trace(&scaled, preset)
    }
}

/// Run the tracer on `rgba` as it is.
fn trace(rgba: &Rgba8, preset: VtracerPreset) -> Result<String, AppError> {
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
