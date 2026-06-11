//! Decode a WebP source into an RGBA8 bitmap (v1.1.0).
//!
//! Enabling the `webp` feature on the `image` crate pulls in `image-webp`,
//! which handles static WebP (VP8 / VP8L / VP8X). Animated WebP is
//! supported by extracting the first frame only.
//!
//! The implementation pattern intentionally mirrors `decode_png.rs`.
//! Format dispatch (which decoder to call) is handled by the caller
//! (`exporter::render_at_size`, `preview::render_at`), keeping this
//! module focused on a single format.

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// Decode a WebP source to RGBA8.
///
/// Returns `Err(UnsupportedFile)` if the input is not WebP.
pub fn decode(asset: &SourceAsset) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Webp {
        return Err(AppError::unsupported_file(format!(
            "decode_webp called on non-WebP source ({})",
            asset.kind.label()
        )));
    }

    let img = image::load_from_memory(&asset.raw)
        .map_err(|e| AppError::decode(format!("WebP decode: {e}")))?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    Rgba8::try_from_raw(w, h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::decode("internal: rgba length mismatch"))
}
