//! The active page in the navigation (RFC 021 Q1, D7), and no text glyph as a state cue.
//!
//! The cue has three parts so that the state is never colour alone: the item is filled, its label
//! is bold, and a thin accent bar sits on the edge facing the content (the right edge of the
//! desktop sidebar, the top edge of the mobile bottom bar). The bar is measured in pixels, on both
//! themes and both layouts; the bold label by the width of its text; and a source test keeps the
//! old glyph cues (`✓` for a selection, `⏳`, `▣`, `⚠`) out of the production code.

use iced::{Size, Theme};

use super::support::{Case, Combo, Probe, probe_of};
use crate::app::{AppState, Message, NavPage};
use crate::ui::colors;
use logolig_i18n::Locale;

const SCALE: f32 = 2.0;
const BAR: f32 = 3.0;
const SIDEBAR_WIDTH: f32 = 90.0;
const BOTTOM_BAR_HEIGHT: f32 = 64.0;

fn state_on(page: NavPage, size: Size, dark: bool) -> AppState {
    let mut state = Combo {
        case: Case::Home,
        size,
        locale: Locale::En,
        dark,
    }
    .state();
    state.nav_page = page;
    state
}

fn theme(dark: bool) -> Theme {
    if dark { Theme::Dark } else { Theme::Light }
}

fn pixel(rgba: &[u8], size: Size, x: f32, y: f32) -> [f32; 3] {
    let width_px = (size.width * SCALE) as usize;
    let i = ((y * SCALE) as usize * width_px + (x * SCALE) as usize) * 4;
    [rgba[i] as f32, rgba[i + 1] as f32, rgba[i + 2] as f32]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn label_centre(p: &mut Probe<'_, Message>, label: &str) -> iced::Point {
    p.walk()
        .iter()
        .find(|s| s.text == label)
        .unwrap_or_else(|| panic!("no nav label {label:?}"))
        .bounds
        .center()
}

/// The accent bar is on the active item only, on the edge facing the content, in the accent of
/// the theme; the other two items show no bar.
#[test]
fn the_active_page_has_an_accent_bar_on_the_edge_facing_the_content() {
    let pages = [
        (NavPage::Home, "Home"),
        (NavPage::Customize, "Customize"),
        (NavPage::Settings, "Settings"),
    ];
    // (window, is the bottom bar)
    for (size, bottom) in [
        (Size::new(1280.0, 720.0), false),
        (Size::new(360.0, 640.0), true),
    ] {
        for dark in [false, true] {
            let th = theme(dark);
            let accent = colors::nav_accent(&th);
            let accent = [accent.r * 255.0, accent.g * 255.0, accent.b * 255.0];
            for (active, _) in pages {
                let state = state_on(active, size, dark);
                let mut p = probe_of(&state, size);
                let rgba = p.rgba(&th, SCALE);
                for (page, label) in pages {
                    let at = label_centre(&mut p, label);
                    let on_bar = if bottom {
                        pixel(
                            &rgba,
                            size,
                            at.x,
                            size.height - BOTTOM_BAR_HEIGHT + BAR / 2.0,
                        )
                    } else {
                        pixel(&rgba, size, SIDEBAR_WIDTH - BAR / 2.0, at.y)
                    };
                    let near = distance(on_bar, accent) < 6.0;
                    let layout = if bottom { "bottom bar" } else { "sidebar" };
                    let theme_name = if dark { "Dark" } else { "Light" };
                    assert_eq!(
                        near,
                        page == active,
                        "{layout}, {theme_name}, active {active:?}: the bar of {page:?} is {on_bar:?} (accent {accent:?})"
                    );
                }
            }
        }
    }
}

/// The bar is the edge of the item: it is as thick as it should be, and the page beside it is
/// not tinted (no accent leaks onto the content).
#[test]
fn the_bar_is_thin_and_flush_with_the_content() {
    let size = Size::new(1280.0, 720.0);
    let th = theme(false);
    let accent = colors::nav_accent(&th);
    let accent = [accent.r * 255.0, accent.g * 255.0, accent.b * 255.0];
    let state = state_on(NavPage::Customize, size, false);
    let mut p = probe_of(&state, size);
    let rgba = p.rgba(&th, SCALE);
    let y = label_centre(&mut p, "Customize").y;
    let columns: Vec<f32> = (0..(size.width as usize))
        .map(|x| x as f32 + 0.25)
        .filter(|x| distance(pixel(&rgba, size, *x, y), accent) < 6.0)
        .collect();
    assert!(
        columns
            .first()
            .is_some_and(|x| (*x - (SIDEBAR_WIDTH - BAR)).abs() < 1.0)
            && columns
                .last()
                .is_some_and(|x| (*x - (SIDEBAR_WIDTH - 0.25)).abs() < 1.0),
        "the accent occupies x {:?}..{:?}, expected the last {BAR} px of the sidebar",
        columns.first(),
        columns.last()
    );
}

/// The label of the active item is bold, the others are not. (The hermetic test fonts have no bold
/// face, so the width of the text cannot show it here: the font the label asks for is checked, and
/// the real window's capture shows the result.)
#[test]
fn the_active_label_asks_for_a_bold_face() {
    use iced::font::Weight;
    assert_eq!(crate::ui::sidebar::label_font(true).weight, Weight::Bold);
    assert_eq!(crate::ui::sidebar::label_font(false).weight, Weight::Normal);
}

/// No state cue is a text glyph: a selection (`✓`, `▣`), busy (`⏳`) or a warning (`⚠`) is drawn
/// from the icon font or by a widget, never from a character the machine's fonts may not have.
#[test]
fn no_state_cue_is_a_text_glyph() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    let mut stack = vec![root];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("read the source folder") {
            let path = entry.expect("an entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                // The tests quote the glyphs on purpose.
                if name != "ui_layout_tests" {
                    stack.push(path);
                }
            } else if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
                let text = std::fs::read_to_string(&path).expect("read a source file");
                for (number, line) in text.lines().enumerate() {
                    let code = line.trim_start();
                    if code.starts_with("//") {
                        continue; // a comment may name the glyphs it replaced
                    }
                    for glyph in ["✓", "▣", "⏳", "⚠"] {
                        if line.contains(glyph) {
                            offenders.push(format!("{}:{}: {glyph}", path.display(), number + 1));
                        }
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "a text glyph is a state cue again:\n{}",
        offenders.join("\n")
    );
}
