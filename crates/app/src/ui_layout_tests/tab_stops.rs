//! The Tab stops (RFC 027, job J14a): their order, and their completeness against the pointer.
//!
//! * **Order.** Read in Tab order, the stops go down the page and, within a row, left to right (a
//!   row is a set of stops whose tops are within 8 px). Tab order is the order of the tree, and the
//!   layout puts navigation first; this checks that no page lets Tab jump around.
//! * **Every stop is pointable.** Each stop lies in an interactive area of the RFC 035 hover scan:
//!   a control the keyboard reaches must be one the pointer can use, or the model has an orphan.
//! * **Every interactive area is a stop.** No exceptions: the controls that were not yet
//!   `snora::control`s were listed by name until RFC 027's J14b migrated the last (a stale entry
//!   failed the test); the list is gone and the test asserts completeness.

use iced::Size;

use super::support::{Finding, label_in, layout_matrix, probe_of};

#[test]
fn every_interactive_area_is_a_tab_stop_and_every_stop_is_an_interactive_area() {
    let mut missing: Vec<Finding> = Vec::new();
    let mut orphans: Vec<Finding> = Vec::new();
    for combo in layout_matrix() {
        let state = combo.state();
        // Tall, so that nothing is below the viewport (as the hit-area scan does).
        let mut p = probe_of(&state, Size::new(combo.size.width, 4000.0));
        let seen = p.walk();
        let areas = p.interactive_areas(&seen);
        let stops = p.focus_targets();
        for area in &areas {
            if stops.iter().any(|s| s.bounds.contains(area.center())) {
                continue;
            }
            missing.push(Finding {
                combo: combo.describe(),
                what: format!("{:?} is clickable and no Tab stop", label_in(&seen, *area)),
            });
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
        missing.is_empty(),
        "{} control(s) outside the Tab order:\n{}",
        missing.len(),
        join(&missing)
    );
    assert!(
        orphans.is_empty(),
        "{} orphan stop(s):\n{}",
        orphans.len(),
        join(&orphans)
    );
}

/// Tab order: the **regions** in the order of the tree (the side bar on a desktop window, then the
/// page, then the bottom bar on a phone), and within a region down the page and, within a row,
/// left to right; within the page, a fixed header, the scroll area and a fixed footer in that
/// order on the screen, and the stops inside one area by their place in its content.
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
            let ok = if ra != rb {
                ra < rb
            } else {
                pair_in_order(&pair[0], &pair[1])
            };
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

/// Does Tab from stop `a` to stop `b` of the same region go the right way? Two stops in the same
/// scroll area: by their place in its content. Two in different zones (a fixed header, an area, a
/// fixed footer): by where the zones are on the screen (an area's content overlaps whatever lies
/// below it while it is unscrolled).
fn pair_in_order(a: &super::support::FocusTarget, b: &super::support::FocusTarget) -> bool {
    if a.area == b.area {
        in_order(a.bounds, b.bounds)
    } else {
        in_order(a.area.unwrap_or(a.bounds), b.area.unwrap_or(b.bounds))
    }
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

/// A fixed header, a scroll area longer than its viewport and a fixed footer (the Result page):
/// the stops are in order on the screen although the area's unscrolled content overlaps the
/// footer; and the rule reports the footer coming before the header.
#[test]
fn a_fixed_header_an_area_and_a_fixed_footer_are_in_order() {
    use iced::widget::{column, scrollable, text};
    let rows =
        column((0..30).map(|i| snora::control(text(format!("row {i}"))).on_press(()).into()))
            .spacing(8);
    let page: iced::Element<'_, ()> = column![
        snora::control(text("header")).on_press(()),
        scrollable(rows).height(200),
        snora::control(text("footer")).on_press(()),
    ]
    .spacing(10)
    .into();
    let mut p = super::support::Probe::new(page, Size::new(300.0, 400.0));
    let stops = p.focus_targets();
    assert_eq!(stops.len(), 32);
    assert!(stops[0].area.is_none() && stops[31].area.is_none() && stops[1].area.is_some());
    // In unscrolled content space the last row lies below the footer: the naive rule fails.
    assert!(
        stops[30].bounds.y > stops[31].bounds.y,
        "the overlap this test is about"
    );
    for pair in stops.windows(2) {
        assert!(pair_in_order(&pair[0], &pair[1]), "{pair:?}");
    }
    assert!(
        !pair_in_order(&stops[31], &stops[0]),
        "footer then header is a jump"
    );
}
