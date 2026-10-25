//! Bounded tracing cost (RFC 030): the raster handed to the tracer is scaled down so that its
//! longest side is at most `VECTORIZE_MAX_SIDE`; nothing below the cap changes, and the raster
//! outputs are never touched.

mod fixtures;

use std::sync::Arc;

use logolig::services::exporter::run_in_memory;
use logolig::services::ingest::ingest_bytes;
use logolig::services::vectorize::{
    VECTORIZE_MAX_SIDE, trace_dimensions, vectorize, vectorize_with_cap,
};
use logolig::{ExportPlan, InMemoryArtifact, ResizeAlgorithm, Rgba8, VtracerPreset};

const ALGO: ResizeAlgorithm = ResizeAlgorithm::Lanczos3;

fn noise(width: u32, height: u32) -> Rgba8 {
    let mut seed = 7u64;
    let mut next = move || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (seed >> 56) as u8
    };
    let mut px = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..width * height {
        px.extend_from_slice(&[next(), next(), next(), 255]);
    }
    Rgba8::try_from_raw(width, height, Arc::from(px)).unwrap()
}

/// A flat two-colour checker; the golden values below were captured from the code as it was
/// before the cap existed (`vectorize` on this exact image).
fn checker(side: u32) -> Rgba8 {
    let mut px = Vec::with_capacity((side * side * 4) as usize);
    for y in 0..side {
        for x in 0..side {
            let c = if (x / 32 + y / 32) % 2 == 0 {
                [200u8, 60, 40, 255]
            } else {
                [40u8, 60, 200, 255]
            };
            px.extend_from_slice(&c);
        }
    }
    Rgba8::try_from_raw(side, side, Arc::from(px)).unwrap()
}

/// FNV-1a 64: a fixed, dependency-free hash (std's `DefaultHasher` is not stable across releases).
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Declared `(width, height)` of the root `<svg>` element.
fn declared_size(svg: &str) -> (u32, u32) {
    let tag_start = svg.find("<svg").expect("svg root element");
    let tag = &svg[tag_start..svg[tag_start..].find('>').unwrap() + tag_start];
    let attr = |name: &str| -> u32 {
        let key = format!(" {name}=\"");
        let from = tag.find(&key).expect("attribute") + key.len();
        let rest = &tag[from..];
        rest[..rest.find('"').unwrap()].parse().unwrap()
    };
    (attr("width"), attr("height"))
}

// 1 -----------------------------------------------------------------------

#[test]
fn trace_dimensions_table() {
    let table = [
        ((512, 512, 512), (512, 512)), // exactly at the cap: unchanged
        ((100, 50, 512), (100, 50)),   // below the cap: unchanged
        ((1, 1, 512), (1, 1)),
        ((1024, 512, 512), (512, 256)),
        ((512, 1024, 512), (256, 512)),
        ((4000, 3000, 512), (512, 384)),
        ((3000, 4000, 512), (384, 512)),
        ((5000, 1, 512), (512, 1)), // never 0
        ((1, 5000, 512), (1, 512)),
        ((1400, 1400, 512), (512, 512)),
        ((10, 5, 1), (1, 1)), // cap 1: the long side is 1, the short side never below 1
        ((513, 513, 512), (512, 512)), // just above the cap
    ];
    for ((w, h, cap), want) in table {
        assert_eq!(trace_dimensions(w, h, cap), want, "{w}x{h} cap {cap}");
    }
}

#[test]
fn trace_dimensions_keeps_the_aspect_ratio_within_rounding() {
    for (w, h) in [(4000u32, 3000u32), (1920, 1080), (1001, 777), (2048, 100)] {
        let (tw, th) = trace_dimensions(w, h, VECTORIZE_MAX_SIDE);
        assert!(tw.max(th) <= VECTORIZE_MAX_SIDE, "{w}x{h}");
        assert!(tw >= 1 && th >= 1);
        // |w/h - tw/th| is bounded by one pixel of the short side.
        let exact_short = f64::from(w.min(h)) * f64::from(tw.max(th)) / f64::from(w.max(h));
        assert!(
            (f64::from(tw.min(th)) - exact_short).abs() <= 0.5 + 1e-9,
            "{w}x{h}"
        );
    }
}

#[test]
fn the_default_cap_is_512() {
    assert_eq!(VECTORIZE_MAX_SIDE, 512);
}

