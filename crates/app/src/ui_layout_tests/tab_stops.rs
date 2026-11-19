//! The Tab stops (RFC 027, job J14a): their order, and their completeness against the pointer.
//!
//! * **Order.** Read in Tab order, the stops go down the page and, within a row, left to right (a
//!   row is a set of stops whose tops are within 8 px). Tab order is the order of the tree, and the
//!   layout puts navigation first; this checks that no page lets Tab jump around.
//! * **Every stop is pointable.** Each stop lies in an interactive area of the RFC 035 hover scan:
//!   a control the keyboard reaches must be one the pointer can use, or the model has an orphan.
//! * **Every interactive area is a stop, except a named list.** The controls not yet migrated to
//!   `snora::control` (RFC 027 J14b: one reviewed step per page) are listed below by page and
//!   label; the job that migrates a control deletes its entry, a stale entry fails the test, and
//!   the list must end empty.

use std::collections::BTreeSet;

use iced::Size;

use super::support::{Finding, label_in, layout_matrix, probe_of};

/// Interactive areas that are not Tab stops yet: (page, label). A page of `"*"` is every page.
/// The icon-only controls are named by their icon glyph (the lucide private-use code point), as the
/// scan prints them: `e0b2` Download (the "Save" buttons of the cards and "Download all (ZIP)"). The disclosure
/// triangles `▶ ▼` are text glyphs until J14b makes them chevrons. (The navigation items left
/// the list in J14b step 1.)
const NOT_MIGRATED: &[(&str, &str)] = &[
    // Result: Back, the Save buttons and Download all, the preview toggle.
    ("ResultDefault", "\u{e0b2}"),
    ("ResultDefault", "← Back"),
    ("ResultDefault", "← 戻る"),
    ("ResultDefault", "▶ Preview"),
    ("ResultDefault", "▶ プレビューを見る"),
    ("ResultAllOptions", "\u{e0b2}"),
    ("ResultAllOptions", "← Back"),
    ("ResultAllOptions", "← 戻る"),
    ("ResultAllOptions", "▶ Preview"),
    ("ResultAllOptions", "▶ プレビューを見る"),
    ("ResultPreview", "\u{e0b2}"),
    ("ResultPreview", "← Back"),
    ("ResultPreview", "← 戻る"),
    ("ResultPreview", "▼ Preview"),
    ("ResultPreview", "▼ プレビューを見る"),
    ("ResultPreviewChecker", "\u{e0b2}"),
    ("ResultPreviewChecker", "← Back"),
    ("ResultPreviewChecker", "← 戻る"),
    ("ResultPreviewChecker", "▼ Preview"),
    ("ResultPreviewChecker", "▼ プレビューを見る"),
    // Customize: "Add custom size", the "Advanced settings" header, "Reset to defaults".
    ("CustomizeAdvanced", "+ Add custom size"),
    ("CustomizeAdvanced", "+ カスタムサイズ追加"),
    ("CustomizeAdvanced", "▼"),
    ("CustomizeAdvanced", "↻"),
];

fn is_listed(page: &str, label: &str) -> bool {
    NOT_MIGRATED
        .iter()
        .any(|(p, l)| (*p == "*" || *p == page) && *l == label)
}

#[test]
fn every_interactive_area_is_a_tab_stop_and_every_stop_is_an_interactive_area() {
    let mut unlisted: Vec<Finding> = Vec::new();
    let mut orphans: Vec<Finding> = Vec::new();
    let mut seen_entries: BTreeSet<(String, String)> = BTreeSet::new();
    for combo in layout_matrix() {
        let state = combo.state();
        // Tall, so that nothing is below the viewport (as the hit-area scan does).
        let mut p = probe_of(&state, Size::new(combo.size.width, 4000.0));
        let seen = p.walk();
        let areas = p.interactive_areas(&seen);
        let stops = p.focus_targets();
        let page = format!("{:?}", combo.case);
        for area in &areas {
            if stops.iter().any(|s| s.bounds.contains(area.center())) {
                continue;
            }
            let label = label_in(&seen, *area);
            if is_listed(&page, &label) {
                seen_entries.insert((page.clone(), label));
            } else {
                unlisted.push(Finding {
                    combo: combo.describe(),
                    what: format!("{label:?} is clickable and no Tab stop"),
                });
            }
        }
        for stop in &stops {
            if !areas.iter().any(|a| a.contains(stop.bounds.center())) {
                orphans.push(Finding {
                    combo: combo.describe(),
                    what: format!("a Tab stop at {:?} is no interactive area", stop.bounds),
                });
            }
        }
    }
    assert!(
        unlisted.is_empty(),
        "{} control(s) outside the Tab order:\n{}",
        unlisted.len(),
        join(&unlisted)
    );
    assert!(
        orphans.is_empty(),
        "{} orphan stop(s):\n{}",
        orphans.len(),
        join(&orphans)
    );
    // A listed control that is a stop now, or no longer exists, is a stale entry: delete it.
    let stale: Vec<_> = NOT_MIGRATED
        .iter()
        .filter(|(page, label)| {
            !seen_entries
                .iter()
                .any(|(p, l)| (*page == "*" || page == p) && label == l)
        })
        .collect();
    assert!(
        stale.is_empty(),
        "{} entry(ies) no longer needed, delete them: {stale:?}",
        stale.len()
    );
}

