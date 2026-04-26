//! ingest の振る舞いテスト。
//!
//! - PNG / SVG が正しく種別判定される
//! - PNG のヘッダから論理サイズが取れる
//! - SVG のサイズが usvg 経由で取れる
//! - 種別判定不能なら `UnsupportedFile`
//! - 拡張子が ".png" を名乗る別バイト列も拒絶する (拡張子偽装防止)
//! - async 版がランタイム上で動く

mod fixtures;

use logolig_core::services::ingest::{ingest, ingest_bytes};
use logolig_core::{AppError, SourceKind};

#[test]
fn detects_png_and_reads_intrinsic_size() {
    let bytes = fixtures::png_4x4_red();
    let asset = ingest_bytes("dummy.png", bytes).expect("PNG should ingest");
    assert_eq!(asset.kind, SourceKind::Png);
    assert_eq!(asset.intrinsic_size, Some((4, 4)));
}

#[test]
fn detects_svg_and_reads_intrinsic_size() {
    let asset = ingest_bytes("dummy.svg", fixtures::SVG_16.as_bytes().to_vec())
        .expect("SVG should ingest");
    assert_eq!(asset.kind, SourceKind::Svg);
    assert_eq!(asset.intrinsic_size, Some((16, 16)));
}

#[test]
fn rejects_unrelated_bytes_even_with_png_extension() {
    let err =
        ingest_bytes("fake.png", b"not really an image".to_vec()).expect_err("should reject");
    assert!(matches!(err, AppError::UnsupportedFile(_)));
}

#[test]
fn rejects_unknown_extension_and_unknown_content() {
    let err = ingest_bytes("notes.txt", b"hello".to_vec()).expect_err("should reject");
    assert!(matches!(err, AppError::UnsupportedFile(_)));
}

#[test]
fn ingest_async_round_trip_via_tempfile() {
    // tokio runtime をテスト時にだけ立ててファイル経由の async ingest を確認。
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let tmp = std::env::temp_dir().join("logolig-step2-ingest.png");
    let bytes = fixtures::png_4x4_red();
    std::fs::write(&tmp, &bytes).unwrap();

    let asset = rt.block_on(ingest(tmp.clone())).expect("async ingest");
    assert_eq!(asset.kind, SourceKind::Png);
    assert_eq!(asset.intrinsic_size, Some((4, 4)));
    // 元ファイルが書き換わっていない (§6.4 非破壊性) ことの確認
    assert_eq!(std::fs::read(&tmp).unwrap(), bytes);

    let _ = std::fs::remove_file(&tmp);
}
