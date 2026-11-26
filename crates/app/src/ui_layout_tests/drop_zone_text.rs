//! The drop-zone card's text fits (RFC 038, job W1).
//!
//! The wording became true on every system ("Choose a PNG, SVG, WebP, or JPEG"; "Click anywhere in
//! this area. You can also drop a file here, if your system allows it."), and the secondary line is
//! longer than before, in Japanese too. At every size the layout tests cover, and at the sizes the
//! largest interface step leads to, each of the card's three texts must lie fully inside the card and
//! be fully visible (not clipped by the window or an area), and the card must stay one Tab stop.

use iced::{Rectangle, Size};
use logolig::MessageKey;
use logolig_i18n::{Locale, Translator};

use super::support::{Case, Combo, Kind, SIZES, probe_of};

/// The window sizes of the matrix, plus the narrow ones the interface sizes lead to: the minimum
/// window (360 x 480), and the 1042 x 1144 window of the real runs at Large (125 %) and Extra large
/// (150 %), which the layout sees as 833.6 x 915.2 and 694.7 x 762.7 scaled pixels.
fn sizes() -> Vec<Size> {
    let mut all: Vec<Size> = SIZES.iter().map(|&(w, h)| Size::new(w, h)).collect();
    all.extend([
        Size::new(360.0, 480.0),
        Size::new(1042.0 / 1.25, 1144.0 / 1.25),
        Size::new(1042.0 / 1.5, 1144.0 / 1.5),
    ]);
    all
}

/// **Found by this test, not caused by the wording:** at the smallest window (360 x 480) with the
/// history card on the Home page, the card is squeezed to 235 px (214 in Japanese) and its last
/// line, "Accepted: .png, ..." (`DropZoneAcceptedFormats`), is squeezed to nothing. It is the same with
/// the 1.29.0 strings (`evidence/fit-test-on-the-old-strings.txt`); the layout may not change in
/// job W1, so it is listed here (both languages) and reported. The test fails if the entry stops being
/// true, so it cannot rot into a hiding place.
const KNOWN_SQUEEZE: [(f32, f32, Case, MessageKey); 1] = [(
    360.0,
    480.0,
    Case::HomeWithHistory,
    MessageKey::DropZoneAcceptedFormats,
)];

fn inside(inner: Rectangle, outer: Rectangle) -> bool {
    inner.x >= outer.x - 0.5
        && inner.y >= outer.y - 0.5
        && inner.x + inner.width <= outer.x + outer.width + 0.5
        && inner.y + inner.height <= outer.y + outer.height + 0.5
}

#[test]
fn the_card_text_is_inside_the_card_and_not_clipped_at_every_size_in_both_languages() {
    let mut problems = Vec::new();
    let mut checked = 0;
    let mut known_seen = 0;
    for size in sizes() {
        for (locale, case) in [Locale::En, Locale::Ja]
            .into_iter()
            .flat_map(|l| [(l, Case::Home), (l, Case::HomeWithHistory)])
        {
            let t = Translator::for_locale(locale);
            let state = Combo {
                case,
                size,
                locale,
                dark: false,
            }
            .state();
            let mut p = probe_of(&state, size);
            let stops = p.focus_targets();
            let page: Vec<_> = (0..stops.len())
                .filter(|&i| !p.is_navigation_stop(i, stops.len()))
                .collect();
            let want_stops = if case == Case::Home { 1 } else { 2 }; // the card, then "View results →"
            if page.len() != want_stops {
                problems.push(format!(
                    "{}x{} {locale:?} {case:?}: {} page stops, want {want_stops}",
                    size.width,
                    size.height,
                    page.len()
                ));
                continue;
            }
            let card = stops[page[0]].bounds;
            let seen = p.walk();
            for key in [
                MessageKey::DropZoneHeadline,
                MessageKey::DropZoneSecondary,
                MessageKey::DropZoneAcceptedFormats,
            ] {
                let want = t.t(key);
                let Some(text) = seen.iter().find(|s| s.kind == Kind::Text && s.text == want)
                else {
                    problems.push(format!(
                        "{}x{} {locale:?}: no text {want:?}",
                        size.width, size.height
                    ));
                    continue;
                };
                checked += 1;
                let known = KNOWN_SQUEEZE.iter().any(|&(w, h, c, k)| {
                    size.width == w && size.height == h && case == c && key == k
                });
                let clipped = text.visible.is_none_or(|v| {
                    (v.width - text.bounds.width).abs() > 0.5
                        || (v.height - text.bounds.height).abs() > 0.5
                });
                if known {
                    if !inside(text.bounds, card) || clipped {
                        known_seen += 1;
                    }
                } else if !inside(text.bounds, card) || clipped {
                    problems.push(format!(
                        "{}x{} {locale:?}: {key:?} at {:?} is outside the card {card:?} or clipped (visible {:?})",
                        size.width, size.height, text.bounds, text.visible
                    ));
                }
                if key == MessageKey::DropZoneSecondary {
                    println!(
                        "secondary {}x{} {locale:?}: {:.0} x {:.0} px in a card {:.0} x {:.0}",
                        size.width,
                        size.height,
                        text.bounds.width,
                        text.bounds.height,
                        card.width,
                        card.height
                    );
                }
            }
        }
    }
    assert!(checked >= 3 * 11 * 4, "only {checked} texts were checked");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert_eq!(
        known_seen,
        2 * KNOWN_SQUEEZE.len(),
        "the known squeeze changed (it is gone, or it spread): update KNOWN_SQUEEZE"
    );
}
