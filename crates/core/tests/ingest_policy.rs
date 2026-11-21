//! Ingest policy and input limits (RFC 014, job K1).
//!
//! * Content decides the format; the extension is a hint that is compared, never a gate. A **known**
//!   extension that contradicts the content gives a `FormatNote`; an unknown or missing extension
//!   with recognised content gives none.
//! * The limits (file 64 MiB, SVG 8 MiB, side 16 384, pixels 50 MP) are checked from the header
//!   before any decode, with hand-built byte arrays (no large allocation here).
//! * A path that is not a regular file is rejected before anything is read.
//! * Errors carry the file name only, never the directory.

mod fixtures;

use std::path::PathBuf;
use std::time::Duration;

use logolig::services::ingest::{
    MAX_INPUT_BYTES, MAX_PIXELS, MAX_SIDE, MAX_SVG_BYTES, ingest, ingest_bytes, ingest_bytes_noted,
    ingest_noted,
};
use logolig::{AppError, FormatNote, SourceKind};

/// The 24 bytes of a PNG that matter to the header check: the magic, an IHDR chunk length and
/// type, and the declared width and height. No pixel data and no CRC: only the header is read.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut b = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    b.extend_from_slice(&13u32.to_be_bytes());
    b.extend_from_slice(b"IHDR");
    b.extend_from_slice(&width.to_be_bytes());
    b.extend_from_slice(&height.to_be_bytes());
    b.extend_from_slice(&[8, 6, 0, 0, 0]);
    b
}

fn note_of(path: &str, bytes: Vec<u8>) -> Option<FormatNote> {
    ingest_bytes_noted(path, bytes).expect("ingests").note
}

// ---------------------------------------------------------------------------------------------
// Policy: content decides
// ---------------------------------------------------------------------------------------------

#[test]
fn a_jpeg_named_png_is_a_jpeg_and_carries_one_note() {
    let ingested = ingest_bytes_noted("logo.png", fixtures::jpeg_8x8_red()).unwrap();
    assert_eq!(ingested.asset.kind, SourceKind::Jpeg);
    assert_eq!(
        ingested.note,
        Some(FormatNote {
            named: SourceKind::Png,
            detected: SourceKind::Jpeg
        })
    );
}

#[test]
fn a_png_named_jpg_and_an_svg_named_png_each_carry_their_note() {
    assert_eq!(
        note_of("logo.jpg", fixtures::png_4x4_red()),
        Some(FormatNote {
            named: SourceKind::Jpeg,
            detected: SourceKind::Png
        })
    );
    let ingested = ingest_bytes_noted("logo.png", fixtures::SVG_16.as_bytes().to_vec()).unwrap();
    assert_eq!(ingested.asset.kind, SourceKind::Svg);
    assert_eq!(
        ingested.note,
        Some(FormatNote {
            named: SourceKind::Png,
            detected: SourceKind::Svg
        })
    );
}

#[test]
fn a_matching_extension_gives_no_note_and_jpg_equals_jpeg() {
    assert_eq!(note_of("logo.png", fixtures::png_4x4_red()), None);
    assert_eq!(note_of("logo.PNG", fixtures::png_4x4_red()), None);
    assert_eq!(note_of("logo.jpg", fixtures::jpeg_8x8_red()), None);
    assert_eq!(note_of("logo.jpeg", fixtures::jpeg_8x8_red()), None);
    assert_eq!(note_of("logo.webp", fixtures::webp_8x8_blue()), None);
    assert_eq!(
        note_of("logo.svg", fixtures::SVG_16.as_bytes().to_vec()),
        None
    );
}

#[test]
fn an_unknown_or_missing_extension_with_recognised_content_is_accepted_without_a_note() {
    for name in ["logo.dat", "logo", "logo.txt", ".hidden", "logo."] {
        let ingested = ingest_bytes_noted(name, fixtures::png_4x4_red())
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(ingested.asset.kind, SourceKind::Png, "{name}");
        assert_eq!(
            ingested.note, None,
            "{name}: no note for an unknown extension"
        );
    }
}

#[test]
fn text_content_with_a_text_extension_is_rejected_and_so_is_an_empty_file() {
    for (name, bytes) in [
        ("notes.txt", b"just some words".to_vec()),
        ("empty.png", Vec::new()),
        ("empty", Vec::new()),
    ] {
        let err = ingest_bytes(name, bytes).expect_err(name);
        assert!(
            matches!(err, AppError::UnsupportedFile { .. }),
            "{name}: {err:?}"
        );
    }
}

