//! In-memory conversion result bundle (v1.16.0).
//!
//! ## v1.16.0 model change
//!
//! Old model (≤ v1.15):
//!   file dropped → Preview confirmed → directory chosen → bulk write
//!
//! New model (v1.16+):
//!   file dropped → auto-convert (in memory) → Result screen
//!   → individual download or ZIP download
//!
//! On the Result screen each asset is shown as a card. The user can:
//! - Click a per-card download button to save a single file via a dialog
//! - Click "Download all (ZIP)" to bundle everything into a zip

#![allow(dead_code)]

use logolig::Rgba8;

/// Full set of conversion results held in memory.
#[derive(Debug, Clone)]
pub struct ResultAssets {
    /// Individual assets in display order.
    pub items: Vec<ResultAssetItem>,
}

impl ResultAssets {
    /// Total byte size of all assets (for UI display).
    pub fn total_bytes(&self) -> usize {
        self.items.iter().map(|i| i.bytes.len()).sum()
    }

    /// Asset count.
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// One asset item.
#[derive(Debug, Clone)]
pub struct ResultAssetItem {
    /// Output file name (e.g. `favicon.ico`, `favicon-16.png`, `manifest.webmanifest`).
    pub file_name: String,
    /// Raw output bytes (used for individual download and ZIP bundle).
    pub bytes: Vec<u8>,
    /// Kind, used to decide how to render the card thumbnail.
    pub kind: ResultAssetKind,
    /// Image dimensions (e.g. `(16, 16)`). `None` for non-image assets.
    pub dimensions: Option<(u32, u32)>,
    /// Pre-decoded small raster for the card thumbnail.
    /// `Some` for image assets; `None` for text assets (snippet, manifest).
    pub thumbnail: Option<Rgba8>,
}

impl ResultAssetItem {
    /// Human-readable size string (e.g. "46 KB", "1.2 KB").
    pub fn size_display(&self) -> String {
        let bytes = self.bytes.len();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 100 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 {
            format!("{} KB", bytes / 1024)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// Dimensions as a display string. Meaningful for image assets only.
    pub fn dimensions_display(&self) -> Option<String> {
        self.dimensions.map(|(w, h)| format!("{} × {}", w, h))
    }
}

/// Asset kind.
///
/// Determines whether the card thumbnail shows the actual raster image
/// or a document icon placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultAssetKind {
    /// PNG (per-size favicon-NN.png, apple-touch-icon.png)
    Png,
    /// ICO (favicon.ico, may contain multiple sizes)
    Ico,
    /// SVG (favicon.svg — source copy or vtracer output)
    Svg,
    /// PNG monochrome (files under the mono/ subdirectory)
    PngMono,
    /// HTML snippet file (favicon-snippet.html)
    HtmlSnippet,
    /// Web manifest (manifest.webmanifest)
    WebManifest,
}

impl ResultAssetKind {
    /// Short label shown on the card badge.
    pub fn badge_label(self) -> &'static str {
        match self {
            ResultAssetKind::Png => "PNG",
            ResultAssetKind::Ico => "ICO",
            ResultAssetKind::Svg => "SVG",
            ResultAssetKind::PngMono => "PNG mono",
            ResultAssetKind::HtmlSnippet => "HTML",
            ResultAssetKind::WebManifest => "JSON",
        }
    }

    /// Whether a raster thumbnail can be rendered for this kind.
    pub fn has_visual_thumbnail(self) -> bool {
        matches!(
            self,
            ResultAssetKind::Png
                | ResultAssetKind::Ico
                | ResultAssetKind::Svg
                | ResultAssetKind::PngMono
        )
    }
}
