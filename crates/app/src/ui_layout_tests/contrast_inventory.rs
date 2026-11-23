//! The contrast inventory (RFC 034, job J21): every (foreground, background) pair the application
//! derives from its palette, in Light and Dark, **before and after**, against the rule: **7:1 for
//! text, 3:1 for the borders and marks of controls**. `contrast.rs` checks the pictures; this file
//! checks the colours the helper functions and the themes give, including states a picture of a
//! resting screen does not show (hover, pressed, the drop-zone hover tint).
//!
//! "Before" is the colour the role had in 1.28.0 (iced's built-in `Theme::Light` / `Theme::Dark` and
//! the old helper formulas, kept here as data); "after" is the real function. The table is printed by
//! the test (`--nocapture`) and quoted in the review request.
//!
//! **Exempt, and why** (the test does not look at them): the card and container edges that only
//! frame other content (decorative: the controls inside carry their own borders); the preview's
//! mock-up colours (an illustration of a browser tab and a phone home screen, not interface text:
//! listed and measured in `contrast.rs` as drawn); a disabled control; snora's focus ring (its own
//! rule keeps it at 4.58:1 or more against what is behind it; measured in `ring.rs`).

use iced::widget::{button, text_input};
use iced::{Color, Theme};

use crate::ui::colors::{self, PrimaryState};
use crate::ui::contrast::{NON_TEXT_MIN, TEXT_MIN, contrast_ratio};
use crate::ui::theme;

/// One row of the inventory.
struct Row {
    /// The helper or style it comes from (`colors::*` names are checked against the source).
    source: &'static str,
    what: &'static str,
    before: Option<(Color, Color)>,
    after: (Color, Color),
    need: f32,
}

/// The colour snora draws a checked box and a selected radio in (RFC 021 amendment 2026-11-24, J21b):
/// the theme's `primary.strong`, which `ui/theme.rs` makes the primary fill itself.
fn selection_mark(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.color
}

/// The tick drawn on the selection mark.
fn mark_tick(theme: &Theme) -> Color {
    theme.extended_palette().primary.strong.text
}

fn alpha_over(fg: Color, alpha: f32, bg: Color) -> Color {
    Color::from_rgb(
        fg.r * alpha + bg.r * (1.0 - alpha),
        fg.g * alpha + bg.g * (1.0 - alpha),
        fg.b * alpha + bg.b * (1.0 - alpha),
    )
}

