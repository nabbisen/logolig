//! テスト用の最小 fixture を集約したヘルパ。
//!
//! 実ファイル I/O テストには `ingest_bytes` (同期版) を使い、ランタイムを引き込まない。
//! async 版は別テストで `#[tokio::test]` を介して検証する。

#![allow(dead_code)]

/// 16×16 の青いマス目を中央に置く SVG。
/// width / height 属性を持ち、resvg が確実にパースできる構造。
pub const SVG_16: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 16 16">
  <rect x="2" y="2" width="12" height="12" fill="#3366cc"/>
</svg>"#;

/// 4×4 の赤色 PNG。
///
/// `image` クレートでエンコードして得たバイト列をそのまま埋め込む。
/// 4×4 にしているのは IHDR の幅/高さが 0x00000004 という分かりやすい値になり、
/// `parse_png_size` の挙動を目視確認しやすいため。
pub fn png_4x4_red() -> Vec<u8> {
    let mut buf = image::RgbaImage::new(4, 4);
    for px in buf.pixels_mut() {
        *px = image::Rgba([0xCC, 0x33, 0x33, 0xFF]);
    }
    let mut out = Vec::new();
    let dynamic = image::DynamicImage::ImageRgba8(buf);
    dynamic
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .expect("PNG encoding for fixture should not fail");
    out
}
