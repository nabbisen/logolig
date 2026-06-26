//! Export plan — the complete output specification.
//!
//! `ExportPlan` is a pure value type: it describes *what* to export but
//! contains no I/O logic. The exporter service reads this struct and
//! produces the corresponding artifacts.
//!
//! `ExportPlan` is also the persisted user preference: it is serialised
//! into `PersistedSettings` and saved on every change.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::resize_algorithm::ResizeAlgorithm;
use crate::domain::vtracer_preset::VtracerPreset;

/// Minimum practical PNG size in pixels (below this rendering degrades).
pub const PNG_SIZE_MIN: u32 = 16;
/// Maximum practical PNG size in pixels (above this the file size is excessive for favicons).
pub const PNG_SIZE_MAX: u32 = 1024;

/// Minimum practical ICO frame size in pixels.
pub const ICO_SIZE_MIN: u32 = 16;
/// Maximum practical ICO frame size in pixels.
///
/// 256 is the ICO format maximum (the BMP-mode dimension field is
/// `u8`; 256 is encoded as 0 by convention). The `ico` crate respects this.
pub const ICO_SIZE_MAX: u32 = 256;

/// Per-size source image override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeOverride {
    pub size: u32,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ExportPlan {
    pub include_ico: bool,
    pub include_apple_touch: bool,
    /// PNG sizes to write as individual favicon-NN.png files.
    pub png_sizes: Vec<u32>,
    pub include_html_snippet: bool,
    pub algorithm: ResizeAlgorithm,
    pub overrides: Vec<SizeOverride>,
    /// Frame sizes to include in the ICO file.
    pub ico_sizes: Vec<u32>,
    /// Whether to write `favicon.svg` (added in v1.2.0).
    /// SVG source: copied as-is. Raster source: traced only when `vectorize_on_raster` is true.
    pub include_svg: bool,
    /// Whether to trace a raster source (PNG/WebP) to SVG using vtracer (v1.2.0+).
    /// When `false`, SVG output is skipped for raster sources
    /// (and the `<link type="image/svg+xml">` line is omitted from the HTML snippet).
    pub vectorize_on_raster: bool,
    /// vtracer tuning preset (v1.4.1+).
    /// `Sharp` for logos/icons; `Default` matches the v1.2.0 behaviour;
    /// `PhotoRich` for photo-like or gradient sources.
    pub vtracer_preset: VtracerPreset,

    /// Web manifest output (v1.8.0+). When `Some(_)`, writes `manifest.webmanifest`
    /// and appends `<link rel="manifest">` to the HTML snippet.
    /// `None` (default): no manifest-related output.
    ///
    /// `Some(WebManifestSettings::default())` enables it with placeholder values
    /// ("My App" etc.); users should fill in real values via the Customize page
    /// `name`, `short_name`, `theme_color`, `background_color` should be filled in.
    pub web_manifest: Option<crate::domain::WebManifestSettings>,

    /// Monochrome output set (v1.9.0+). When `true`, writes greyscale variants
    /// into `mono/` alongside the normal outputs:
    ///
    /// ```text
    /// mono/
    /// ├── favicon.svg     (if include_svg is true)
    /// ├── favicon.ico     (if include_ico is true)
    /// └── favicon-{N}.png (for each png_size)
    /// ```
    ///
    /// `apple-touch-icon` is not monochromed (iOS home screen assumes colour).
    /// No mono `<link>` is added to the HTML snippet (CSS `prefers-color-scheme`
    /// integration varies; left to the user).
    pub monochrome: bool,

    /// Whether to preserve the alpha channel (v1.21.0).
    ///
    /// - `true` (default): preserve alpha as-is. PNG/ICO outputs retain
    ///   transparent areas. The modern favicon standard.
    /// - `false`: composite alpha against **white**, making every pixel opaque.
    ///   SVG outputs are unaffected (flatten is a raster operation).

    ///   JPEG sources have alpha=255 already; results are the same either way.
    ///
    /// Persisted. Old settings JSON without this field gets `true` via
    /// the struct-level `#[serde(default)]` (= `ExportPlan::default()`).
    /// Existing users see no behaviour change after upgrade.
    pub keep_transparency: bool,

    /// Whether to generate the minimal Microsoft app logo set (v1.26.0).
    ///
    /// When enabled, the exporter writes four PNG files at the output root:
    /// `StoreLogo.png`, `Square44x44Logo.png`, `Square150x150Logo.png`,
    /// and `Wide310x150Logo.png`. This is intentionally an advanced,
    /// opt-in setting rather than part of the default favicon bundle.
    pub include_microsoft_app_logos: bool,
}