#[test]
fn the_plain_entry_points_keep_their_signature_and_still_return_the_asset() {
    // `ingest_bytes` and `ingest` are published: they still return a SourceAsset, and the note is
    // only reachable through the new `_noted` functions.
    let asset = ingest_bytes("logo.png", fixtures::jpeg_8x8_red()).unwrap();
    assert_eq!(asset.kind, SourceKind::Jpeg);
}

#[test]
fn truncated_headers_never_panic_for_any_format() {
    let full: Vec<(&str, Vec<u8>)> = vec![
        ("a.png", fixtures::png_4x4_red()),
        ("a.jpg", fixtures::jpeg_8x8_red()),
        ("a.webp", fixtures::webp_8x8_blue()),
        ("a.svg", fixtures::SVG_16.as_bytes().to_vec()),
    ];
    for (name, bytes) in full {
        for cut in 0..bytes.len().min(64) {
            // Whatever comes out, it is a result, not a panic; a cut file never exceeds a limit.
            match ingest_bytes(name, bytes[..cut].to_vec()) {
                Ok(_) => {}
                Err(AppError::UnsupportedFile { .. }) => {}
                Err(other) => panic!("{name} cut at {cut}: unexpected {other:?}"),
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Limits, from the header, before decoding
// ---------------------------------------------------------------------------------------------

#[test]
fn the_limits_are_the_decided_numbers() {
    assert_eq!(MAX_INPUT_BYTES, 64 * 1024 * 1024);
    assert_eq!(MAX_SVG_BYTES, 8 * 1024 * 1024);
    assert_eq!(MAX_SIDE, 16_384);
    assert_eq!(MAX_PIXELS, 50_000_000);
}

#[test]
fn a_header_declaring_70000_by_70000_is_rejected_by_the_side_limit() {
    let err = ingest_bytes("big.png", png_header(70_000, 70_000)).expect_err("too large");
    assert!(matches!(err, AppError::ImageTooLarge { .. }), "{err:?}");
}

#[test]
fn one_side_over_the_limit_is_rejected_even_when_the_pixel_count_is_small() {
    let err = ingest_bytes("long.png", png_header(MAX_SIDE + 1, 1)).expect_err("too long");
    assert!(matches!(err, AppError::ImageTooLarge { .. }), "{err:?}");
}

#[test]
fn the_pixel_limit_rejects_just_over_and_accepts_just_under_from_the_header_alone() {
    // 8 000 x 8 000 = 64 MP and 9 000 x 8 000 = 72 MP: over. 7 071 x 7 071 = 49 999 041: under the
    // 50 MP limit, accepted. 7 072 x 7 071 = 50 006 112: one column over, rejected. Only the header
    // exists in these byte arrays: nothing here allocates an image.
    for (w, h) in [(8_000, 8_000), (9_000, 8_000), (7_072, 7_071)] {
        let err = ingest_bytes("a.png", png_header(w, h)).expect_err("over the pixel limit");
        assert!(
            matches!(err, AppError::ImageTooLarge { .. }),
            "{w}x{h}: {err:?}"
        );
    }
    let asset = ingest_bytes("a.png", png_header(7_071, 7_071)).expect("49.99 MP");
    assert_eq!(asset.intrinsic_size, Some((7_071, 7_071)));
    // A wide but thin image is under the pixel limit and under the side limit: accepted.
    ingest_bytes("a.png", png_header(MAX_SIDE, 3_000)).expect("16 384 x 3 000 = 49.2 MP");
}

#[test]
fn the_same_limits_apply_to_jpeg_and_webp_headers() {
    // A JPEG SOF0 declaring 20 000 x 20 000 (the 16-bit fields hold it).
    let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x11, 0x08];
    jpeg.extend_from_slice(&20_000u16.to_be_bytes());
    jpeg.extend_from_slice(&20_000u16.to_be_bytes());
    jpeg.extend_from_slice(&[3, 1, 0x22, 0, 2, 0x11, 1, 3, 0x11, 1]);
    let err = ingest_bytes("a.jpg", jpeg).expect_err("jpeg");
    assert!(matches!(err, AppError::ImageTooLarge { .. }), "{err:?}");
    // A WebP VP8X declaring 30 000 x 30 000 (24-bit fields, stored as size - 1).
    let mut webp = b"RIFF".to_vec();
    webp.extend_from_slice(&30u32.to_le_bytes());
    webp.extend_from_slice(b"WEBPVP8X");
    webp.extend_from_slice(&10u32.to_le_bytes());
    webp.extend_from_slice(&[0, 0, 0, 0]);
    webp.extend_from_slice(&(29_999u32).to_le_bytes()[..3]);
    webp.extend_from_slice(&(29_999u32).to_le_bytes()[..3]);
    let err = ingest_bytes("a.webp", webp).expect_err("webp");
    assert!(matches!(err, AppError::ImageTooLarge { .. }), "{err:?}");
}

#[test]
fn an_svg_over_8_mib_is_rejected_and_a_raster_of_that_size_is_not() {
    let mut svg = fixtures::SVG_16.as_bytes().to_vec();
    svg.resize(MAX_SVG_BYTES as usize + 1, b' ');
    let err = ingest_bytes("big.svg", svg).expect_err("svg too large");
    match err {
        AppError::InputTooLarge { limit, .. } => assert_eq!(limit, MAX_SVG_BYTES),
        other => panic!("{other:?}"),
    }
    // A PNG header followed by 9 MiB of padding is a raster: the 8 MiB limit is for SVG only.
    let mut png = png_header(4, 4);
    png.resize(9 * 1024 * 1024, 0);
    ingest_bytes("big.png", png).expect("a raster is not held to the SVG limit");
}

#[test]
fn bytes_over_the_file_limit_are_rejected_by_the_byte_check() {
    // The check on the bytes that were read (the second of the two checks: the first is the file's
    // length before it is read, below). 64 MiB + 1 of zeros: no format, but the size comes first.
    let err = ingest_bytes("x.png", vec![0u8; MAX_INPUT_BYTES as usize + 1]).expect_err("size");
    match err {
        AppError::InputTooLarge { limit, .. } => assert_eq!(limit, MAX_INPUT_BYTES),
        other => panic!("{other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Files: size from the metadata before reading, regular files only
// ---------------------------------------------------------------------------------------------

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("logolig-k1-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Runs the async ingest on a helper thread with a deadline: a read that blocks for ever (a FIFO,
/// a device) is a failing test, not a hung one.
fn ingest_with_deadline(path: PathBuf) -> Result<logolig::SourceAsset, AppError> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let _ = tx.send(rt.block_on(ingest(path)));
    });
    rx.recv_timeout(Duration::from_secs(10))
        .expect("the ingest did not return within 10 s (it blocked on a non-regular file?)")
}

#[test]
fn a_file_one_byte_over_the_limit_is_rejected_from_its_length_without_being_read() {
    let dir = scratch("over");
    let path = dir.join("huge.png");
    // A sparse file: its length is 64 MiB + 1, and almost nothing is on disk. Reading it in full
    // would be the failure (it would also fail, but only after allocating 64 MiB).
    let f = std::fs::File::create(&path).unwrap();
    f.set_len(MAX_INPUT_BYTES + 1).unwrap();
    drop(f);
    let err = ingest_with_deadline(path).expect_err("too large");
    match err {
        AppError::InputTooLarge { path, limit } => {
            assert_eq!(path, "huge.png", "the file name only");
            assert_eq!(limit, MAX_INPUT_BYTES);
        }
        other => panic!("{other:?}"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_file_exactly_at_the_limit_is_not_rejected_for_its_size() {
    let dir = scratch("at");
    let path = dir.join("edge.bin");
    let f = std::fs::File::create(&path).unwrap();
    f.set_len(MAX_INPUT_BYTES).unwrap();
    drop(f);
    // Zeros: no known format, so UnsupportedFile, not InputTooLarge.
    let err = ingest_with_deadline(path).expect_err("not an image");
    assert!(matches!(err, AppError::UnsupportedFile { .. }), "{err:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_directory_is_rejected_before_any_read() {
    let dir = scratch("dir");
    let err = ingest_with_deadline(dir.clone()).expect_err("a directory");
    match err {
        AppError::UnsupportedFile { path } => assert!(!path.contains('/'), "{path}"),
        other => panic!("{other:?}"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn a_fifo_is_rejected_without_blocking() {
    let dir = scratch("fifo");
    let fifo = dir.join("pipe.png");
    let made = std::process::Command::new("mkfifo").arg(&fifo).status();
    if !matches!(made, Ok(s) if s.success()) {
        eprintln!("SKIPPED: `mkfifo` is not available here");
        let _ = std::fs::remove_dir_all(dir);
        return;
    }
    let err = ingest_with_deadline(fifo).expect_err("a FIFO");
    assert!(matches!(err, AppError::UnsupportedFile { .. }), "{err:?}");
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
#[test]
fn a_device_that_reads_for_ever_is_rejected_before_reading() {
    if !std::path::Path::new("/dev/zero").exists() {
        eprintln!("SKIPPED: no /dev/zero");
        return;
    }
    let err = ingest_with_deadline(PathBuf::from("/dev/zero")).expect_err("a device");
    assert!(matches!(err, AppError::UnsupportedFile { .. }), "{err:?}");
}

#[cfg(unix)]
#[test]
fn a_symlink_to_a_regular_file_is_followed_and_accepted() {
    let dir = scratch("link");
    let real = dir.join("real.png");
    std::fs::write(&real, fixtures::png_4x4_red()).unwrap();
    let link = dir.join("link.png");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let asset = ingest_with_deadline(link).expect("a symlink to a regular file");
    assert_eq!(asset.kind, SourceKind::Png);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_missing_file_is_an_io_error_with_the_file_name_only() {
    let dir = scratch("missing");
    let err = ingest_with_deadline(dir.join("gone.png")).expect_err("missing");
    match err {
        AppError::Io { path, .. } => assert_eq!(path, "gone.png"),
        other => panic!("{other:?}"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_async_ingest_reports_the_note_too_and_the_error_fields_never_hold_a_directory() {
    let dir = scratch("note");
    let path = dir.join("pic.png");
    std::fs::write(&path, fixtures::jpeg_8x8_red()).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let ingested = rt.block_on(ingest_noted(path)).expect("ingests");
    assert_eq!(ingested.asset.kind, SourceKind::Jpeg);
    assert!(ingested.note.is_some());
    // A bad file in the same directory: the error carries "bad.txt", not the directory.
    let bad = dir.join("bad.txt");
    std::fs::write(&bad, b"hello").unwrap();
    let err = rt.block_on(ingest_noted(bad)).expect_err("rejected");
    match err {
        AppError::UnsupportedFile { path } => assert_eq!(path, "bad.txt"),
        other => panic!("{other:?}"),
    }
    let _ = std::fs::remove_dir_all(dir);
}

// ---------------------------------------------------------------------------------------------
// The second line: decoders run with limits when the header could not be checked
// ---------------------------------------------------------------------------------------------

/// A real PNG whose width is over the side limit but whose pixel data is tiny (17 000 x 1).
fn png_17000_by_1() -> Vec<u8> {
    let img = image::RgbaImage::new(MAX_SIDE + 616, 1);
    let mut out = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

#[test]
fn a_decode_that_ingest_did_not_check_is_still_stopped_by_the_decoder_limits() {
    use logolig::services::decode_png;
    // The same bytes as a SourceAsset built by hand, with no intrinsic size (what a damaged header
    // gives): the header check cannot refuse it, the decoder limits must.
    let bytes = png_17000_by_1();
    let asset = logolig::SourceAsset {
        path: PathBuf::from("/some/dir/wide.png"),
        kind: SourceKind::Png,
        raw: std::sync::Arc::<[u8]>::from(bytes.clone()),
        intrinsic_size: None,
    };
    match decode_png::decode(&asset) {
        Err(AppError::ImageTooLarge { path, .. }) => assert_eq!(path, "wide.png"),
        other => panic!("{other:?}"),
    }
    // And ingest itself refuses the same bytes from the header.
    let err = ingest_bytes("wide.png", bytes).expect_err("wide");
    assert!(matches!(err, AppError::ImageTooLarge { .. }), "{err:?}");
}

#[test]
fn an_ordinary_image_still_decodes_through_the_limited_path() {
    use logolig::services::{decode_jpeg, decode_png, decode_webp};
    for (name, bytes, decode) in [
        (
            "a.png",
            fixtures::png_4x4_red(),
            decode_png::decode as fn(&logolig::SourceAsset) -> _,
        ),
        ("a.jpg", fixtures::jpeg_8x8_red(), decode_jpeg::decode),
        ("a.webp", fixtures::webp_8x8_blue(), decode_webp::decode),
    ] {
        let asset = ingest_bytes(name, bytes).unwrap();
        decode(&asset).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}
