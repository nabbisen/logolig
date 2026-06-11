//! Decode a JPEG source into an RGBA8 bitmap (v1.11.0).
//!
//! Enabling the `jpeg` feature on the `image` crate pulls in `zune-jpeg`,
//! which handles both Baseline and Progressive JPEG.
//!
//! ## Why support JPEG?
//!
//! JPEG is unsuitable for favicons (no transparency, lossy compression
//! that blurs fine detail), but users often only have their logo as a
//! JPEG (scanned logo, PowerPoint export, etc.).
//!
//! Rejecting JPEG outright would produce an unhelpful "unsupported file"
//! error. Instead, logolig accepts JPEG, decodes it, and shows an
//! educational toast warning about the transparency limitation.
//!
//! ## Alpha channel
//!
//! JPEG has no alpha channel. `image::load_from_memory` converts the
//! RGB data to RGBA8 by setting every alpha byte to 255. The resulting
//! `TransparencyReport` will always be `FullyOpaque`, triggering the
//! transparency warning toast.

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// Decode a JPEG source to RGBA8.
///
/// Returns `Err(UnsupportedFile)` if the input is not JPEG.
pub fn decode(asset: &SourceAsset) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Jpeg {
        return Err(AppError::unsupported_file(format!(
            "decode_jpeg called on non-JPEG source ({})",
            asset.kind.label()
        )));
    }

    let img = image::load_from_memory(&asset.raw)
        .map_err(|e| AppError::decode(format!("JPEG decode: {e}")))?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    Rgba8::try_from_raw(w, h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::decode("internal: rgba length mismatch"))
}