/// Tab order: the **regions** in the order of the tree (the side bar on a desktop window, then the
/// page, then the bottom bar on a phone), and within a region down the page and, within a row,
/// left to right.
#[test]
fn tab_order_goes_down_the_page_and_left_to_right_within_a_row() {
    let mut problems: Vec<Finding> = Vec::new();
    let mut compared = 0;
    for combo in layout_matrix() {
        let state = combo.state();
        let mut p = probe_of(&state, Size::new(combo.size.width, 4000.0));
        let stops = p.focus_targets();
        // 0 side bar, 1 page, 2 bottom bar (by the order of the tree: see `is_navigation_stop`).
        let total = stops.len();
        let region = |p: &super::support::Probe<'_, crate::app::Message>, i: usize| {
            if !p.is_navigation_stop(i, total) {
                1
            } else if p.size.width >= 768.0 {
                0
            } else {
                2
            }
        };
        for (i, pair) in stops.windows(2).enumerate() {
            compared += 1;
            let (ra, rb) = (region(&p, i), region(&p, i + 1));
            let (a, b) = (pair[0].bounds, pair[1].bounds);
            let ok = if ra == rb { in_order(a, b) } else { ra < rb };
            if !ok {
                problems.push(Finding {
                    combo: combo.describe(),
                    what: format!(
                        "Tab goes from ({:.0}, {:.0}) in region {ra} to ({:.0}, {:.0}) in region {rb}",
                        a.x, a.y, b.x, b.y
                    ),
                });
            }
        }
        let _ = &mut p;
    }
    assert!(
        compared > 100,
        "only {compared} neighbouring stops were compared"
    );
    assert!(
        problems.is_empty(),
        "{} jump(s):\n{}",
        problems.len(),
        join(&problems)
    );
}

/// Does Tab from `a` to `b` go down the page, or right within a row (tops within 8 px)?
fn in_order(a: iced::Rectangle, b: iced::Rectangle) -> bool {
    if (a.y - b.y).abs() < 8.0 {
        b.x >= a.x - 0.5
    } else {
        b.y > a.y
    }
}

fn join(findings: &[Finding]) -> String {
    findings
        .iter()
        .take(15)
        .map(|f| f.to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The checks can fail: a control the pointer can use and the keyboard cannot is reported, and a
/// layout whose Tab order jumps back up is reported.
#[test]
fn the_checks_can_fail() {
    use iced::widget::{button, column, text};
    // A plain iced button is clickable and no stop; a snora control is both.
    let mixed: iced::Element<'_, ()> = column![
        button(text("plain")).on_press(()),
        snora::control(text("snora")).on_press(()),
    ]
    .spacing(20)
    .into();
    let mut p = super::support::Probe::new(mixed, Size::new(300.0, 300.0));
    let seen = p.walk();
    let areas = p.interactive_areas(&seen);
    let stops = p.focus_targets();
    assert_eq!(areas.len(), 2);
    assert_eq!(stops.len(), 1, "only the snora control is a stop");
    let outside: Vec<_> = areas
        .iter()
        .filter(|a| !stops.iter().any(|s| s.bounds.contains(a.center())))
        .collect();
    assert_eq!(
        outside.len(),
        1,
        "the plain button is clickable and no stop"
    );

    // The order rule reports a jump back up and a jump back left within a row, and accepts the
    // forward moves.
    let at = |x: f32, y: f32| iced::Rectangle::new(iced::Point::new(x, y), Size::new(50.0, 20.0));
    assert!(in_order(at(0.0, 0.0), at(0.0, 40.0)), "down");
    assert!(in_order(at(0.0, 0.0), at(80.0, 3.0)), "right in a row");
    assert!(
        !in_order(at(0.0, 100.0), at(0.0, 40.0)),
        "back up is a jump"
    );
    assert!(
        !in_order(at(80.0, 0.0), at(0.0, 2.0)),
        "back left in a row is a jump"
    );
}