fn rows(dark: bool) -> Vec<Row> {
    let new = if dark { theme::dark() } else { theme::light() };
    let old = if dark { Theme::Dark } else { Theme::Light };
    let (np, op) = (new.extended_palette(), old.extended_palette());
    let (page, card, strong) = (
        colors::surface_page(&new),
        colors::surface_card(&new),
        colors::surface_strong(&new),
    );
    let (opage, ocard, ostrong) = (
        op.background.base.color,
        op.background.weak.color,
        op.background.strong.color,
    );
    let old_muted = |bg: Color| alpha_over(op.background.base.text, 0.6, bg);
    let hover_tint = colors::hover_tint(&new);
    let mut v: Vec<Row> = Vec::new();
    let mut text = |source, what, before, after| {
        v.push(Row {
            source,
            what,
            before,
            after,
            need: TEXT_MIN,
        })
    };

    // --- text on the page and on a card ---------------------------------------------------------
    text(
        "colors::body_text",
        "body text on the page",
        Some((op.background.base.text, opage)),
        (colors::body_text(&new), page),
    );
    text(
        "colors::body_text",
        "body text on a card",
        Some((op.background.base.text, ocard)),
        (colors::body_text(&new), card),
    );
    text(
        "colors::muted_text",
        "secondary text on the page",
        Some((old_muted(opage), opage)),
        (colors::muted_text(&new), page),
    );
    text(
        "colors::muted_text",
        "secondary text on a card",
        Some((old_muted(ocard), ocard)),
        (colors::muted_text(&new), card),
    );
    text(
        "colors::app_name",
        "app name on the page",
        Some((op.background.weak.text, opage)),
        (colors::app_name(&new), page),
    );
    text(
        "colors::section_label",
        "section label on a card",
        Some((op.background.weak.text, ocard)),
        (colors::section_label(&new), card),
    );
    text(
        "colors::file_name",
        "file name on the page",
        Some((op.background.base.text, opage)),
        (colors::file_name(&new), page),
    );
    text(
        "colors::page_title",
        "page title on the page",
        Some((op.background.base.text, opage)),
        (colors::page_title(&new), page),
    );
    text(
        "colors::group_heading",
        "group heading on the page",
        Some((op.background.base.text, opage)),
        (colors::group_heading(&new), page),
    );
    text(
        "colors::drop_zone_headline",
        "drop-zone headline on the card",
        Some((op.background.base.text, ocard)),
        (colors::drop_zone_headline(&new), card),
    );
    text(
        "colors::drop_zone_headline",
        "drop-zone headline on the hover tint",
        Some((op.background.base.text, op.primary.weak.color)),
        (colors::drop_zone_headline(&new), hover_tint),
    );
    text(
        "colors::muted_on",
        "drop-zone secondary text on the hover tint",
        Some((old_muted(op.primary.weak.color), op.primary.weak.color)),
        (colors::muted_on(&new, &[card, hover_tint]), hover_tint),
    );
    text(
        "colors::muted_on",
        "drop-zone secondary text on the card",
        Some((old_muted(ocard), ocard)),
        (colors::muted_on(&new, &[card, hover_tint]), card),
    );
    text(
        "colors::nav_label",
        "inactive navigation label on the sidebar",
        Some((old_muted(ocard), ocard)),
        (colors::nav_label(&new, false), card),
    );
    text(
        "colors::nav_label",
        "inactive navigation label on the hover fill",
        Some((old_muted(ostrong), ostrong)),
        (colors::nav_label(&new, false), strong),
    );
    text(
        "colors::nav_label",
        "active navigation label on its fill",
        Some((op.background.base.text, ostrong)),
        (colors::nav_label(&new, true), strong),
    );
    text(
        "colors::text_on",
        "text on the hover fill of an auxiliary button",
        Some((op.background.weak.text, ostrong)),
        (colors::text_on(&new, strong), strong),
    );
    text(
        "colors::placeholder",
        "placeholder of a text field",
        Some((
            text_input::default(&old, text_input::Status::Active).placeholder,
            opage,
        )),
        (colors::placeholder(&new), page),
    );

    // --- the primary button, every state that is read -------------------------------------------
    for (state, status, label) in [
        (PrimaryState::Rest, button::Status::Active, "rest"),
        (PrimaryState::Hovered, button::Status::Hovered, "hovered"),
        (PrimaryState::Pressed, button::Status::Pressed, "pressed"),
    ] {
        let o = button::primary(&old, status);
        let obg = match o.background {
            Some(iced::Background::Color(c)) => c,
            _ => op.primary.base.color,
        };
        let fill = colors::primary_fill(&new, state);
        text(
            "colors::on_primary",
            Box::leak(format!("text on the primary button, {label}").into_boxed_str()),
            Some((o.text_color, obg)),
            (colors::on_primary(&new, fill), fill),
        );
    }

    // --- every tier's text on its own fill (iced's tiers, lifted by the theme) -------------------
    for (name, o, n) in [
        ("background.base", op.background.base, np.background.base),
        ("background.weak", op.background.weak, np.background.weak),
        (
            "background.strong",
            op.background.strong,
            np.background.strong,
        ),
        ("primary.base", op.primary.base, np.primary.base),
        ("success.base", op.success.base, np.success.base),
        ("warning.base", op.warning.base, np.warning.base),
        ("danger.base", op.danger.base, np.danger.base),
    ] {
        text(
            "theme::generate (tier text)",
            Box::leak(
                format!("tier {name}: its text on its fill (toasts, default widgets)")
                    .into_boxed_str(),
            ),
            Some((o.text, o.color)),
            (n.text, n.color),
        );
    }

    // The tick on the selection mark (J21b). Before: black on white (Light) / white on black (Dark)... the
    // pole snora fell back to, with the opposite pole for the tick.
    let (old_mark, old_tick) = if op.is_dark {
        (Color::WHITE, Color::BLACK)
    } else {
        (Color::BLACK, Color::WHITE)
    };
    text(
        "theme::generate (primary.strong.text)",
        "the tick on the selection mark",
        Some((old_tick, old_mark)),
        (mark_tick(&new), selection_mark(&new)),
    );

    // --- non-text: borders and marks -----------------------------------------------------------
    let mut non_text = |source, what, before, after| {
        v.push(Row {
            source,
            what,
            before,
            after,
            need: NON_TEXT_MIN,
        })
    };
    non_text(
        "colors::control_border",
        "border of a button or text field, against the page",
        Some((ostrong, opage)),
        (colors::control_border(&new), page),
    );
    non_text(
        "colors::control_border",
        "border of a button or text field, against a card",
        Some((ostrong, ocard)),
        (colors::control_border(&new), card),
    );
    non_text(
        "colors::danger_border",
        "border of the reset button, against the page",
        Some((op.danger.weak.color, opage)),
        (colors::danger_border(&new), page),
    );
    non_text(
        "colors::danger_border",
        "border of the reset button, against a card",
        Some((op.danger.weak.color, ocard)),
        (colors::danger_border(&new), card),
    );
    let old_accent = if op.is_dark {
        op.primary.strong.color
    } else {
        op.primary.base.color
    };
    let (hover_row, press_row) = (card, strong);
    for (what, behind_old, behind_new) in [
        ("the selection mark, against the page", opage, page),
        (
            "the selection mark, against the hovered row",
            ocard,
            hover_row,
        ),
        (
            "the selection mark, against the pressed row",
            ostrong,
            press_row,
        ),
    ] {
        non_text(
            "theme::generate (primary.strong)",
            what,
            Some((old_mark, behind_old)),
            (selection_mark(&new), behind_new),
        );
    }
    non_text(
        "colors::nav_accent",
        "the active page's bar, against the page",
        Some((old_accent, opage)),
        (colors::nav_accent(&new), page),
    );
    non_text(
        "colors::nav_accent",
        "the active page's bar, against the sidebar",
        Some((old_accent, ocard)),
        (colors::nav_accent(&new), card),
    );
    for (state, label) in [
        (PrimaryState::Rest, "rest"),
        (PrimaryState::Hovered, "hovered"),
        (PrimaryState::Pressed, "pressed"),
    ] {
        non_text(
            "colors::primary_fill",
            Box::leak(
                format!("the primary button's fill, {label}, against the page").into_boxed_str(),
            ),
            Some((op.primary.base.color, opage)),
            (colors::primary_fill(&new, state), page),
        );
    }
    non_text(
        "colors::hover_tint",
        "the drop-zone accent border against the page",
        Some((op.primary.strong.color, opage)),
        (np.primary.base.color, page),
    );
    v
}

