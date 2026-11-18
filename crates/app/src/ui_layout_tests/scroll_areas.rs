//! A page scrolls as one (RFC 027 J14a; RFC 026): no scroll area inside another.
//!
//! Two nested scroll areas clip what reaches past the inner one's edge (the focus ring of a
//! checkbox was cut on its left, found by `ring.rs`), make the keyboard's "keep the focused
//! control in view" ambiguous (which area scrolls?), and give the pointer two wheels to fight over.
//! The property is read from the layout: no scrollable's **visible** part lies inside another's (an
//! inner area inside an outer one is as tall as its content, so its bounds are not the measure).

use iced::Rectangle;

use super::support::{CASES, Combo, Kind, layout_matrix, probe_of};

fn scroll_areas(combo: &Combo) -> Vec<Rectangle> {
    let state = combo.state();
    probe_of(&state, combo.size)
        .walk()
        .iter()
        .filter(|s| s.kind == Kind::Scrollable)
        .map(|s| s.visible.unwrap_or(s.bounds))
        .collect()
}

fn contains(outer: &Rectangle, inner: &Rectangle) -> bool {
    inner.x >= outer.x - 0.5
        && inner.y >= outer.y - 0.5
        && inner.x + inner.width <= outer.x + outer.width + 0.5
        && inner.y + inner.height <= outer.y + outer.height + 0.5
}

#[test]
fn no_scroll_area_is_inside_another_on_any_page() {
    let mut problems = Vec::new();
    for combo in layout_matrix() {
        let areas = scroll_areas(&combo);
        for (i, a) in areas.iter().enumerate() {
            for (j, b) in areas.iter().enumerate() {
                if i != j && contains(a, b) {
                    problems.push(format!(
                        "[{}] a scroll area lies inside another",
                        combo.describe()
                    ));
                }
            }
        }
    }
    problems.dedup();
    assert!(
        problems.is_empty(),
        "{} finding(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

/// Each page that scrolls has exactly one area.
#[test]
fn customize_scrolls_as_one_area() {
    let mut counted = 0;
    for combo in layout_matrix()
        .into_iter()
        .filter(|c| c.case == super::support::Case::CustomizeAdvanced)
    {
        let n = scroll_areas(&combo).len();
        assert_eq!(n, 1, "[{}] {n} scroll areas", combo.describe());
        counted += 1;
    }
    assert!(
        counted >= 14,
        "the whole Customize matrix was checked ({counted})"
    );
    assert!(CASES.contains(&super::support::Case::CustomizeAdvanced));
}

/// A self-test: the check can fail. A scroll area inside another is found.
#[test]
fn a_nested_scroll_area_is_reported() {
    use iced::widget::{column, scrollable, text};
    let nested: iced::Element<'_, ()> = scrollable(
        column![text("above"), scrollable(column![text("inner")]).height(80)].spacing(10),
    )
    .height(300)
    .into();
    let mut probe = super::support::Probe::new(nested, iced::Size::new(300.0, 400.0));
    let areas: Vec<Rectangle> = probe
        .walk()
        .iter()
        .filter(|s| s.kind == Kind::Scrollable)
        .map(|s| s.visible.unwrap_or(s.bounds))
        .collect();
    assert_eq!(areas.len(), 2);
    assert!(
        contains(&areas[0], &areas[1]) || contains(&areas[1], &areas[0]),
        "the nesting is seen: {areas:?}"
    );
}
