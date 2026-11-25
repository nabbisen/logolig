//! The "Sharp" preset keeps corners (RFC 022, job H2b).
//!
//! `corner_threshold` is the minimum turn angle that counts as a corner, so a higher value smooths
//! more. `Sharp` was 80, identical to `Default` (60) on 90 degree corners and rounder on obtuse ones;
//! it is 45 now. The evidence is `.git-exclude/review-request/rfc-022-sharp-spike/` (fixtures,
//! tables, picture sheet); these tests rebuild two of its shapes (a square and a hexagon, hard edged,
//! 256 px, dark blue on white) and pin the result.

use std::sync::Arc;

use logolig::services::vectorize::vectorize;
use logolig::{Rgba8, VtracerPreset};
use vtracer::{ColorImage, ColorMode, Config, Hierarchical};

const INK: [u8; 4] = [30, 60, 120, 255];

/// A 256 px raster: white, with `ink` where the pixel centre is inside `inside`.
fn shape(inside: impl Fn(f64, f64) -> bool) -> Rgba8 {
    let mut px = Vec::with_capacity(256 * 256 * 4);
    for y in 0..256 {
        for x in 0..256 {
            let on = inside(f64::from(x) + 0.5, f64::from(y) + 0.5);
            px.extend_from_slice(if on { &INK } else { &[255, 255, 255, 255] });
        }
    }
    Rgba8::try_from_raw(256, 256, Arc::from(px)).unwrap()
}

fn square() -> Rgba8 {
    shape(|x, y| (64.0..192.0).contains(&x) && (64.0..192.0).contains(&y))
}

/// The regular hexagon of radius 90 around (128, 128), one corner pointing right.
fn hexagon_corners() -> Vec<(f64, f64)> {
    (0..6)
        .map(|k| {
            let a = (60.0 * f64::from(k)).to_radians();
            (128.0 + 90.0 * a.cos(), 128.0 + 90.0 * a.sin())
        })
        .collect()
}

fn hexagon() -> Rgba8 {
    let c = hexagon_corners();
    // Inside a convex polygon: on the left of every edge (the corners run clockwise on screen).
    shape(move |x, y| {
        (0..6).all(|i| {
            let (a, b) = (c[i], c[(i + 1) % 6]);
            (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0) >= 0.0
        })
    })
}

/// vtracer itself, with `config`, on the same pixels `vectorize` hands it.
fn traced_with(rgba: &Rgba8, config: Config) -> String {
    let image = ColorImage {
        pixels: rgba.as_bytes().to_vec(),
        width: rgba.width as usize,
        height: rgba.height as usize,
    };
    format!("{}", vtracer::convert(image, config).unwrap())
}

fn corner(threshold: i32) -> Config {
    Config {
        corner_threshold: threshold,
        ..Config::default()
    }
}

/// `Sharp` is vtracer's default with `corner_threshold` 45; `Default` is vtracer's default (60);
/// `PhotoRich` is unchanged (colour precision 8, speckle 8, corner 45, stacked, colour).
#[test]
fn the_presets_are_the_values_the_spike_measured() {
    let photo_rich = Config {
        color_precision: 8,
        filter_speckle: 8,
        corner_threshold: 45,
        hierarchical: Hierarchical::Stacked,
        color_mode: ColorMode::Color,
        ..Config::default()
    };
    for (name, rgba) in [("square", square()), ("hexagon", hexagon())] {
        for (preset, config) in [
            (VtracerPreset::Sharp, corner(45)),
            (VtracerPreset::Default, corner(60)),
            (VtracerPreset::PhotoRich, photo_rich.clone()),
        ] {
            assert_eq!(
                vectorize(&rgba, preset).unwrap(),
                traced_with(&rgba, config),
                "{name} at {preset:?}"
            );
        }
    }
}

/// Path segments of the whole SVG (every `C` and `L` command).
fn segments(svg: &str) -> usize {
    svg.matches(" C").count() + svg.matches(" L").count()
}

/// The spike: a hexagon has 23 path nodes at `Sharp` (45) against 50 at `Default` (60) (here 38 and 65
/// segments of the whole SVG, a different pixel pattern and a different count); a square is the same
/// at every value.
#[test]
fn sharp_has_no_more_segments_than_default_on_a_hexagon_and_equals_it_on_a_square() {
    let hex = hexagon();
    let (sharp, default) = (
        segments(&vectorize(&hex, VtracerPreset::Sharp).unwrap()),
        segments(&vectorize(&hex, VtracerPreset::Default).unwrap()),
    );
    assert!(sharp <= default, "hexagon: {sharp} vs {default} segments");
    println!("hexagon segments: Sharp {sharp}, Default {default}");
    let sq = square();
    assert_eq!(
        vectorize(&sq, VtracerPreset::Sharp).unwrap(),
        vectorize(&sq, VtracerPreset::Default).unwrap(),
        "a square (90 degree corners) traces the same at 45 and 60"
    );
}

