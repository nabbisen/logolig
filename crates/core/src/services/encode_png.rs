//! Encode RGBA8 to PNG bytes.
//!
//! Uses `image`'s `PngEncoder` with the `Default` compression preset —
//! favicon-sized images gain little from aggressive compression.

use std::io::Cursor;

use image::ColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;

use crate::domain::Rgba8;
use crate::error::AppError;

/// Encode `Rgba8` to PNG bytes.
pub fn encode(rgba: &Rgba8) -> Result<Vec<u8>, AppError> {
    let mut out = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut out));
    encoder
        .write_image(
            rgba.as_bytes(),
            rgba.width,
            rgba.height,
            ColorType::Rgba8.into(),
        )
        .map_err(|e| AppError::export(format!("PNG encode: {e}")))?;
    Ok(out)
}
