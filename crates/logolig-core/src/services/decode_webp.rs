//! WebP ソースを RGBA8 ビットマップに展開する (v1.1.0)。
//!
//! `image` クレートに `webp` feature を有効化することで `image-webp` 経由で
//! 静的 WebP (VP8 / VP8L / VP8X) のいずれもデコード可能になる。
//! アニメーション WebP は最初のフレームのみが取り出される。
//!
//! 実装パターンは `decode_png.rs` と意図的に揃えている。 入力種別ごとの
//! 分岐を上位 (`exporter::render_at_size`, `preview::render_at`) でだけ
//! 行えるようにするため。

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// WebP ソースを RGBA8 にデコード。
///
/// 入力が WebP でなければ `Err(UnsupportedFile)`。
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