/// The traced outline of the ink path, Bezier curves sampled (48 points a curve).
fn outline(svg: &str) -> Vec<(f64, f64)> {
    let at = svg.find("fill=\"#1E3C78\"").expect("the ink path");
    let tag = &svg[svg[..at].rfind("<path").unwrap()..at + svg[at..].find("/>").unwrap()];
    let d = tag.split("d=\"").nth(1).unwrap().split('"').next().unwrap();
    let t = tag
        .split("translate(")
        .nth(1)
        .unwrap()
        .split(')')
        .next()
        .unwrap();
    let (tx, ty) = t
        .split_once(',')
        .map(|(a, b)| (a.parse::<f64>().unwrap(), b.parse::<f64>().unwrap()))
        .unwrap();
    let spaced = d
        .replace('M', " M ")
        .replace('C', " C ")
        .replace('Z', " Z ");
    let mut tok = spaced.split_whitespace().peekable();
    let (mut pts, mut cur) = (Vec::new(), (0.0, 0.0));
    while let Some(c) = tok.next() {
        let mut n = || {
            tok.next_if(|t| t.parse::<f64>().is_ok())
                .map(|t| t.parse::<f64>().unwrap())
        };
        if c == "M" {
            cur = (n().unwrap(), n().unwrap());
            pts.push(cur);
        } else if c == "C" {
            let v: Vec<f64> = (0..6).map(|_| n().unwrap()).collect();
            for s in 1..=48 {
                let (t, u) = (f64::from(s) / 48.0, 1.0 - f64::from(s) / 48.0);
                let b = |p0: f64, p1: f64, p2: f64, p3: f64| {
                    u * u * u * p0 + 3.0 * u * u * t * p1 + 3.0 * u * t * t * p2 + t * t * t * p3
                };
                pts.push((b(cur.0, v[0], v[2], v[4]), b(cur.1, v[1], v[3], v[5])));
            }
            cur = (v[4], v[5]);
        }
    }
    pts.iter().map(|p| (p.0 + tx, p.1 + ty)).collect()
}

/// The interior angle of `outline` at `corner` in degrees: from the outline point nearest the corner to
/// the points 8 px of arc away on both sides (the method of the spike's `measure.py`).
fn angle_at(outline: &[(f64, f64)], corner: (f64, f64)) -> f64 {
    let d = |a: (f64, f64), b: (f64, f64)| (a.0 - b.0).hypot(a.1 - b.1);
    let n = outline.len();
    let i = (0..n)
        .min_by(|&a, &b| d(outline[a], corner).total_cmp(&d(outline[b], corner)))
        .unwrap();
    let walk = |step: usize| {
        let (mut j, mut arc) = (i, 0.0);
        while arc < 8.0 {
            let k = (j + step) % n;
            arc += d(outline[j], outline[k]);
            j = k;
        }
        outline[j]
    };
    let (a, b, p) = (walk(1), walk(n - 1), outline[i]);
    let (v1, v2) = ((a.0 - p.0, a.1 - p.1), (b.0 - p.0, b.1 - p.1));
    ((v1.0 * v2.0 + v1.1 * v2.1) / (v1.0.hypot(v1.1) * v2.0.hypot(v2.1)))
        .acos()
        .to_degrees()
}

/// The claim of the label, as a number: the true interior angle of the hexagon is 120; at `Sharp` the
/// traced corners average at most 130 degrees (the spike, rasterised by another program: 126; here 125), at `Default` they are rounded (the spike 145; here 134, a different pixel pattern of the same shape).
#[test]
fn the_hexagon_keeps_its_corners_at_sharp_and_loses_them_at_default() {
    let mean = |preset| {
        let line = outline(&vectorize(&hexagon(), preset).unwrap());
        hexagon_corners()
            .iter()
            .map(|&c| angle_at(&line, c))
            .sum::<f64>()
            / 6.0
    };
    let (sharp, default) = (mean(VtracerPreset::Sharp), mean(VtracerPreset::Default));
    println!("hexagon mean interior angle: Sharp {sharp:.0}, Default {default:.0}");
    assert!(sharp <= 130.0, "Sharp keeps the corners: {sharp:.0}");
    assert!(
        default > 130.0 && default - sharp >= 5.0,
        "Default rounds them: {default:.0}"
    );
}
