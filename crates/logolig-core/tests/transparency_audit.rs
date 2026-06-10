//! `transparency_audit` の挙動確認 (v1.7.0)。
//!
//! 検証する性質:
//! 1. 全 alpha=255 の画像が `FullyOpaque` に分類されること
//! 2. 全 alpha=0 の画像が `FullyTransparent` に分類されること
//! 3. 透明・不透明が混在する画像が `HasTransparency` に分類されること
//! 4. 1 ピクセルでも違えば混在判定 (境界条件: 早期終了の正しさ)
//! 5. 空画像 (width=0 or height=0) を `FullyTransparent` 扱いにする (安全側)
//! 6. `needs_warning` が混在ケースで false、 警告ケースで true

use std::sync::Arc;

use logolig_core::services::transparency_audit::{audit, TransparencyReport};
use logolig_core::Rgba8;

/// width × height のサイズで、 全ピクセルの alpha を `alpha` に揃えた画像を作る。
/// RGB はゼロで揃える (alpha 判定だけが目的なので)。
fn solid_alpha_image(width: u32, height: u32, alpha: u8) -> Rgba8 {
    let n = (width as usize) * (height as usize);
    let mut buf = Vec::with_capacity(n * 4);
    for _ in 0..n {
        buf.extend_from_slice(&[0, 0, 0, alpha]);
    }
    Rgba8::try_from_raw(width, height, Arc::from(buf.into_boxed_slice()))
        .expect("solid_alpha_image: dimensions must match buffer length")
}

/// 全ピクセル alpha=255 だが、 引数で指定した index 番目の alpha だけ 0 にした画像。
fn alpha_with_one_transparent(width: u32, height: u32, transparent_pixel_idx: usize) -> Rgba8 {
    let n = (width as usize) * (height as usize);
    let mut buf = Vec::with_capacity(n * 4);
    for i in 0..n {
        let alpha = if i == transparent_pixel_idx { 0 } else { 255 };
        buf.extend_from_slice(&[0, 0, 0, alpha]);
    }
    Rgba8::try_from_raw(width, height, Arc::from(buf.into_boxed_slice()))
        .expect("alpha_with_one_transparent: dimensions must match buffer length")
}

#[test]
fn fully_opaque_is_detected() {
    let img = solid_alpha_image(4, 4, 255);
    assert_eq!(audit(&img), TransparencyReport::FullyOpaque);
}

#[test]
fn fully_transparent_is_detected() {
    let img = solid_alpha_image(4, 4, 0);
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}

#[test]
fn mixed_alpha_at_boundary_pixel_counts_as_has_transparency() {
    // 16 ピクセルのうち最後の 1 つだけ alpha=0、 他は 255
    // → 「ほぼ不透明だが 1 ピクセルだけ透明」 でも HasTransparency になるべき
    let img = alpha_with_one_transparent(4, 4, 15); // 16 ピクセル中 index 15 (最後)
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn mixed_alpha_at_first_pixel_counts_as_has_transparency() {
    // 早期終了 (`min==0 && max==255` で return) が正しく働くかの確認
    // 最初のピクセルが透明、 残りは不透明
    let img = alpha_with_one_transparent(4, 4, 0);
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn semi_transparent_uniform_treated_as_has_transparency() {
    // 全ピクセルが alpha=128 のような半透明一様画像。
    // FullyOpaque でも FullyTransparent でもないので HasTransparency 扱い。
    // (favicon 用途で実害は限定的なので警告対象から外す判断)
    let img = solid_alpha_image(2, 2, 128);
    assert_eq!(audit(&img), TransparencyReport::HasTransparency);
}

#[test]
fn empty_image_is_fully_transparent() {
    // width=0 でピクセル 0 個のケース。
    let img = Rgba8::try_from_raw(0, 0, Arc::from(Vec::<u8>::new().into_boxed_slice()))
        .expect("zero-size with empty buffer must be valid");
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}

#[test]
fn needs_warning_only_for_extreme_cases() {
    assert!(TransparencyReport::FullyOpaque.needs_warning());
    assert!(TransparencyReport::FullyTransparent.needs_warning());
    assert!(!TransparencyReport::HasTransparency.needs_warning());
}

#[test]
fn single_pixel_opaque_image_is_fully_opaque() {
    // 境界条件: 1×1 の画像で audit が破綻しないこと
    let img = solid_alpha_image(1, 1, 255);
    assert_eq!(audit(&img), TransparencyReport::FullyOpaque);
}

#[test]
fn single_pixel_transparent_image_is_fully_transparent() {
    let img = solid_alpha_image(1, 1, 0);
    assert_eq!(audit(&img), TransparencyReport::FullyTransparent);
}
