//! The Customize page's buttons (RFC 027 J14b step 4): "+ Add another size", the "Advanced
//! settings" header (open or closed) and "Reset to defaults" (the fixed footer). Each is a Tab stop
//! and Enter, Space and a click publish the one message the button published. With these the page
//! has no control left outside the Tab order.

use iced::Size;
use logolig_i18n::Locale;

use super::controls::{Region, press_three_ways};
use super::support::{Case, Combo};

const DESKTOP: Size = Size::new(1280.0, 4000.0);
const PHONE: Size = Size::new(360.0, 4000.0);

fn state(size: Size, locale: Locale, extras_open: bool) -> crate::app::AppState {
    let mut state = Combo {
        case: Case::CustomizeAdvanced,
        size,
        locale,
        dark: false,
    }
    .state();
    state.advanced_extras_open = extras_open;
    state
}

#[test]
fn the_three_buttons_are_stops_and_keep_their_messages() {
    for size in [DESKTOP, PHONE] {
        for (locale, add, advanced, reset) in [
            (
                Locale::En,
                "+ Add another size",
                "Advanced settings",
                "Reset to defaults",
            ),
            (
                Locale::Ja,
                "+ ほかのサイズを追加",
                "上級設定",
                "既定値に戻す",
            ),
        ] {
            // The header both closed and open (its chevron differs, its message does not).
            for open in [false, true] {
                let s = state(size, locale, open);
                press_three_ways(&s, size, (Region::Page, add, 0), "PngSizeAddRequested");
                press_three_ways(
                    &s,
                    size,
                    (Region::Page, advanced, 0),
                    "AdvancedExtrasToggled",
                );
                press_three_ways(
                    &s,
                    size,
                    (Region::Page, reset, 0),
                    "ExportPlanResetRequested",
                );
            }
        }
    }
}
