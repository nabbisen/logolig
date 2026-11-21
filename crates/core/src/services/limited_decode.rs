//! The one way raster sources are decoded (RFC 014): through `image`'s decoder **limits**.
//!
//! `ingest` refuses an image whose header declares too many pixels. This is the second line: when a
//! header cannot be parsed (a crafted or damaged file), the decoder still cannot be made to
//! allocate without bound, because every decode runs with a maximum width, height and allocation.

use std::io::Cursor;

use image::{DynamicImage, ImageError, ImageReader, Limits};

use crate::domain::SourceAsset;
use crate::error::AppError;
use crate::services::ingest::MAX_SIDE;

/// The most a decoder may allocate for one image: 1 GiB (a 64-megapixel image with 16 bits per
/// channel and alpha is 0.5 GiB).
const MAX_DECODE_ALLOC: u64 = 1024 * 1024 * 1024;

/// Decodes `asset` (the format is guessed from the bytes, never from the file name) within the
/// limits. `what` names the format in the message of an ordinary decode failure ("PNG").
pub(crate) fn load(asset: &SourceAsset, what: &str) -> Result<DynamicImage, AppError> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    let mut reader = ImageReader::new(Cursor::new(&asset.raw[..]));
    reader.limits(limits);
    let reader = reader
        .with_guessed_format()
        .map_err(|e| AppError::decode(format!("{what} decode: {e}")))?;
    reader.decode().map_err(|e| match e {
        // Over a limit: the same message as an image that ingest refused, by name.
        ImageError::Limits(_) => AppError::image_too_large(
            asset.display_name(),
            asset
                .intrinsic_size
                .map_or(0, |(w, h)| u64::from(w) * u64::from(h)),
        ),
        other => AppError::decode(format!("{what} decode: {other}")),
    })
}
