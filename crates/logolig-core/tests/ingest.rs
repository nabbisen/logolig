//! Ingest behaviour tests.
//!
//! - PNG / SVG are classified correctly
//! - PNG header yields correct logical dimensions
//! - SVG dimensions are read via usvg
//! - Unrecognised format returns `UnsupportedFile`
//! - A file with a ".png" extension but wrong magic bytes is rejected
//! - Async ingest runs correctly on the runtime

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
fn detects_webp_and_reads_intrinsic_size() {
    let bytes = fixtures::webp_8x8_blue();
    let asset = ingest_bytes("dummy.webp", bytes).expect("WebP should ingest");
    assert_eq!(asset.kind, SourceKind::Webp);
    // The encoder may choose VP8 or VP8L — implementation-defined, but
    // Both extensions should decode to 8×8.
    assert_eq!(asset.intrinsic_size, Some((8, 8)));
}

#[test]
fn rejects_unrelated_bytes_even_with_png_extension() {
    let err =
        ingest_bytes("fake.png", b"not really an image".to_vec()).expect_err("should reject");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}

#[test]
fn rejects_unknown_extension_and_unknown_content() {
    let err = ingest_bytes("notes.txt", b"hello".to_vec()).expect_err("should reject");
    assert!(matches!(err, AppError::UnsupportedFile { .. }));
}

#[test]
fn ingest_async_round_trip_via_tempfile() {
    // Spin up a tokio runtime for this test to verify async file-based ingestion.
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
    // Verify the source file is unchanged (§6.4 non-destructive).
    assert_eq!(std::fs::read(&tmp).unwrap(), bytes);

    let _ = std::fs::remove_file(&tmp);
}

// ---------------------------------------------------------------------------
// v1.11.0: JPEG support
// ---------------------------------------------------------------------------

#[test]
fn ingest_recognizes_jpeg_magic_bytes() {
    // Bytes starting with JPEG SOI (FF D8 FF) are classified as JPEG
    let asset = ingest_bytes("photo.jpg", fixtures::jpeg_8x8_red()).unwrap();
    assert_eq!(asset.kind, SourceKind::Jpeg);
}

#[test]
fn ingest_recognizes_jpeg_via_jpeg_extension() {
    // .jpeg extension is treated the same as .jpg
    let asset = ingest_bytes("photo.jpeg", fixtures::jpeg_8x8_red()).unwrap();
    assert_eq!(asset.kind, SourceKind::Jpeg);
}

#[test]
fn ingest_parses_jpeg_intrinsic_size_from_sof_marker() {
    // Width/height correctly extracted from the SOF marker
    let asset = ingest_bytes("photo.jpg", fixtures::jpeg_8x8_red()).unwrap();
    assert_eq!(asset.intrinsic_size, Some((8, 8)));
}

#[test]
fn ingest_rejects_corrupt_jpeg_with_only_soi() {
    // SOI followed by invalid bytes → intrinsic_size = None,
    // but ingest still succeeds (image crate will reject at decode time).
    // Only the ingest-stage behaviour is checked here.
    let bytes = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00];
    let asset = ingest_bytes("broken.jpg", bytes).unwrap();
    assert_eq!(asset.kind, SourceKind::Jpeg);
    // Truncated → no SOF found → intrinsic_size = None
    assert_eq!(asset.intrinsic_size, None);
}
