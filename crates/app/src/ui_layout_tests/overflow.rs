//! P1: nothing lies beyond the right edge of the window (RFC 035).
//!
//! Every piece of text and every text input must end at or before the window's width, and must
//! not be squeezed down to nothing. `iced` lays a row out inside its limits (a child that does
//! not fit gets what is left, down to zero width), so the overflow of a crowded row shows as a
//! text squeezed out at or beyond the right edge; both are reported.

use iced::Size;

use super::support::{Combo, Finding, Kind, Seen, matrix, probe_of};

/// How far past the edge a piece may reach before it counts (rounding of the layout).
const TOLERANCE: f32 = 0.5;

/// The texts and text inputs of `seen` that end beyond `width`.
pub(super) fn overflowing(seen: &[Seen], width: f32) -> Vec<&Seen> {
    seen.iter()
        .filter(|s| matches!(s.kind, Kind::Text | Kind::TextInput))
        .filter(|s| s.bounds.x + s.bounds.width > width + TOLERANCE)
        .collect()
}

/// The pieces of text of `seen` that hold text but have been squeezed to less than a pixel wide.
pub(super) fn squeezed_out(seen: &[Seen]) -> Vec<&Seen> {
    seen.iter()
        .filter(|s| matches!(s.kind, Kind::Text | Kind::TextInput))
        .filter(|s| !s.text.trim().is_empty() && s.bounds.width < 1.0)
        .collect()
}

fn findings(combo: &Combo, pieces: Vec<&Seen>, what: &str) -> Vec<Finding> {
    pieces
        .into_iter()
        .map(|s| Finding {
            combo: combo.describe(),
            what: format!(
                "{:?} {what} (x {:.1}, width {:.1}, window {:.0} px wide)",
                s.text, s.bounds.x, s.bounds.width, combo.size.width
            ),
        })
        .collect()
}

/// P1 over the whole matrix: nothing holds today, so there is no exception.
#[test]
fn nothing_lies_beyond_the_right_edge_of_the_window() {
    let all: Vec<Finding> = matrix()
        .iter()
        .flat_map(|combo| {
            let state = combo.state();
            let seen = probe_of(&state, combo.size).walk();
            findings(
                combo,
                overflowing(&seen, combo.size.width),
                "ends beyond the right edge",
            )
        })
        .collect();
    assert!(all.is_empty(), "{} finding(s):\n{}", all.len(), join(&all));
}

/// P1b over the whole matrix, minus the exceptions named in [`KNOWN`]. `iced` lays a row out
/// inside its limits (a child that does not fit gets what is left, down to nothing), so a
/// crowded or too-short page shows as text squeezed out rather than as text past the edge.
#[test]
fn no_text_is_squeezed_out() {
    let all: Vec<Finding> = matrix()
        .iter()
        .flat_map(|combo| {
            let state = combo.state();
            let seen = probe_of(&state, combo.size).walk();
            findings(combo, squeezed_out(&seen), "is squeezed to nothing")
        })
        .collect();
    let unexpected: Vec<_> = all.iter().filter(|f| !is_known(f)).cloned().collect();
    assert!(
        unexpected.is_empty(),
        "{} finding(s) of {} combinations:\n{}",
        unexpected.len(),
        matrix().len(),
        join(&unexpected)
    );
    // An exception that no longer matches a finding is a fixed defect: delete the entry.
    let stale: Vec<_> = KNOWN
        .iter()
        .filter(|e| !all.iter().any(|f| entry_matches(e, f)))
        .collect();
    assert!(
        stale.is_empty(),
        "{} exception(s) no longer needed, delete them: {stale:?}",
        stale.len()
    );
}

fn join(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Texts squeezed out today. Cause: the Settings page is not scrollable, so at a height that
/// does not hold its content the last rows of the shortcut list (and, at narrow widths, the
/// whole description column) are laid out with no height or width left. Belongs to RFC 034
/// (jobs J9 and J10): each entry is one combination (screen, size, language; the theme does not
/// matter) and the text; the jobs delete entries as they fix them, and the list must end empty.
const KNOWN: &[(&str, &str)] = &[
    (
        "Settings 1024x600 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 1024x600 en", "Show this list"),
    (
        "Settings 360x640 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 360x640 en", "Show this list"),
    ("Settings 640x480 en", "Choose a file"),
    (
        "Settings 640x480 en",
        "Close the message, or go back to the start",
    ),
    ("Settings 640x480 en", "Download all (ZIP)"),
    ("Settings 640x480 en", "Go to Home, Customize or Settings"),
    ("Settings 640x480 en", "Keyboard shortcuts"),
    (
        "Settings 640x480 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 640x480 en", "Show this list"),
    ("Settings 767x480 en", "Choose a file"),
    (
        "Settings 767x480 en",
        "Close the message, or go back to the start",
    ),
    ("Settings 767x480 en", "Download all (ZIP)"),
    ("Settings 767x480 en", "Go to Home, Customize or Settings"),
    ("Settings 767x480 en", "Keyboard shortcuts"),
    (
        "Settings 767x480 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 767x480 en", "Show this list"),
    ("Settings 768x480 en", "Choose a file"),
    (
        "Settings 768x480 en",
        "Close the message, or go back to the start",
    ),
    ("Settings 768x480 en", "Download all (ZIP)"),
    ("Settings 768x480 en", "Go to Home, Customize or Settings"),
    (
        "Settings 768x480 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 768x480 en", "Show this list"),
    (
        "Settings 800x600 en",
        "Moving between buttons and fields with the Tab key is not available yet.",
    ),
    ("Settings 800x600 en", "Show this list"),
];

fn entry_matches(entry: &(&str, &str), f: &Finding) -> bool {
    f.combo.starts_with(entry.0) && f.what.starts_with(&format!("{:?} ", entry.1))
}

fn is_known(f: &Finding) -> bool {
    KNOWN.iter().any(|entry| entry_matches(entry, f))
}

// ---- self-test: the property can fail ---------------------------------------------------------

/// A row wider than the window squeezes its last text out at the edge: that text, and only
/// that one, must be reported (so a pass is not vacuous). A row that fits reports nothing.
#[test]
fn p1b_reports_a_text_squeezed_out_of_a_crowded_row() {
    use iced::widget::{row, text};
    let wide: iced::Element<'_, ()> = row![
        text("first").width(400),
        text("second").width(400),
        text("third").width(400)
    ]
    .into();
    let mut probe = super::support::Probe::new(wide, Size::new(800.0, 600.0));
    let seen = probe.walk();
    let found = squeezed_out(&seen);
    assert_eq!(
        found.len(),
        1,
        "exactly the one squeezed text is reported: {found:?}"
    );
    assert_eq!(found[0].text, "third");

    let fits: iced::Element<'_, ()> =
        row![text("first").width(400), text("second").width(400)].into();
    let mut probe = super::support::Probe::new(fits, Size::new(800.0, 600.0));
    assert!(
        squeezed_out(&probe.walk()).is_empty(),
        "a row that fits reports nothing"
    );
}

/// The predicate on hand-made pieces: one ends beyond the edge, one ends exactly at it.
#[test]
fn p1_reports_a_piece_that_ends_beyond_the_edge() {
    let piece = |x: f32, w: f32| Seen {
        kind: Kind::Text,
        text: "text".to_string(),
        bounds: iced::Rectangle::new(iced::Point::new(x, 0.0), Size::new(w, 20.0)),
        visible: None,
    };
    let seen = [piece(700.0, 200.0), piece(700.0, 100.0)];
    let found = overflowing(&seen, 800.0);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!((found[0].bounds.width - 200.0).abs() < 0.1);
}
