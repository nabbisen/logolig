//! The explanation of an invalid web manifest value is in view (release 1.29.0, job R2b).
//!
//! The real-window pass of the release candidate found it: Tab to the Theme color field puts it at
//! the very bottom of the scroll area (`keep_in_view` scrolls the least amount that shows the
//! control), the line under it for its error is outside the visible area, and a user who types
//! `not-a-colour` sees nothing happen. These tests do what the user does, in the J14 `Session`
//! (the real view, widget state kept, the real task operations): Tab to each of the four fields,
//! make the value invalid, and ask whether the field **and its error line** are fully inside every
//! scroll area that holds them; and the other way round (Tab to a field that already shows its
//! error, from either side).

use iced::Size;

use super::focus_after::Session;
use super::support::{Case, Kind, Seen};
use crate::app::{ManifestField, Message};

/// The window of the real runs, and a small one.
const SIZES: [Size; 2] = [Size::new(1042.0, 1144.0), Size::new(853.0, 480.0)];
const COLOR_SENTENCE: &str = "Use a color like #1A2B3C or #1AB";
const NAME_SENTENCE: &str = "Enter a name";

/// The four fields: the order of the page, the sentence each shows, a message that makes it invalid.
fn fields() -> [(ManifestField, &'static str, Message); 4] {
    [
        (
            ManifestField::Name,
            NAME_SENTENCE,
            Message::WebManifestNameChanged(String::new()),
        ),
        (
            ManifestField::ShortName,
            NAME_SENTENCE,
            Message::WebManifestShortNameChanged(String::new()),
        ),
        (
            ManifestField::ThemeColor,
            COLOR_SENTENCE,
            Message::WebManifestThemeColorChanged("not-a-colour".into()),
        ),
        (
            ManifestField::BackgroundColor,
            COLOR_SENTENCE,
            Message::WebManifestBackgroundColorChanged("not-a-colour".into()),
        ),
    ]
}

/// The Tab-order index of the stop of the `k`-th text input of the page (0 is the custom size).
fn stop_of_text_input(s: &mut Session, k: usize) -> usize {
    s.with(|p| {
        p.focus_targets()
            .iter()
            .enumerate()
            .filter(|(_, t)| t.text_input)
            .map(|(i, _)| i)
            .nth(k)
            .expect("the text input exists")
    })
}

/// Tabs until the stop `target` holds the focus (a bounded walk).
fn tab_to(s: &mut Session, target: usize) {
    for _ in 0..80 {
        if s.focus().0 == Some(target) {
            return;
        }
        s.tab();
    }
    panic!("Tab never reached stop {target}");
}

/// How far the text `sentence` lies outside the visible area, in pixels (0 = fully visible);
/// `None` when the page has no such text. `nth` picks among equal sentences (both name fields show
/// the same one, and so do both colour fields).
fn pixels_outside(s: &mut Session, sentence: &str, nth: usize) -> Option<f32> {
    s.with(|p| {
        let all: Vec<Seen> = p.walk();
        let line = all
            .iter()
            .filter(|w| w.kind == Kind::Text && w.text == sentence)
            .nth(nth)?;
        Some(match line.visible {
            None => line.bounds.height,
            Some(v) => {
                (line.bounds.height - v.height).max(0.0) + (line.bounds.width - v.width).max(0.0)
            }
        })
    })
}

fn state_line(s: &mut Session, sentence: &str, nth: usize) -> String {
    let outside = pixels_outside(s, sentence, nth);
    let control = s.with(|p| p.focused_view()).map(|v| v.fully_visible());
    format!("line outside by {outside:?} px, control fully visible: {control:?}")
}

/// Tab to the field, make its value invalid by typing: the field and its line are both in view.
#[test]
fn the_error_line_is_in_view_after_typing_an_invalid_value() {
    let mut problems = Vec::new();
    for size in SIZES {
        for (k, (field, sentence, invalid)) in fields().into_iter().enumerate() {
            let mut s = Session::new(Case::CustomizeAdvanced, size);
            let target = stop_of_text_input(&mut s, k + 1);
            tab_to(&mut s, target);
            s.apply(invalid);
            assert!(
                s.state.manifest_inputs.shows_error(field),
                "{field:?} shows its error"
            );
            let outside = pixels_outside(&mut s, sentence, 0).expect("the sentence is on the page");
            let control = s
                .with(|p| p.focused_view())
                .expect("a focused control")
                .fully_visible();
            if outside > 0.5 || !control {
                problems.push(format!(
                    "{}x{} {field:?}: {}",
                    size.width,
                    size.height,
                    state_line(&mut s, sentence, 0)
                ));
            }
            assert_eq!(s.focus().0, Some(target), "{field:?} keeps the focus");
        }
    }
    assert!(
        problems.is_empty(),
        "the error is not in view:\n{}",
        problems.join("\n")
    );
}

/// Tab (and Shift+Tab) to a field that already shows its error: the line is in view too.
#[test]
fn the_error_line_is_in_view_when_the_focus_arrives_on_an_invalid_field() {
    let mut problems = Vec::new();
    for size in SIZES {
        for k in 1..=4 {
            let (field, sentence, _) = fields()[k - 1].clone();
            // Both name fields show the same sentence, and so do both colour fields: the second of
            // each pair is the line of the second field.
            let nth = (k - 1) % 2;
            for backwards in [false, true] {
                let mut s = Session::new(Case::CustomizeManifestErrors, size);
                let target = stop_of_text_input(&mut s, k);
                if backwards {
                    // Past the field, then back with Shift+Tab.
                    tab_to(&mut s, target + 1);
                    s.press_shift_tab();
                } else {
                    tab_to(&mut s, target);
                }
                assert_eq!(s.focus().0, Some(target), "{field:?} has the focus");
                let outside =
                    pixels_outside(&mut s, sentence, nth).expect("the sentence is on the page");
                let control = s
                    .with(|p| p.focused_view())
                    .expect("a focused control")
                    .fully_visible();
                if outside > 0.5 || !control {
                    problems.push(format!(
                        "{}x{} {field:?} {}: {}",
                        size.width,
                        size.height,
                        if backwards { "Shift+Tab" } else { "Tab" },
                        state_line(&mut s, sentence, nth)
                    ));
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "the error is not in view:\n{}",
        problems.join("\n")
    );
}

/// Enter in a field that is invalid (touching it) shows the line in view as well.
#[test]
fn the_error_line_is_in_view_after_enter_in_an_invalid_field() {
    let mut s = Session::new(Case::CustomizeAdvanced, SIZES[0]);
    let target = stop_of_text_input(&mut s, 3);
    tab_to(&mut s, target);
    s.apply(Message::WebManifestThemeColorChanged("not-a-colour".into()));
    s.apply(Message::WebManifestSubmitted(ManifestField::ThemeColor));
    assert!(
        pixels_outside(&mut s, COLOR_SENTENCE, 0).is_some_and(|px| px < 0.5),
        "{}",
        state_line(&mut s, COLOR_SENTENCE, 0)
    );
}
