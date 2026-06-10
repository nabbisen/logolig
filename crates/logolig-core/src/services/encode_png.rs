//! RGBA8 を PNG バイト列にエンコードする。
//!
//! `image` クレートの `PngEncoder` を使用。 圧縮プリセットは `Default`
//! (Rust の bindgen が選ぶ穏当な圧縮率) — favicon サイズの画像なので
//! 強圧縮しても得は小さく、 高速・小サイズのバランスを優先。

use std::io::Cursor;

use image::ColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;

use crate::domain::Rgba8;
use crate::error::AppError;

/// `Rgba8` を PNG バイト列にエンコードして返す。
pub fn encode(rgba: &Rgba8) -> Result<Vec<u8>, AppError> {
    let mut out = Vec::new();
    let encoder = PngEncoder::new(Cursor::new(&mut out));
    encoder
        .write_image(rgba.as_bytes(), rgba.width, rgba.height, ColorType::Rgba8.into())
        .map_err(|e| AppError::Export(format!("PNG encode: {e}")))?;
    Ok(out)
}
