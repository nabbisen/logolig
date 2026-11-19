//! The navigation items (RFC 027 J14b step 1): the side bar on the desktop and the bottom bar on a
//! phone, three items each, `snora::control`s. A click and Enter or Space publish
//! `NavPageSelected(page)`; the accent bar, bold label and fill of RFC 021 are guarded by
//! `nav_cue.rs`, which must stay green.

use iced::Size;
use logolig_i18n::Locale;

use super::controls::{Region, press_three_ways};
use super::support::{Case, Combo};

const DESKTOP: Size = Size::new(1280.0, 720.0);
const PHONE: Size = Size::new(360.0, 640.0);

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
fn each_navigation_item_is_a_tab_stop_and_enter_space_and_a_click_navigate() {
    for size in [DESKTOP, PHONE] {
        // From Home, Customize and Settings: the items are on every page.
        for case in [Case::Home, Case::CustomizeAdvanced, Case::Settings] {
            let s = state(case, size, Locale::En);
            for (label, page) in [
                ("Home", "Home"),
                ("Customize", "Customize"),
                ("Settings", "Settings"),
            ] {
                press_three_ways(
                    &s,
                    size,
                    (Region::Navigation, label, 0),
                    &format!("NavPageSelected({page})"),
                );
            }
        }
        // Japanese labels.
        let s = state(Case::Home, size, Locale::Ja);
        for (label, page) in [
            ("ホーム", "Home"),
            ("カスタマイズ", "Customize"),
            ("設定", "Settings"),
        ] {
            press_three_ways(
                &s,
                size,
                (Region::Navigation, label, 0),
                &format!("NavPageSelected({page})"),
            );
        }
    }
}

/// The helper tells the page's controls from the navigation by region, also where an unscrolled
/// page overlaps the bottom bar's coordinates (a phone): at a point that lies in a page stop **and**
/// in a navigation item, each region finds its own stop.
#[test]
fn a_page_stop_and_a_navigation_item_at_one_point_are_told_apart() {
    use super::controls::stop_at;
    use super::support::probe_of;
    let state = state(Case::CustomizeAdvanced, PHONE, Locale::En);
    let mut p = probe_of(&state, PHONE);
    let stops = p.focus_targets();
    let total = stops.len();
    let (page, nav) = (0..total)
        .filter(|i| !p.is_navigation_stop(*i, total))
        .flat_map(|i| {
            (0..total)
                .filter(|j| p.is_navigation_stop(*j, total))
                .map(move |j| (i, j))
        })
        .find(|(i, j)| stops[*i].bounds.intersects(&stops[*j].bounds))
        .expect("a page stop overlaps a navigation item in the unscrolled layout of a phone");
    let overlap = stops[page]
        .bounds
        .intersection(&stops[nav].bounds)
        .expect("overlap");
    let at = overlap.center();
    assert_eq!(stop_at(&mut p, at, Region::Page), Some(page));
    assert_eq!(stop_at(&mut p, at, Region::Navigation), Some(nav));
    assert!(p.is_navigation_stop(nav, total) && !p.is_navigation_stop(page, total));
}
