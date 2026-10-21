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
    let arts = artifacts("wide.png", fixtures::png_rgba(64, 32, RED), &png_size_only(32));
    let img = decode(&find(&arts, "favicon-32.png").bytes);
    assert_eq!((img.width(), img.height()), (32, 32));
    assert!(row_alpha_all_zero(&img, 0), "top row must be transparent padding");
    assert!(row_alpha_all_zero(&img, 31), "bottom row must be transparent padding");
    assert_eq!(img.get_pixel(16, 16)[3], 255, "centre must be opaque");
    let (x0, y0, x1, y1) = visible_box(&img);
    let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
    assert!((31..=33).contains(&w) && (15..=17).contains(&h), "visible box {w}x{h}");
}
