//! What each resize algorithm does to a hard edge, in numbers (RFC 034, J23).
//!
//! The Customize page describes each algorithm in one plain line ("crisp", "softer", "fewer halos").
//! A word on that page must follow a measurement, so this file measures one synthetic edge and pins
//! the **ranking** the words rely on. Method: a 256 × 256 source, dark grey (32) on the left and
//! light grey (224) on the right, the edge at x = 109.5 (so it falls between output pixels at both
//! sizes); scaled to 32 and to 16 px with each algorithm; the middle row is read.
//!
//! * **Transition width**: the distance in output pixels between the 10 % and the 90 % level of the
//!   step, by linear interpolation. Smaller is crisper.
//! * **Overshoot**: how far the row goes beyond the two grey levels, as a share of the step (the
//!   halo). Greys are used and not pure black and white because 8-bit output clips an overshoot at 0
//!   and 255. Smaller is cleaner.
//!
//! `cargo test -p logolig --test resize_edges -- --nocapture` prints the table.

use std::sync::Arc;

use logolig::ResizeAlgorithm;
use logolig::domain::Rgba8;
use logolig::services::resize::resize;

const LOW: f32 = 32.0;
const HIGH: f32 = 224.0;
const SOURCE: u32 = 256;
/// The edge, in source pixels (a half-pixel offset keeps it off the output grid).
const EDGE_X: u32 = 110;

#[derive(Debug, Clone, Copy)]
struct EdgeMeasure {
    /// 10–90 % transition width, in output pixels.
    width: f32,
    /// Largest excursion beyond the two levels, as a share of the step (0.0 = none).
    overshoot: f32,
}

fn edge_source() -> Rgba8 {
    let mut px = Vec::with_capacity((SOURCE * SOURCE * 4) as usize);
    for _y in 0..SOURCE {
        for x in 0..SOURCE {
            let v = if x < EDGE_X { LOW } else { HIGH } as u8;
            px.extend_from_slice(&[v, v, v, 255]);
        }
    }
    Rgba8::try_from_raw(SOURCE, SOURCE, Arc::from(px)).unwrap()
}

/// Measure the middle row of `out`.
fn measure(out: &Rgba8) -> EdgeMeasure {
    let w = out.width as usize;
    let row = (out.height as usize / 2) * w * 4;
    let level: Vec<f32> = (0..w)
        .map(|x| (out.pixels[row + x * 4] as f32 - LOW) / (HIGH - LOW))
        .collect();
    // Position where the profile first crosses `t` (linear interpolation between pixels).
    let cross = |t: f32| -> f32 {
        for x in 1..w {
            if level[x - 1] < t && level[x] >= t {
                return (x - 1) as f32 + (t - level[x - 1]) / (level[x] - level[x - 1]);
            }
        }
        f32::NAN
    };
    let overshoot = level
        .iter()
        .map(|v| (v - 1.0).max(-v).max(0.0))
        .fold(0.0_f32, f32::max);
    EdgeMeasure {
        width: cross(0.9) - cross(0.1),
        overshoot,
    }
}

fn measured(algorithm: ResizeAlgorithm, size: u32) -> EdgeMeasure {
    measure(&resize(&edge_source(), size, size, algorithm).unwrap())
}

/// The numbers behind each plain line on the Customize page, per output size:
///
/// | word on the page | what it rests on |
/// |---|---|
/// | Lanczos3 "crisp and detailed" | the narrowest transition of the smooth methods |
/// | Mitchell-Netravali "softer, fewer halos" | wider than Lanczos3 and Catmull-Rom, less overshoot than both |
/// | Catmull-Rom "slightly fewer halos than Lanczos3" | less overshoot than Lanczos3, a little wider |
/// | Bilinear "softest, no halos" | the widest transition, no overshoot |
/// | Nearest "keeps hard pixel edges" | no interpolation: no overshoot, the narrowest of all |
///
/// If an algorithm or the resizer changes and a word stops being true, this fails and the page text
/// has to follow.
#[test]
fn the_edge_table_and_the_ranking_the_page_words_rely_on() {
    for size in [32, 16] {
        let m = |a| measured(a, size);
        let (lanczos, mitchell, catmull, bilinear, nearest) = (
            m(ResizeAlgorithm::Lanczos3),
            m(ResizeAlgorithm::MitchellNetravali),
            m(ResizeAlgorithm::CatmullRom),
            m(ResizeAlgorithm::Bilinear),
            m(ResizeAlgorithm::Nearest),
        );
        println!("--- {size} px: 10-90 % width, overshoot ---");
        for (name, e) in [
            ("Lanczos3", lanczos),
            ("MitchellNetravali", mitchell),
            ("CatmullRom", catmull),
            ("Bilinear", bilinear),
            ("Nearest", nearest),
        ] {
            println!(
                "{name:<18} width {:.2} px   overshoot {:.1} %",
                e.width,
                e.overshoot * 100.0
            );
        }
        // Crisp: Nearest (no transition) < Lanczos3 < Catmull-Rom < Mitchell-Netravali < Bilinear.
        assert!(nearest.width < lanczos.width, "{size}: nearest");
        assert!(lanczos.width < catmull.width, "{size}: lanczos < catmull");
        assert!(catmull.width < mitchell.width, "{size}: catmull < mitchell");
        assert!(
            mitchell.width < bilinear.width,
            "{size}: mitchell < bilinear"
        );
        // Halos: Lanczos3 > Catmull-Rom > Mitchell-Netravali > none (Bilinear, Nearest).
        assert!(lanczos.overshoot > catmull.overshoot, "{size}: halo l > c");
        assert!(catmull.overshoot > mitchell.overshoot, "{size}: halo c > m");
        assert!(mitchell.overshoot > 0.005, "{size}: mitchell has some halo");
        assert!(bilinear.overshoot < 0.005, "{size}: bilinear has none");
        assert!(nearest.overshoot < 0.005, "{size}: nearest has none");
        // A halo is visible on the page's claim: Lanczos3's is at least three times Mitchell's.
        assert!(
            lanczos.overshoot > 3.0 * mitchell.overshoot,
            "{size}: ratio"
        );
    }
}
