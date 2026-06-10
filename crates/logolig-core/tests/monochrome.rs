//! `monochrome` サービスの挙動確認 (v1.9.0)。
//!
//! 検証する性質:
//! 1. RGB すべて同値 (グレー) のピクセルは同じ値で出てくる
//! 2. アルファチャネルが保持される
//! 3. 純赤 / 純緑 / 純青 が BT.709 係数に従って正しく輝度化される
//! 4. 出力画像のサイズ (width, height) が入力と一致する
//! 5. 透明部分は透明のまま (mono 化で黒く塗られない)
//! 6. 全黒入力 → 全黒出力
//! 7. 全白入力 → 全白出力

use std::sync::Arc;

use logolig_core::services::monochrome::{into_grayscale, to_grayscale};
use logolig_core::Rgba8;

fn make(width: u32, height: u32, pixels: Vec<u8>) -> Rgba8 {
    Rgba8::try_from_raw(width, height, Arc::from(pixels.into_boxed_slice()))
        .expect("test fixture: dimensions must match buffer length")
}

#[test]
fn dimensions_are_preserved() {
    let img = make(4, 3, vec![128; 4 * 3 * 4]);
    let mono = to_grayscale(&img);
    assert_eq!(mono.width, 4);
    assert_eq!(mono.height, 3);
    assert_eq!(mono.pixels.len(), img.pixels.len());
}

#[test]
fn alpha_is_preserved_per_pixel() {
    // alpha が 4 種類混在する画像を作って、 mono 化後も同じ alpha が出るか確認
    let img = make(
        4,
        1,
        vec![
            255, 0, 0, 255, // 赤・不透明
            0, 255, 0, 128, // 緑・半透明
            0, 0, 255, 64,  // 青・低不透明度
            128, 128, 128, 0, // グレー・完全透明
        ],
    );
    let mono = to_grayscale(&img);
    assert_eq!(mono.pixels[3], 255);
    assert_eq!(mono.pixels[7], 128);
    assert_eq!(mono.pixels[11], 64);
    assert_eq!(mono.pixels[15], 0);
}

#[test]
fn pure_red_uses_bt709_coefficient() {
    // BT.709: Y = 0.2126 * 255 = 54.213 → 54 (round)
    let img = make(1, 1, vec![255, 0, 0, 255]);
    let mono = to_grayscale(&img);
    let y = mono.pixels[0];
    assert_eq!(y, 54, "BT.709 luma of pure red should be 54, got {y}");
    assert_eq!(mono.pixels[1], y);
    assert_eq!(mono.pixels[2], y);
}

#[test]
fn pure_green_uses_bt709_coefficient() {
    // BT.709: Y = 0.7152 * 255 = 182.376 → 182 (round)
    let img = make(1, 1, vec![0, 255, 0, 255]);
    let mono = to_grayscale(&img);
    let y = mono.pixels[0];
    assert_eq!(y, 182, "BT.709 luma of pure green should be 182, got {y}");
}

#[test]
fn pure_blue_uses_bt709_coefficient() {
    // BT.709: Y = 0.0722 * 255 = 18.411 → 18 (round)
    let img = make(1, 1, vec![0, 0, 255, 255]);
    let mono = to_grayscale(&img);
    let y = mono.pixels[0];
    assert_eq!(y, 18, "BT.709 luma of pure blue should be 18, got {y}");
}

#[test]
fn black_stays_black() {
    let img = make(2, 2, vec![0; 2 * 2 * 4]);
    let mono = to_grayscale(&img);
    for chunk in mono.pixels.chunks_exact(4) {
        assert_eq!(chunk[0], 0);
        assert_eq!(chunk[1], 0);
        assert_eq!(chunk[2], 0);
    }
}

#[test]
fn white_stays_white() {
    // BT.709 係数の和は 1 なので、 純白は 255 のまま
    let img = make(
        2,
        2,
        vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
    );
    let mono = to_grayscale(&img);
    for chunk in mono.pixels.chunks_exact(4) {
        assert_eq!(chunk[0], 255);
        assert_eq!(chunk[1], 255);
        assert_eq!(chunk[2], 255);
    }
}

#[test]
fn already_grayscale_is_unchanged() {
    // R=G=B=K の入力は出力も Y=K になる (BT.709 係数の和が 1 だから)
    let img = make(1, 1, vec![123, 123, 123, 200]);
    let mono = to_grayscale(&img);
    assert_eq!(mono.pixels[0], 123);
    assert_eq!(mono.pixels[1], 123);
    assert_eq!(mono.pixels[2], 123);
    assert_eq!(mono.pixels[3], 200);
}

#[test]
fn into_grayscale_matches_to_grayscale() {
    // owned 版と borrowed 版が同じ結果を返すこと
    let img = make(2, 1, vec![100, 50, 200, 255, 30, 80, 250, 128]);
    let a = to_grayscale(&img);
    let b = into_grayscale(img);
    assert_eq!(a.pixels.as_ref(), b.pixels.as_ref());
}
