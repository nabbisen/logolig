//! Immutable representation of the loaded source image (§6.4 non-destructive).

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Png,
    Svg,
    /// Static WebP (v1.1.0+). Animated WebP: first frame only.
    Webp,
    /// JPEG (v1.11.0+). Cannot represent transparency; an educational
    /// warning toast is shown after loading to encourage converting to PNG
    /// (see `push_jpeg_warning`).
    Jpeg,
}

impl SourceKind {
    /// Inferred from file extension. Returns `None` if unrecognised.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "webp" => Some(Self::Webp),
            // JPEG uses both `.jpg` and `.jpeg` — both are in common use.
            "jpg" | "jpeg" => Some(Self::Jpeg),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Svg => "SVG",
            Self::Webp => "WebP",
            Self::Jpeg => "JPEG",
        }
    }
}

/// A loaded source image.
///
/// Designed to be carried across threads via `Message`.
/// Internal buffer uses `Arc<[u8]>` for cheap cloning and immutability.
#[derive(Debug, Clone)]
pub struct SourceAsset {
    /// Original file path. Used for display and re-ingestion.
    pub path: PathBuf,
    /// Detected source kind.
    pub kind: SourceKind,
    /// Raw data (pre-decode bytes for PNG; UTF-8 source for SVG).
    pub raw: Arc<[u8]>,
    /// PNG: actual pixel dimensions. SVG: size hint from viewBox.
    pub intrinsic_size: Option<(u32, u32)>,
}

impl SourceAsset {
    pub fn display_name(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unnamed>".into())
    }
}