impl Default for ExportPlan {
    fn default() -> Self {
        Self {
            include_ico: true,
            include_apple_touch: true,
            // Minimal modern set. Users can add more in the Customize page.
            png_sizes: Self::default_png_sizes().to_vec(),
            include_html_snippet: true,
            algorithm: ResizeAlgorithm::default(),
            overrides: Vec::new(),
            // ICO contains 16/32/48, each rendered independently.
            ico_sizes: Self::default_ico_sizes().to_vec(),
            // SVG output: v1.2.0 default. Best quality on HiDPI screens.
            include_svg: true,
            // Vectorise raster sources by default.
            // Users can disable in Advanced for photos that vtracer handles poorly.
            vectorize_on_raster: true,
            // vtracer preset: Default (compatible with v1.2.0).
            // Sharp / PhotoRich are opt-in via the Customize page.
            vtracer_preset: VtracerPreset::Default,
            // v1.8.0: web manifest is opt-in (None default).
            // §5 "reduce decision fatigue": most users are not building a PWA,
            // so no manifest is written without an explicit opt-in.
            web_manifest: None,
            // v1.9.0: Monochrome output is opt-in (false default).
            // Most users only need the colour version.
            monochrome: false,
            // v1.21.0: Transparency is preserved by default (modern favicon standard + backward compat).
            keep_transparency: true,
            // v1.26.0: Microsoft app logos are advanced/opt-in, not default favicon output.
            include_microsoft_app_logos: false,
        }
    }
}

impl ExportPlan {
    /// Default PNG size set. v1.10.0 uses this to compare "current == default"
    /// when toggling between the compact "at defaults: 32/192/512" view
    /// and edit mode.
    pub fn default_png_sizes() -> &'static [u32] {
        &[32, 192, 512]
    }

    /// Default ICO size set.
    pub fn default_ico_sizes() -> &'static [u32] {
        &[16, 32, 48]
    }

    /// **Maximum** artifact count for this plan (used for display).
    ///
    /// Note: `include_svg` may not produce SVG when `vectorize_on_raster=false`
    /// and source is raster, so this returns the upper bound.
    /// Actual count is `services::exporter::run`'s `ExportReport.artifacts.len()`.

    pub fn artifact_count(&self) -> usize {
        let ico = usize::from(self.include_ico);
        let apple = usize::from(self.include_apple_touch);
        let html = usize::from(self.include_html_snippet);
        let svg = usize::from(self.include_svg);
        // v1.8.0: +1 for manifest.webmanifest when web_manifest is Some.
        let manifest = usize::from(self.web_manifest.is_some());
        let microsoft_app_logos = if self.include_microsoft_app_logos {
            crate::domain::MICROSOFT_APP_LOGOS.len()
        } else {
            0
        };
        let base = ico + apple + html + svg + manifest + self.png_sizes.len() + microsoft_app_logos;
        // v1.9.0: when monochrome is on, add greyscale PNG + SVG (if include_svg)
        //          + ICO (if include_ico). apple-touch / html / manifest excluded.

        let mono = if self.monochrome {
            self.png_sizes.len() + usize::from(self.include_svg) + usize::from(self.include_ico)
        } else {
            0
        };
        base + mono
    }

    /// Add a PNG size (v1.3.0). Rejects duplicates and out-of-range values; keeps ascending order.
    /// Returns `true` if added, `false` if it already existed or is out of range.
    pub fn add_png_size(&mut self, size: u32) -> bool {
        Self::add_into_sorted_set(&mut self.png_sizes, size, PNG_SIZE_MIN, PNG_SIZE_MAX)
    }

    /// Remove a PNG size (v1.3.0). Returns `true` if removed.
    pub fn remove_png_size(&mut self, size: u32) -> bool {
        Self::remove_from_set(&mut self.png_sizes, size)
    }

    /// Add an ICO frame size (v1.3.0).
    pub fn add_ico_size(&mut self, size: u32) -> bool {
        Self::add_into_sorted_set(&mut self.ico_sizes, size, ICO_SIZE_MIN, ICO_SIZE_MAX)
    }

    /// Remove an ICO frame size (v1.3.0).
    /// Emptying `ico_sizes` is allowed (equivalent to `include_ico=false`).
    pub fn remove_ico_size(&mut self, size: u32) -> bool {
        Self::remove_from_set(&mut self.ico_sizes, size)
    }

    fn add_into_sorted_set(set: &mut Vec<u32>, size: u32, min: u32, max: u32) -> bool {
        if size < min || size > max {
            return false;
        }
        if set.contains(&size) {
            return false;
        }
        set.push(size);
        set.sort_unstable();
        true
    }

    fn remove_from_set(set: &mut Vec<u32>, size: u32) -> bool {
        if let Some(pos) = set.iter().position(|s| *s == size) {
            set.remove(pos);
            true
        } else {
            false
        }
    }
}
