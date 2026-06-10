//! 複数の RGBA8 ラスタを 1 つの `.ico` バイト列にまとめる。
//!
//! 設計上の重要点 (§7.1):
//! - **各サイズはソースから個別にレンダリング** されたものを受け取ること
//!   (これは呼び出し側 `exporter::run` の責務)。 ICO writer は受け取った
//!   フレームをそのまま詰めるだけで、 自前で縮小はしない
//! - `IconDirEntry::encode()` はフレームの寸法を見て BMP / PNG を自動選択。
//!   小さいサイズ (16/32/48 等) は BMP、 大きいサイズ (256以上) は PNG。
//!   後方互換性が最も高い

use std::io::Cursor;

use ico::{IconDir, IconDirEntry, IconImage, ResourceType};

use crate::domain::Rgba8;
use crate::error::AppError;

/// フレーム群を 1 つの ICO バイト列にまとめる。
///
/// `frames` は (size, RGBA8) のリスト。 サイズと RGBA8 の縦横が一致しない
/// 場合は `Err(Export(_))` を返す (誤った組み立てを防ぐためのガード)。
pub fn build(frames: &[(u32, &Rgba8)]) -> Result<Vec<u8>, AppError> {
    if frames.is_empty() {
        return Err(AppError::Export("ICO requires at least one frame".into()));
    }

    let mut dir = IconDir::new(ResourceType::Icon);
    for (size, rgba) in frames {
        if rgba.width != *size || rgba.height != *size {
            return Err(AppError::Export(format!(
                "ICO frame size mismatch: requested {size}, got {}x{}",
                rgba.width, rgba.height
            )));
        }
        // ico 0.4 は所有バッファ (Vec<u8>) を要求する。
        let pixels = rgba.as_bytes().to_vec();
        let image = IconImage::from_rgba_data(rgba.width, rgba.height, pixels);
        let entry = IconDirEntry::encode(&image)
            .map_err(|e| AppError::Export(format!("ICO entry encode at {size}px: {e}")))?;
        dir.add_entry(entry);
    }

    let mut out = Vec::new();
    dir.write(Cursor::new(&mut out))
        .map_err(|e| AppError::Export(format!("ICO write: {e}")))?;
    Ok(out)
}
