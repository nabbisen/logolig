//! The focus ring, drawn (RFC 021 amendment of 2026-11-15, RFC 027; job J10 step 3).
//!
//! The rule: keyboard focus is a **neutral** ring outside the control, so it can never merge with a
//! selection mark. "Snapshot per theme" is measured, not compared with a picture: the interface is
//! drawn twice on the CPU renderer, with and without the focus, and the pixels that differ are the
//! ring. They must exist, lie **outside** the control (nothing inside it changes: the box, the tick
//! and the label are as they were), stay within a few pixels of it, and be a neutral grey (no
//! accent hue), on both themes.

use iced::{Size, Theme};

use super::support::{Case, Combo, Probe, probe_of};
use crate::app::Message;
use logolig_i18n::Locale;

const SCALE: f32 = 2.0;
const SIZE: Size = Size::new(720.0, 700.0);

/// Pixels that differ between two renders of the same size: (x, y, colour of the focused one).
fn differing(a: &[u8], b: &[u8], width_px: usize) -> Vec<(f32, f32, [u8; 3])> {
    a.as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0.iter())
        .enumerate()
        .filter(|(_, (x, y))| x[..3] != y[..3])
        .map(|(i, (_, y))| {
            (
                (i % width_px) as f32 / SCALE,
                (i / width_px) as f32 / SCALE,
                [y[0], y[1], y[2]],
            )
        })
        .collect()
}

fn ring_of(case: Case, dark: bool, target: usize) -> (Vec<(f32, f32, [u8; 3])>, iced::Rectangle) {
    let state = Combo {
        case,
        size: SIZE,
        locale: Locale::En,
        dark,
    }
    .state();
    let theme = if dark { Theme::Dark } else { Theme::Light };
    let width_px = (SIZE.width * SCALE) as usize;

    let mut plain = probe_of(&state, SIZE);
    let before = plain.rgba(&theme, SCALE);
    let mut focused: Probe<'_, Message> = probe_of(&state, SIZE);
    focused.focus_target(target);
    let bounds = focused.focus_targets()[target].bounds;
    let after = focused.rgba(&theme, SCALE);
    (differing(&before, &after, width_px), bounds)
}

#[test]
fn the_ring_is_neutral_and_outside_the_control_on_both_themes() {
    // The Settings page (target 0: the language group) and Customize (target 1: "32 × 32",
    // a checked box, the case where an accent ring would merge with the fill).
    for (case, target, name) in [
        (Case::Settings, 0, "the language group"),
        (Case::CustomizeAdvanced, 1, "the checked 32 x 32 box"),
    ] {
        for dark in [false, true] {
            let theme = if dark { "Dark" } else { "Light" };
            let (ring, bounds) = ring_of(case, dark, target);
            assert!(
                ring.len() > 40,
                "{name}, {theme}: no ring is drawn ({} pixels)",
                ring.len()
            );

            // Outside the control: nothing inside its bounds changes, so the mark and the label
            // are exactly as without focus and the ring cannot touch them.
            let inside: Vec<_> = ring
                .iter()
                .filter(|(x, y, _)| {
                    bounds.contains(iced::Point::new(*x + 0.25, *y + 0.25))
                        && x - bounds.x >= 1.0
                        && y - bounds.y >= 1.0
                        && bounds.x + bounds.width - x >= 2.0
                        && bounds.y + bounds.height - y >= 2.0
                })
                .collect();
            assert!(
                inside.is_empty(),
                "{name}, {theme}: {} ring pixel(s) inside the control, first {:?}",
                inside.len(),
                inside.first()
            );

            // Complete: a stroke on every side, outside the control. A scroll area around the
            // control clips a ring that reaches past its edge, and a clipped ring is no ring.
            let sides = [
                (
                    "left",
                    ring.iter().filter(|(x, _, _)| *x < bounds.x).count(),
                ),
                (
                    "right",
                    ring.iter()
                        .filter(|(x, _, _)| *x > bounds.x + bounds.width)
                        .count(),
                ),
                ("top", ring.iter().filter(|(_, y, _)| *y < bounds.y).count()),
                (
                    "bottom",
                    ring.iter()
                        .filter(|(_, y, _)| *y > bounds.y + bounds.height)
                        .count(),
                ),
            ];
            for (side, count) in sides {
                assert!(
                    count > 20,
                    "{name}, {theme}: the ring has no {side} side outside the control ({count} px): clipped?"
                );
            }

            // Close to the control: within 6 logical pixels of its bounds.
            let far = ring.iter().any(|(x, y, _)| {
                *x < bounds.x - 6.0
                    || *y < bounds.y - 6.0
                    || *x > bounds.x + bounds.width + 6.0
                    || *y > bounds.y + bounds.height + 6.0
            });
            assert!(
                !far,
                "{name}, {theme}: ring pixels farther than 6 px from the control"
            );

            // Neutral: the strongest ring pixels are grey (no accent hue): R, G and B within 10.
            let strongest = ring
                .iter()
                .map(|(_, _, c)| *c)
                .filter(|c| {
                    let luma = (c[0] as u32 + c[1] as u32 + c[2] as u32) / 3;
                    if dark { luma > 200 } else { luma < 55 }
                })
                .collect::<Vec<_>>();
            assert!(
                strongest.len() > 20,
                "{name}, {theme}: no solid ring pixel found ({} of {})",
                strongest.len(),
                ring.len()
            );
            for c in strongest {
                let spread = c.iter().max().unwrap() - c.iter().min().unwrap();
                assert!(
                    spread <= 10,
                    "{name}, {theme}: a ring pixel is not grey: {c:?}"
                );
            }
        }
    }
}
