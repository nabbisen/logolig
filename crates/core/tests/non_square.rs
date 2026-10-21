//! Non-square raster sources in square outputs (RFC 010).
//!
//! Square outputs must contain-fit a non-square PNG/WebP/JPEG on a transparent
//! square canvas: never stretched, never cropped.

mod fixtures;

use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;
use logolig::{ExportPlan, InMemoryArtifact};

const RED: [u8; 4] = [255, 0, 0, 255];

fn artifacts(file: &str, bytes: Vec<u8>, plan: &ExportPlan) -> Vec<InMemoryArtifact> {
    let asset = ingest_bytes(file, bytes).expect("fixture should ingest");
    run_in_memory(&asset, plan).expect("run_in_memory should succeed")
}

fn find<'a>(arts: &'a [InMemoryArtifact], path: &str) -> &'a InMemoryArtifact {
    arts.iter()
        .find(|a| a.relative_path.to_string_lossy().replace('\\', "/") == path)
        .unwrap_or_else(|| panic!("artifact {path} not found"))
}

fn decode(png: &[u8]) -> image::RgbaImage {
    image::load_from_memory(png)
        .expect("artifact should decode")
        .to_rgba8()
}

fn png_size_only(size: u32) -> ExportPlan {
    ExportPlan {
        png_sizes: vec![size],
        ..ExportPlan::default()
    }
}

/// Bounding box `(x0, y0, x1, y1)` (inclusive) of pixels with alpha > 0.
fn visible_box(img: &image::RgbaImage) -> (u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 0 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    assert!(x0 != u32::MAX, "image has no visible pixel");
    (x0, y0, x1, y1)
}

fn row_alpha_all_zero(img: &image::RgbaImage, y: u32) -> bool {
    (0..img.width()).all(|x| img.get_pixel(x, y)[3] == 0)
}

fn col_alpha_all_zero(img: &image::RgbaImage, x: u32) -> bool {
    (0..img.height()).all(|y| img.get_pixel(x, y)[3] == 0)
}

/// T1: a 2:1 source is contain-fitted: 32x32 output, transparent top and bottom
/// rows, opaque centre, visible box about 32 wide and 16 high.
#[test]
fn t1_wide_source_is_letterboxed_not_stretched() {
    let arts = artifacts(
        "wide.png",
        fixtures::png_rgba(64, 32, RED),
        &png_size_only(32),
    );
    let img = decode(&find(&arts, "favicon-32.png").bytes);
    assert_eq!((img.width(), img.height()), (32, 32));
    assert!(
        row_alpha_all_zero(&img, 0),
        "top row must be transparent padding"
    );
    assert!(
        row_alpha_all_zero(&img, 31),
        "bottom row must be transparent padding"
    );
    assert_eq!(img.get_pixel(16, 16)[3], 255, "centre must be opaque");
    let (x0, y0, x1, y1) = visible_box(&img);
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    assert!(
        (31..=33).contains(&w) && (15..=17).contains(&h),
        "visible box {w}x{h}"
    );
}

/// T2: the mirror case. A 1:2 source gives transparent left and right columns.
#[test]
fn t2_tall_source_is_pillarboxed_not_stretched() {
    let arts = artifacts(
        "tall.png",
        fixtures::png_rgba(32, 64, RED),
        &png_size_only(32),
    );
    let img = decode(&find(&arts, "favicon-32.png").bytes);
    assert_eq!((img.width(), img.height()), (32, 32));
    assert!(
        col_alpha_all_zero(&img, 0),
        "left column must be transparent padding"
    );
    assert!(
        col_alpha_all_zero(&img, 31),
        "right column must be transparent padding"
    );
    assert_eq!(img.get_pixel(16, 16)[3], 255, "centre must be opaque");
    let (x0, y0, x1, y1) = visible_box(&img);
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    assert!(
        (15..=17).contains(&w) && (31..=33).contains(&h),
        "visible box {w}x{h}"
    );
}

