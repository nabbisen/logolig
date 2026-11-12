//! The font set of the headless layout tests (RFC 035, job JH2).
//!
//! A layout test that answers differently on different machines is not a test, and the width of
//! a piece of text depends on the font that draws it. So the probe does not use the fonts of the
//! machine at all: its font system holds exactly
//!
//! - **Fira Sans** and the **Iced icon font**, which `iced_graphics` bundles (taken over from
//!   the font system it creates),
//! - the **Lucide** icon font, which the application loads at start-up too, and
//! - a **Japanese test font**: a subset of Noto Sans CJK JP, and the few symbols it lacks from
//!   Adwaita Sans (both SIL Open Font License 1.1), in `crates/app/test-assets/fonts/`, made by
//!   `scripts/make-test-font.sh`.
//!
//! The generic families (sans-serif, serif, monospace, cursive, fantasy) all resolve to Fira
//! Sans, and the locale is fixed, so neither the installed fonts nor the system locale can
//! change a result. The font system is process-global (`iced_graphics::text::font_system`); it
//! is replaced once, by [`ensure`], before any interface is built, and every interface of these
//! tests is built by `Probe::new`, which calls [`ensure`] first and blocks other threads until
//! it is done.

use std::sync::Once;

use iced_test::renderer::graphics::text::cosmic_text::{self, fontdb};
use iced_test::renderer::graphics::text::font_system;

/// The Japanese test font and the few symbols it lacks (subsets; see the README of the
/// directory for the sources, tools and checksums).
const JAPANESE_TEST_FONT: &[u8] = include_bytes!("../../test-assets/fonts/NotoSansJP-subset.otf");
const SYMBOLS_TEST_FONT: &[u8] =
    include_bytes!("../../test-assets/fonts/AdwaitaSans-symbols-subset.ttf");

/// The family the generic families resolve to.
const FAMILY: &str = "Fira Sans";

/// Replaces the font system by the hermetic one, once.
pub(super) fn ensure() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let mut system = font_system().write().expect("the font system");

        // Keep the faces `iced_graphics` bundled (loaded from memory) and drop every face it
        // found on the machine (loaded from a file).
        let mut db = fontdb::Database::new();
        for face in system.raw().db().faces() {
            if let fontdb::Source::Binary(data) = &face.source {
                db.load_font_source(fontdb::Source::Binary(data.clone()));
            }
        }
        db.set_serif_family(FAMILY);
        db.set_sans_serif_family(FAMILY);
        db.set_monospace_family(FAMILY);
        db.set_cursive_family(FAMILY);
        db.set_fantasy_family(FAMILY);
        // A fixed locale: the fallback order for a script must not follow the machine's.
        *system.raw() = cosmic_text::FontSystem::new_with_locale_and_db("en-US".to_string(), db);

        system.load_font(std::borrow::Cow::Borrowed(snora::lucide::LUCIDE_FONT_BYTES));
        system.load_font(std::borrow::Cow::Borrowed(JAPANESE_TEST_FONT));
        system.load_font(std::borrow::Cow::Borrowed(SYMBOLS_TEST_FONT));
    });
}

// ---- guard: the font may never silently lack a glyph ---------------------------------------------

/// The characters of `text` that no face of the font set has a glyph for.
fn uncovered(text: &str) -> Vec<char> {
    ensure();
    let mut system = font_system().write().expect("the font system");
    let ids: Vec<fontdb::ID> = system.raw().db().faces().map(|f| f.id).collect();
    let fonts: Vec<_> = ids
        .into_iter()
        .filter_map(|id| system.raw().get_font(id, fontdb::Weight::NORMAL))
        .collect();
    let mut missing: Vec<char> = text
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .filter(|c| !fonts.iter().any(|f| f.as_swash().charmap().map(*c) != 0))
        .collect();
    missing.sort_unstable();
    missing.dedup();
    missing
}

/// Every character of the Japanese locale file has a glyph in the font set. A character without
/// one would be measured as a placeholder box, and the test would answer for a layout nobody sees.
/// On failure, run `scripts/make-test-font.sh` and commit the new subset.
#[test]
fn the_font_set_covers_every_character_of_the_japanese_strings() {
    let missing = uncovered(include_str!("../../../i18n/locales/ja.toml"));
    assert!(
        missing.is_empty(),
        "no glyph for {missing:?} in the headless test fonts: run `scripts/make-test-font.sh` \
         (needs fonttools) and commit crates/app/test-assets/fonts/"
    );
}

/// The guard itself can fail: a character the subset does not hold is reported.
#[test]
fn the_guard_reports_a_character_the_font_set_lacks() {
    // U+1F9EA (a test tube) is in neither Fira Sans, Lucide, the icon font nor the subset.
    assert_eq!(uncovered("abc 日本語 \u{1F9EA}"), vec!['\u{1F9EA}']);
}
