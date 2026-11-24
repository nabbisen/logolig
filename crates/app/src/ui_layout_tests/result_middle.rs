//! The Result screen's scrolling middle keeps room (RFC 034 J22, from the review note of RFC 026): with
//! the header ("Back") and the footer ("Download all (ZIP)") fixed, and the bottom bar on a phone,
//! the area that scrolls the result cards is at least [`MIN_MIDDLE`] px high at every matrix size, in
//! both languages, with 7 and 15 cards, the preview closed and open.

use super::support::{Case, Combo, Kind, SIZES, probe_of};
use iced::Size;
use logolig_i18n::Locale;

/// The least height of the scrolling middle.
const MIN_MIDDLE: f32 = 200.0;

/// The heights of the scroll areas of the Result screen at `size`.
fn middle_heights(case: Case, size: Size, locale: Locale) -> Vec<f32> {
    let state = Combo {
        case,
        size,
        locale,
        dark: false,
    }
    .state();
    let mut p = probe_of(&state, size);
    p.walk()
        .iter()
        .filter(|s| s.kind == Kind::Scrollable)
        .map(|s| s.bounds.height)
        .collect()
}

#[test]
fn the_scrolling_middle_of_the_result_screen_is_at_least_200_px_at_every_matrix_size() {
    let mut low = Vec::new();
    let mut measured = 0;
    for case in [
        Case::ResultDefault,
        Case::ResultAllOptions,
        Case::ResultPreview,
        Case::ResultPreviewChecker,
    ] {
        for (w, h) in SIZES {
            for locale in Locale::all() {
                let heights = middle_heights(case, Size::new(w, h), locale);
                assert_eq!(
                    heights.len(),
                    1,
                    "{case:?} {w}x{h}: one scroll area, found {heights:?}"
                );
                measured += 1;
                if heights[0] < MIN_MIDDLE {
                    low.push(format!(
                        "[{case:?} {w}x{h} {}] the middle is {:.0} px high",
                        locale.as_bcp47(),
                        heights[0]
                    ));
                }
            }
        }
    }
    assert!(measured >= 56, "only {measured} combinations measured");
    assert!(
        low.is_empty(),
        "{} below {MIN_MIDDLE} px:\n{}",
        low.len(),
        low.join("\n")
    );
}