/// T3: with `keep_transparency = false` the padding is flattened to opaque white.
#[test]
fn t3_flatten_makes_the_padding_opaque_white() {
    let plan = ExportPlan {
        keep_transparency: false,
        ..png_size_only(32)
    };
    let arts = artifacts("wide.png", fixtures::png_rgba(64, 32, RED), &plan);
    let img = decode(&find(&arts, "favicon-32.png").bytes);
    assert_eq!(
        img.get_pixel(0, 0).0,
        [255, 255, 255, 255],
        "corner is white padding"
    );
    assert_eq!(
        img.get_pixel(16, 16).0,
        [255, 0, 0, 255],
        "centre stays red"
    );
    assert!(
        img.pixels().all(|p| p[3] == 255),
        "no pixel may stay translucent"
    );
}

/// T4: oracle for square sources. The exporter output equals the plain resize
/// path byte for byte, for every raster kind, two sizes and apple-touch.
#[test]
fn t4_square_sources_match_the_plain_resize_oracle() {
    use logolig::services::{decode_jpeg, decode_png, decode_webp, encode_png, resize};

    let plan = ExportPlan {
        png_sizes: vec![16, 32],
        include_apple_touch: true,
        ..ExportPlan::default()
    };
    let alg = plan.algorithm;
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("sq.png", fixtures::png_4x4_red()),
        ("sq.webp", fixtures::webp_8x8_blue()),
        ("sq.jpg", fixtures::jpeg_8x8_red()),
    ];
    for (file, bytes) in cases {
        let asset = ingest_bytes(file, bytes).unwrap();
        let decoded = match file {
            "sq.png" => decode_png::decode(&asset).unwrap(),
            "sq.webp" => decode_webp::decode(&asset).unwrap(),
            _ => decode_jpeg::decode(&asset).unwrap(),
        };
        let arts = run_in_memory(&asset, &plan).unwrap();
        for (name, size) in [
            ("favicon-16.png", 16),
            ("favicon-32.png", 32),
            ("apple-touch-icon.png", 180),
        ] {
            let expected =
                encode_png::encode(&resize::resize(&decoded, size, size, alg).unwrap()).unwrap();
            assert_eq!(
                find(&arts, name).bytes,
                expected,
                "{file}: {name} must be byte-identical"
            );
        }
    }
}

/// T5: ICO frames and apple-touch share the same rule.
#[test]
fn t5_ico_frames_and_apple_touch_are_letterboxed() {
    let plan = ExportPlan {
        png_sizes: vec![],
        ..ExportPlan::default()
    };
    let arts = artifacts("wide.png", fixtures::png_rgba(64, 32, RED), &plan);

    let ico_bytes = &find(&arts, "favicon.ico").bytes;
    let dir = ico::IconDir::read(std::io::Cursor::new(ico_bytes)).expect("ico should parse");
    assert!(!dir.entries().is_empty());
    for entry in dir.entries() {
        let frame = entry.decode().expect("frame should decode");
        let (w, h) = (frame.width() as usize, frame.height() as usize);
        let px = frame.rgba_data();
        let alpha = |x: usize, y: usize| px[(y * w + x) * 4 + 3];
        assert_eq!(alpha(0, 0), 0, "{w}x{h}: corner must be transparent");
        assert_eq!(alpha(w / 2, h / 2), 255, "{w}x{h}: centre must be opaque");
    }

    let apple = decode(&find(&arts, "apple-touch-icon.png").bytes);
    assert_eq!((apple.width(), apple.height()), (180, 180));
    assert!(
        row_alpha_all_zero(&apple, 0),
        "apple-touch top row must be transparent"
    );
    assert_eq!(apple.get_pixel(90, 90)[3], 255);
}

/// T6: monochrome outputs follow the same rule and keep the grey values.
#[test]
fn t6_monochrome_padding_is_transparent_and_grey_is_unchanged() {
    let plan = ExportPlan {
        monochrome: true,
        ..png_size_only(32)
    };
    let arts = artifacts("wide.png", fixtures::png_rgba(64, 32, RED), &plan);
    let img = decode(&find(&arts, "mono/favicon-32.png").bytes);
    assert!(
        row_alpha_all_zero(&img, 0),
        "mono top row must be transparent"
    );
    let centre = img.get_pixel(16, 16);
    assert_eq!(centre[3], 255);
    // BT.709 luma of pure red: 0.2126 * 255 = 54.2
    assert!((53..=55).contains(&centre[0]), "grey value {}", centre[0]);
    assert!(
        centre[0] == centre[1] && centre[1] == centre[2],
        "must be grey"
    );
}

