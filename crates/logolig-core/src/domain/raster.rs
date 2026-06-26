//! Immutable raster result type.
//!
//! Embodies §6.4 "treat conversion results as internal artifacts".
//! `Arc<[u8]>` makes cloning cheap; no buffer copies when handing to the UI thread.

use std::sync::Arc;

/// RGBA8 pixel buffer. Holds `width * height * 4` bytes.
///
/// Written by the service layer; the UI layer only reads, never modifies.
#[derive(Debug, Clone)]
pub struct Rgba8 {
    pub width: u32,
    pub height: u32,
    /// RGBA bytes, 4 per pixel, stride = `width * 4`.
    pub pixels: Arc<[u8]>,
}

impl Rgba8 {
    /// Construct from raw bytes, validating `len == width * height * 4`.
    /// Returns `None` on violation; callers convert to a Decode/Resize error.
    pub fn try_from_raw(width: u32, height: u32, pixels: Arc<[u8]>) -> Option<Self> {
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() == expected {
            Some(Self {
                width,
                height,
                pixels,
            })
        } else {
            None
        }
    }

    /// View the pixel buffer as a byte slice (for tests and serialisation).
    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }
}
