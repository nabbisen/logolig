//! PNG ソースを RGBA8 ビットマップに展開する。
//!
//! 仕様 §6.4「変換結果は内部生成物として扱う」「再生成可能なワークフロー」のため、
//! ここでも `SourceAsset.raw` は読むだけ。デコード結果は新しい `Rgba8` を返す。
//!
//! `image` クレートを使い、 features = ["png"] のみ有効化することで
//! 不要なフォーマットのコードがバイナリに入らないようにしている (Cargo.toml)。

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// PNG ソースを RGBA8 にデコード。
///
/// 入力が PNG でなければ `Err(UnsupportedFile)`。
pub fn decode(asset: &SourceAsset) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Png {
        return Err(AppError::unsupported_file(format!(
            "decode_png called on non-PNG source ({})",
            asset.kind.label()
        )));
    }

    // image クレートの load_from_memory は extension を見ず Magic で判定するため安全。
    let img = image::load_from_memory(&asset.raw)
        .map_err(|e| AppError::decode(format!("PNG decode: {e}")))?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<u8> = rgba.into_raw();

    Rgba8::try_from_raw(w, h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::decode("internal: rgba length mismatch"))
}