fn describe(r: &Row, theme: &str) -> String {
    let after = contrast_ratio(r.after.0, r.after.1);
    let before = r.before.map_or("   n/a".to_string(), |(a, b)| {
        format!("{:6.2}", contrast_ratio(a, b))
    });
    format!(
        "{theme:5} | {:<58} | {before} | {after:6.2} | {:>2.0}",
        r.what, r.need
    )
}

#[test]
fn every_pair_of_the_inventory_meets_the_rule_in_both_themes() {
    let mut failing = Vec::new();
    for dark in [false, true] {
        let name = if dark { "Dark" } else { "Light" };
        for r in rows(dark) {
            eprintln!("{}", describe(&r, name));
            if contrast_ratio(r.after.0, r.after.1) < r.need {
                failing.push(describe(&r, name));
            }
        }
    }
    assert!(
        failing.is_empty(),
        "pairs under the rule:\n{}",
        failing.join("\n")
    );
}

#[test]
fn the_inventory_shows_what_was_wrong_before() {
    // The old values of the pairs that failed are data of this table: if the old theme is ever
    // changed in iced, the table says so. At least the five known failures are in it.
    let mut under = 0;
    for dark in [false, true] {
        for r in rows(dark) {
            if let Some((a, b)) = r.before
                && contrast_ratio(a, b) < r.need
            {
                under += 1;
            }
        }
    }
    assert!(
        under >= 20,
        "the inventory finds only {under} pairs under the rule in 1.28.0"
    );
}

/// Every helper of `ui/colors.rs` that returns a `Color` is in the inventory (so a colour added
/// later cannot be forgotten silently), except the plain surface accessors, which are the
/// backgrounds of the pairs.
#[test]
fn every_color_function_of_colors_rs_is_in_the_inventory() {
    let source = include_str!("../ui/colors.rs");
    let in_inventory: String = rows(false)
        .iter()
        .map(|r| r.source)
        .collect::<Vec<_>>()
        .join(" ");
    // Backgrounds and building blocks, not pairs: named here so that adding another is a decision.
    const SURFACES: [&str; 4] = ["surface_page", "surface_card", "surface_strong", "muted_on"];
    let mut missing = Vec::new();
    for line in source.lines() {
        let Some(rest) = line.strip_prefix("pub fn ") else {
            continue;
        };
        if !line.contains("-> Color") {
            continue;
        }
        let name = rest.split('(').next().expect("a name");
        if SURFACES.contains(&name) {
            continue;
        }
        if !in_inventory.contains(&format!("colors::{name}")) {
            missing.push(name.to_string());
        }
    }
    assert!(
        missing.is_empty(),
        "colour functions missing from the inventory: {missing:?}"
    );
}

#[test]
fn the_themes_have_no_tier_that_carries_text_in_the_middle_of_the_range() {
    // A fill with relative luminance between about 0.10 and 0.30 cannot reach 7:1 with black or white.
    for t in [theme::light(), theme::dark()] {
        let p = t.extended_palette();
        let tiers = [
            ("background.base", p.background.base),
            ("background.weak", p.background.weak),
            ("background.strong", p.background.strong),
            ("primary.base", p.primary.base),
            ("success.base", p.success.base),
            ("warning.base", p.warning.base),
            ("danger.base", p.danger.base),
        ];
        for (name, pair) in tiers {
            assert!(
                contrast_ratio(pair.text, pair.color) >= TEXT_MIN,
                "{} {name}: {:.2}",
                if p.is_dark { "Dark" } else { "Light" },
                contrast_ratio(pair.text, pair.color)
            );
        }
    }
}