/// T7: the preview rasters follow the same rule as the exports.
#[test]
fn t7_preview_rasters_are_letterboxed() {
    let asset = ingest_bytes("wide.png", fixtures::png_rgba(64, 32, RED)).unwrap();
    let cache =
        logolig::services::preview::build_preview(&asset, logolig::ResizeAlgorithm::default())
            .expect("preview should build");
    for (name, r, size) in [
        ("tab_16", &cache.tab_16, 16u32),
        ("icon_120", &cache.icon_120, 120),
    ] {
        assert_eq!((r.width, r.height), (size, size), "{name} dimensions");
        let alpha = |x: u32, y: u32| r.pixels[((y * r.width + x) * 4 + 3) as usize];
        assert!(
            (0..size).all(|x| alpha(x, 0) == 0),
            "{name}: top row must be transparent"
        );
        assert_eq!(
            alpha(size / 2, size / 2),
            255,
            "{name}: centre must be opaque"
        );
    }
}

/// T8: an extreme aspect ratio neither panics nor errors; the fitted side is clamped to 1 px.
#[test]
fn t8_extreme_aspect_ratio_keeps_one_visible_row() {
    let arts = artifacts(
        "strip.png",
        fixtures::png_rgba(1000, 1, RED),
        &png_size_only(16),
    );
    let img = decode(&find(&arts, "favicon-16.png").bytes);
    assert_eq!((img.width(), img.height()), (16, 16));
    let visible_rows = (0..16).filter(|&y| !row_alpha_all_zero(&img, y)).count();
    assert_eq!(visible_rows, 1, "exactly one visible row");
}

/// T9: non-square SVG sources keep their existing letterboxing.
#[test]
fn t9_non_square_svg_still_letterboxes() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="32" viewBox="0 0 64 32"><rect width="64" height="32" fill="#ff0000"/></svg>"##;
    let arts = artifacts("wide.svg", svg.as_bytes().to_vec(), &png_size_only(32));
    let img = decode(&find(&arts, "favicon-32.png").bytes);
    assert_eq!((img.width(), img.height()), (32, 32));
    assert!(row_alpha_all_zero(&img, 0));
    assert!(row_alpha_all_zero(&img, 31));
    assert_eq!(img.get_pixel(16, 16).0, [255, 0, 0, 255]);
}

/// Every raster kind follows the rule, not only PNG: 2:1 WebP and JPEG sources.
#[test]
fn webp_and_jpeg_sources_are_letterboxed_too() {
    let mut webp = Vec::new();
    let mut buf = image::RgbaImage::new(16, 8);
    buf.pixels_mut().for_each(|p| *p = image::Rgba(RED));
    image::DynamicImage::ImageRgba8(buf)
        .write_to(
            &mut std::io::Cursor::new(&mut webp),
            image::ImageFormat::WebP,
        )
        .expect("webp fixture");
    let mut jpeg = Vec::new();
    let mut buf = image::RgbImage::new(16, 8);
    buf.pixels_mut().for_each(|p| *p = image::Rgb([255, 0, 0]));
    image::DynamicImage::ImageRgb8(buf)
        .write_to(
            &mut std::io::Cursor::new(&mut jpeg),
            image::ImageFormat::Jpeg,
        )
        .expect("jpeg fixture");

    for (file, bytes) in [("wide.webp", webp), ("wide.jpg", jpeg)] {
        let arts = artifacts(file, bytes, &png_size_only(32));
        let img = decode(&find(&arts, "favicon-32.png").bytes);
        assert_eq!((img.width(), img.height()), (32, 32), "{file}");
        assert!(
            row_alpha_all_zero(&img, 0),
            "{file}: top row must be transparent"
        );
        assert_eq!(
            img.get_pixel(16, 16)[3],
            255,
            "{file}: centre must be opaque"
        );
    }
}
