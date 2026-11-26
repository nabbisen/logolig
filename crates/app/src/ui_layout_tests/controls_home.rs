//! The Home page's controls (RFC 027 J14b step 2): the drop-zone card, one Tab stop whose Enter,
//! Space and click all ask for the file chooser, and "View results →" on the history card.

use iced::Size;
use logolig_i18n::Locale;

use super::controls::{Region, press_three_ways};
use super::support::{Case, Combo};

fn state(case: Case, size: Size, locale: Locale) -> crate::app::AppState {
    Combo {
        case,
        size,
        locale,
        dark: false,
    }
    .state()
}

#[test]
fn the_drop_zone_card_is_one_tab_stop_and_asks_for_the_file_chooser() {
    for size in [Size::new(1280.0, 720.0), Size::new(360.0, 640.0)] {
        for (locale, headline) in [
            (Locale::En, "Choose a PNG, SVG, WebP, or JPEG"),
            (Locale::Ja, "PNG / SVG / WebP / JPEG を選択"),
        ] {
            for case in [Case::Home, Case::HomeWithHistory] {
                let s = state(case, size, locale);
                press_three_ways(&s, size, (Region::Page, headline, 0), "PickFileRequested");
            }
        }
    }
}

#[test]
fn view_results_is_a_tab_stop_and_shows_the_last_result() {
    for size in [Size::new(1280.0, 720.0), Size::new(360.0, 640.0)] {
        for (locale, label) in [(Locale::En, "View results →"), (Locale::Ja, "結果を見る →")]
        {
            let s = state(Case::HomeWithHistory, size, locale);
            press_three_ways(
                &s,
                size,
                (Region::Page, label, 0),
                "ShowLastResultRequested",
            );
        }
    }
}

/// The page's two stops are in layout order: the card, then "View results →".
#[test]
fn the_card_comes_before_view_results_in_the_tab_order() {
    use super::support::probe_of;
    let size = Size::new(1280.0, 720.0);
    let s = state(Case::HomeWithHistory, size, Locale::En);
    let mut p = probe_of(&s, size);
    let stops = p.focus_targets();
    let page: Vec<_> = (0..stops.len())
        .filter(|i| !p.is_navigation_stop(*i, stops.len()))
        .map(|i| stops[i].bounds)
        .collect();
    assert_eq!(page.len(), 2, "the card and \"View results →\"");
    assert!(page[0].y < page[1].y, "the card is above the history card");
}
