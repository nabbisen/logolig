//! Pack multiple RGBA8 rasters into a single `.ico` byte stream.
//!
//! Key design points (§7.1):
//! - **Each frame is pre-rendered by the caller** (`exporter::run`).
//!   The ICO writer packs frames it receives; it never downsamples.
//! - `IconDirEntry::encode()` auto-selects BMP (≤48 px) or PNG (≥256 px)
//!   per frame, maximising backward compatibility.

use std::io::Cursor;

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

use crate::domain::Rgba8;
use crate::error::AppError;

/// Pack a list of frames into a single ICO byte stream.
///
/// `frames` is a list of (size, RGBA8) pairs. Returns `Err` if a declared
/// size does not match the RGBA8 dimensions (guards against mis-assembly).
pub fn build(frames: &[(u32, &Rgba8)]) -> Result<Vec<u8>, AppError> {
    if frames.is_empty() {
        return Err(AppError::export("ICO requires at least one frame"));
    }

    let mut dir = IconDir::new(ResourceType::Icon);
    for (size, rgba) in frames {
        if rgba.width != *size || rgba.height != *size {
            return Err(AppError::export(format!(
                "ICO frame size mismatch: requested {size}, got {}x{}",
                rgba.width, rgba.height
            )));
        }
        // ico 0.4 requires an owned buffer (Vec<u8>).
        let pixels = rgba.as_bytes().to_vec();
        let image = IconImage::from_rgba_data(rgba.width, rgba.height, pixels);
        let entry = IconDirEntry::encode(&image)
            .map_err(|e| AppError::export(format!("ICO entry encode at {size}px: {e}")))?;
        dir.add_entry(entry);
    }

    let mut out = Vec::new();
    dir.write(Cursor::new(&mut out))
        .map_err(|e| AppError::export(format!("ICO write: {e}")))?;
    Ok(out)
}
