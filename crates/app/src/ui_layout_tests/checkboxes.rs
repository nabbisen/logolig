//! The checkboxes of the Customize page (RFC 021, job J10): `snora::checkbox`es.
//!
//! Questions: is every checkbox a Tab stop; do a click, Enter and Space each publish **exactly one**
//! message, the same for all three and the one the iced checkbox published (the toggled value of
//! the current setting); is nothing published twice.

use iced::Size;
use iced::keyboard::{Key, key::Named};
use iced::mouse::Cursor;
use logolig::MessageKey;
use logolig_i18n::Locale;

use super::support::{Case, Combo, Probe, click_events, key_press, probe_of};
use crate::app::{AppState, Message};

/// Tall enough for the whole page, so that nothing is below the viewport.
const SIZE: Size = Size::new(720.0, 4000.0);

fn state() -> AppState {
    // Every option on and the advanced settings open: the widest set of controls.
    Combo {
        case: Case::CustomizeAdvanced,
        size: SIZE,
        locale: Locale::En,
        dark: false,
    }
    .state()
}

fn debug(messages: &[Message]) -> Vec<String> {
    messages.iter().map(|m| format!("{m:?}")).collect()
}

/// Every checkbox of the page: its label and the message its toggle must publish.
fn checkboxes(state: &AppState) -> Vec<(String, String)> {
    let t = &state.translator;
    let plan = &state.export_plan;
    let mut list: Vec<(String, String)> = Vec::new();
    for px in [16u32, 32, 48, 96, 192, 512] {
        let message = if plan.png_sizes.contains(&px) {
            format!("PngSizeRemoveRequested({px})")
        } else {
            format!("PngPresetSizeAdded({px})")
        };
        list.push((format!("{px} × {px}"), message));
    }
    let toggled = |name: &str, on: bool| format!("{name}({})", !on);
    list.push((
        t.t(MessageKey::KeepTransparency),
        toggled("KeepTransparencyToggled", plan.keep_transparency),
    ));
    list.push((
        t.t(MessageKey::IncludeAppleTouchLabel),
        toggled("IncludeAppleTouchToggled", plan.include_apple_touch),
    ));
    list.push((
        t.t(MessageKey::IncludeHtmlSnippetLabel),
        toggled("IncludeHtmlSnippetToggled", plan.include_html_snippet),
    ));
    list.push((
        t.t(MessageKey::IncludeSvgLabel),
        toggled("IncludeSvgToggled", plan.include_svg),
    ));
    list.push((
        t.t(MessageKey::VectorizeOnRasterLabel),
        toggled("VectorizeOnRasterToggled", plan.vectorize_on_raster),
    ));
    list.push((
        t.t(MessageKey::IncludeWebManifestLabel),
        toggled("IncludeWebManifestToggled", plan.web_manifest.is_some()),
    ));
    list.push((
        t.t(MessageKey::IncludeMicrosoftAppLogosLabel),
        toggled(
            "IncludeMicrosoftAppLogosToggled",
            plan.include_microsoft_app_logos,
        ),
    ));
    list.push((
        t.t(MessageKey::IncludeMonochromeLabel),
        toggled("IncludeMonochromeToggled", plan.monochrome),
    ));
    list
}

fn centre_of(p: &mut Probe<'_, Message>, label: &str) -> iced::Point {
    p.walk()
        .iter()
        .find(|s| s.text == label)
        .unwrap_or_else(|| panic!("no text {label:?} on the page"))
        .bounds
        .center()
}

/// The Tab-order index of the focus target that holds the point `at`.
fn target_at(p: &mut Probe<'_, Message>, at: iced::Point) -> usize {
    p.focus_targets()
        .iter()
        .position(|t| t.bounds.contains(at))
        .unwrap_or_else(|| panic!("no focus target at {at:?}"))
}

#[test]
fn every_checkbox_is_a_tab_stop() {
    let state = state();
    let list = checkboxes(&state);
    assert_eq!(
        list.len(),
        14,
        "the 6 sizes, Keep transparency and the 7 options"
    );
    let mut p = probe_of(&state, SIZE);
    for (label, _) in &list {
        let at = centre_of(&mut p, label);
        let index = target_at(&mut p, at);
        assert!(
            !p.focus_targets()[index].text_input,
            "{label:?} is a text input?"
        );
    }
    // The targets of the page: the 14 checkboxes and the 5 text inputs (the custom size and the
    // four manifest fields). The slider and the pick lists are not targets yet (step 4).
    let targets = p.focus_targets();
    assert_eq!(targets.iter().filter(|t| t.text_input).count(), 5);
}

/// A click, Enter and Space each publish exactly one message, the same for the three, and it is
/// the toggled value of the current setting.
#[test]
fn a_click_enter_and_space_publish_the_one_toggle_message() {
    let state = state();
    for (label, expected) in checkboxes(&state) {
        // The mouse.
        let mut p = probe_of(&state, SIZE);
        let at = centre_of(&mut p, &label);
        let (_s, messages) = p.send(&click_events(at), Cursor::Available(at));
        assert_eq!(
            debug(&messages),
            std::slice::from_ref(&expected),
            "{label:?}: a click"
        );

        // The keys, on the focused checkbox.
        for key in [Named::Enter, Named::Space] {
            let mut p = probe_of(&state, SIZE);
            let at = centre_of(&mut p, &label);
            let index = target_at(&mut p, at);
            p.focus_target(index);
            let (_s, messages) = p.send(&[key_press(Key::Named(key))], Cursor::Unavailable);
            assert_eq!(
                debug(&messages),
                std::slice::from_ref(&expected),
                "{label:?}: {key:?}"
            );
        }
    }
}

/// A size the user added is a checked checkbox of its own; unchecking removes it.
#[test]
fn a_user_added_size_is_a_checkbox_that_removes_it() {
    let mut state = state();
    state.export_plan.png_sizes.push(256);
    let mut p = probe_of(&state, SIZE);
    let at = centre_of(&mut p, "256 × 256");
    let (_s, messages) = p.send(&click_events(at), Cursor::Available(at));
    assert_eq!(debug(&messages), ["PngSizeRemoveRequested(256)"]);
    let index = target_at(&mut p, at);
    let mut p = probe_of(&state, SIZE);
    p.focus_target(index);
    let (_s, messages) = p.send(&[key_press(Key::Named(Named::Space))], Cursor::Unavailable);
    assert_eq!(debug(&messages), ["PngSizeRemoveRequested(256)"]);
}
