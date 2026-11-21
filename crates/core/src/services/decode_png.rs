//! Decode a PNG source to an RGBA8 bitmap.
//!
//! Per spec §6.4 "treat results as internal artifacts / non-destructive workflow":
//! `SourceAsset.raw` is read-only here; the decode result is a new `Rgba8`.
//!
//! Uses the `image` crate with only the `"png"` feature enabled
//! to keep unused format code out of the binary (see Cargo.toml).

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// Decode a PNG source to RGBA8.
///
/// Returns `Err(UnsupportedFile)` if the input is not PNG.
pub fn decode(asset: &SourceAsset) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Png {
        return Err(AppError::unsupported_file(format!(
            "decode_png called on non-PNG source ({})",
            asset.kind.label()
        )));
    }

    // The format is guessed from the bytes, never from the extension, and the decoder runs with
    // limits (RFC 014).
    let img = crate::services::limited_decode::load(asset, "PNG")?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    Rgba8::try_from_raw(w, h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::decode("internal: rgba length mismatch"))
}