// 2 -----------------------------------------------------------------------

#[test]
fn a_flat_logo_below_the_cap_traces_exactly_as_before() {
    // Captured from the unmodified `vectorize` (same image, all three presets).
    const GOLDEN_LEN: usize = 10216;
    const GOLDEN_FNV: u64 = 0x7afc_9cc0_9d3d_fa01;
    let img = checker(256);
    for preset in [
        VtracerPreset::Default,
        VtracerPreset::Sharp,
        VtracerPreset::PhotoRich,
    ] {
        for svg in [
            vectorize(&img, preset).unwrap(),
            vectorize_with_cap(&img, preset, VECTORIZE_MAX_SIDE, ALGO).unwrap(),
        ] {
            assert_eq!(svg.len(), GOLDEN_LEN, "{preset:?}");
            assert_eq!(fnv1a64(svg.as_bytes()), GOLDEN_FNV, "{preset:?}");
        }
    }
}

// 3 -----------------------------------------------------------------------

#[test]
fn the_cap_scales_the_traced_image_and_shrinks_the_svg() {
    let img = noise(256, 256);
    let capped = vectorize_with_cap(&img, VtracerPreset::Default, 64, ALGO).unwrap();
    let uncapped = vectorize_with_cap(&img, VtracerPreset::Default, u32::MAX, ALGO).unwrap();
    let (w, h) = declared_size(&capped);
    assert!(w <= 64 && h <= 64, "declared {w}x{h}");
    assert_eq!(declared_size(&uncapped), (256, 256), "no cap: source size");
    assert!(
        capped.len() < uncapped.len(),
        "capped {} bytes must be smaller than uncapped {} bytes",
        capped.len(),
        uncapped.len()
    );
}

// 4 -----------------------------------------------------------------------

#[test]
fn the_cap_keeps_a_two_to_one_ratio() {
    let img = noise(256, 128);
    let svg = vectorize_with_cap(&img, VtracerPreset::Default, 64, ALGO).unwrap();
    let (w, h) = declared_size(&svg);
    assert_eq!((w, h), (64, 32));
    assert!(w.abs_diff(2 * h) <= 1);
}

// 5 -----------------------------------------------------------------------

fn find<'a>(arts: &'a [InMemoryArtifact], path: &str) -> &'a InMemoryArtifact {
    arts.iter()
        .find(|a| a.relative_path.to_string_lossy().replace('\\', "/") == path)
        .unwrap_or_else(|| panic!("artifact {path} not found"))
}

#[test]
fn the_exporter_traces_a_large_raster_capped_and_leaves_raster_outputs_untouched() {
    // 640 x 480 is above the cap (512) and flat, so tracing it stays fast.
    let png = fixtures::png_rgba(640, 480, [200, 60, 40, 255]);
    let asset = ingest_bytes("big.png", png).unwrap();
    let with_svg = ExportPlan {
        png_sizes: vec![32, 192],
        include_svg: true,
        ..ExportPlan::default()
    };
    let without_svg = ExportPlan {
        include_svg: false,
        ..with_svg.clone()
    };
    let a = run_in_memory(&asset, &with_svg).unwrap();
    let b = run_in_memory(&asset, &without_svg).unwrap();

    let svg = String::from_utf8(find(&a, "favicon.svg").bytes.clone()).unwrap();
    assert_eq!(declared_size(&svg), (512, 384), "traced at the capped size");
    assert!(b.iter().all(|x| !x.relative_path.ends_with("favicon.svg")));

    for name in ["favicon-32.png", "favicon-192.png", "favicon.ico"] {
        assert_eq!(
            find(&a, name).bytes,
            find(&b, name).bytes,
            "{name} must not depend on the tracing cap"
        );
    }
}

// 6 -----------------------------------------------------------------------

#[test]
fn invalid_input_is_an_error_not_a_panic() {
    let empty = Rgba8::try_from_raw(0, 4, Arc::from(Vec::<u8>::new())).unwrap();
    assert!(vectorize_with_cap(&empty, VtracerPreset::Default, 512, ALGO).is_err());
    assert!(vectorize(&empty, VtracerPreset::Default).is_err());
    let ok = checker(32);
    assert!(
        vectorize_with_cap(&ok, VtracerPreset::Default, 0, ALGO).is_err(),
        "a cap of 0 is invalid"
    );
}
