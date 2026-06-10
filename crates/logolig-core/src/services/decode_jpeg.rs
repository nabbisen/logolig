//! JPEG ソースを RGBA8 ビットマップに展開する (v1.11.0)。
//!
//! `image` クレートに `jpeg` feature を有効化することで `zune-jpeg` 経由で
//! JPEG (Baseline / Progressive 双方) をデコード可能になる。
//!
//! ## なぜ JPEG をサポートするか
//!
//! favicon 用途で JPEG は本来不適当 (透過を持てず、 ロスシー圧縮で細部が
//! 潰れる) だが、 ユーザがロゴを JPEG でしか持っていないケースは少なくない
//! (写真撮影されたロゴ、 PowerPoint からエクスポート、 等)。
//!
//! 受け付けないと「対応していない」 エラーで突き返すことになり、 ユーザは
//! 「変換しないと使えない」 と困る。 受け付けて、 v1.7 の透過 audit で出る
//! `FullyOpaque` 警告とは別に **JPEG 専用の教育的な警告** を出すのが
//! v1.11.0 の方針 (UI 層 = `app.rs` で `push_jpeg_warning` を実装する)。
//!
//! 実装パターンは `decode_webp.rs` と意図的に揃えている。 入力種別ごとの
//! 分岐を上位 (`exporter::render_at_size`, `preview::render_at`) でだけ
//! 行えるようにするため。
//!
//! ## アルファチャネル
//!
//! JPEG は本来アルファを持たない (RGB のみ)。 `image::DynamicImage::to_rgba8`
//! は不足する alpha を 255 (完全不透明) で埋めて RGBA8 に変換するため、
//! 結果として「全ピクセル alpha=255」 の `Rgba8` が返る。 これは v1.7 の
//! `transparency_audit` で `FullyOpaque` と分類されるが、 UI 層は SourceKind
//! を見て JPEG 専用警告に振り替える。

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// JPEG ソースを RGBA8 にデコード。
///
/// 入力が JPEG でなければ `Err(UnsupportedFile)`。
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
