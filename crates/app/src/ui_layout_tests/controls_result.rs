//! The Result page's controls (RFC 027 J14b step 3): Back (the fixed header), each card's Save
//! (card order), the preview toggle and Download all (the fixed footer). Each is a Tab stop and
//! Enter, Space and a click publish the one message the button published.

use iced::Size;
use logolig_i18n::Locale;

use super::controls::{Region, press_three_ways};
use super::support::{Case, Combo};

/// Tall windows, so that nothing is below the viewport and a click can reach every card.
const DESKTOP: Size = Size::new(1280.0, 3000.0);
const PHONE: Size = Size::new(360.0, 4000.0);

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
fn back_and_download_all_are_stops_and_keep_their_messages() {
    for size in [DESKTOP, PHONE] {
        for (locale, back, all) in [
            (Locale::En, "← Back", "Download all (ZIP)"),
            (Locale::Ja, "← 戻る", "すべてダウンロード (ZIP)"),
        ] {
            let s = state(Case::ResultDefault, size, locale);
            press_three_ways(&s, size, (Region::Page, back, 0), "EditCancelled");
            press_three_ways(&s, size, (Region::Page, all, 0), "DownloadAllRequested");
        }
    }
}

#[test]
fn each_cards_save_button_downloads_that_card() {
    for case in [Case::ResultDefault, Case::ResultAllOptions] {
        for size in [DESKTOP, PHONE] {
            for (locale, save) in [(Locale::En, "Save"), (Locale::Ja, "保存")] {
                let s = state(case, size, locale);
                // The cards of the fixture: 7 for the default plan, more with every option on.
                let cards = s.result_assets.as_ref().expect("a result").count();
                assert!(cards >= 7, "{case:?}: {cards} cards");
                for i in 0..cards {
                    press_three_ways(
                        &s,
                        size,
                        (Region::Page, save, i),
                        &format!("DownloadOneRequested({i})"),
                    );
                }
            }
        }
    }
}

#[test]
fn the_preview_toggle_opens_and_closes_the_preview() {
    for size in [DESKTOP, PHONE] {
        for (locale, label) in [(Locale::En, "Preview"), (Locale::Ja, "プレビューを見る")] {
            // Closed and open: the toggle is the first "Preview" text on the page.
            for case in [Case::ResultDefault, Case::ResultPreview] {
                let s = state(case, size, locale);
                press_three_ways(&s, size, (Region::Page, label, 0), "ResultPreviewToggled");
            }
        }
    }
}
