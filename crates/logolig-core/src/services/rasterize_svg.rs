//! SVG ラスタライザ。
//!
//! Step 2 で `resvg` + `tiny-skia` を使って実装する。
//! ターゲットサイズに対して **個別に** レンダリングし、
//! 「大きいサイズで描いて縮小」由来のジャギーを避ける (§6.2)。

use crate::domain::SourceAsset;
use crate::error::AppError;

/// SVG ソースを指定サイズの RGBA8 ビットマップ (`width * height * 4`) に展開する。
///
/// 入力がそもそも SVG でない場合は `Err(UnsupportedFile)`。
pub fn rasterize(_asset: &SourceAsset, _size: u32) -> Result<Vec<u8>, AppError> {
    Err(AppError::NotImplemented(
        "logolig_core::services::rasterize_svg",
    ))
}
