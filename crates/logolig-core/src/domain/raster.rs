//! ラスタライズ結果の不変表現。
//!
//! 仕様 §6.4「変換結果は内部生成物として扱う」を素直に表現するための型。
//! `Arc<[u8]>` でクローン安価に持ち、UI スレッドへ受け渡してもバッファの
//! 重複コピーが起きない (Step 3 のプレビュー描画で使う想定)。

use std::sync::Arc;

/// RGBA8 ピクセルバッファ。`width * height * 4` バイトを保持。
///
/// この型はサービス層が出力し、UI 層は参照するだけで書き換えない。
#[derive(Debug, Clone)]
pub struct Rgba8 {
    pub width: u32,
    pub height: u32,
    /// RGBA, 1 ピクセル = 4 バイト, ストライド = `width * 4`。
    pub pixels: Arc<[u8]>,
}

impl Rgba8 {
    /// バイト列が `width * height * 4` であることを保証する。
    /// 違反時は `None` を返し、 サービス側で Decode/Resize エラーへ変換する。
    pub fn try_from_raw(width: u32, height: u32, pixels: Arc<[u8]>) -> Option<Self> {
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() == expected {
            Some(Self { width, height, pixels })
        } else {
            None
        }
    }

    /// バッファ末尾のスライス。テストや書き出しから読みたい時用。
    pub fn as_bytes(&self) -> &[u8] {
        &self.pixels
    }
}
