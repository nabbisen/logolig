//! The sub-section layout helpers and the web manifest form of the Customize page.

use iced::widget::{checkbox, column, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

use logolig::MessageKey;

use crate::app::{AppState, Message};

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

/// Sub-section within a group: title + optional blurb + body.
/// Successor to the old `section` helper (renamed when the group hierarchy grew).
///
/// v1.14.0: `muted_color` is resolved by the caller and passed in.
pub(super) fn subsection<'a>(
    title: &str,
    blurb: Option<&str>,
    body: Element<'a, Message>,
    muted_color: Color,
) -> Element<'a, Message> {
    let mut col = column![text(title.to_string()).size(15),].spacing(2);
    if let Some(b) = blurb {
        col = col.push(text(b.to_string()).size(11).color(muted_color));
    }
    col = col.push(container(body).padding(Padding::default().top(4)));
    container(col)
        .padding(Padding::default().top(2).bottom(2))
        .into()
}

/// v1.8.0: Web manifest section body.
///
/// When off: toggle only. When on: four text inputs (name /
/// short_name / theme_color / background_color) stacked vertically.
///
/// Design decisions:
/// - No validation during typing (avoids mid-input warning UX)
/// - Validation runs at export time
/// - Persist on every keystroke (`persist_settings`) — consistent with other inputs
pub(super) fn web_manifest_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let toggle = checkbox(state.export_plan.web_manifest.is_some())
        .label(t.t(MessageKey::IncludeWebManifestLabel))
        .on_toggle(Message::IncludeWebManifestToggled)
        .text_size(13);

    let Some(manifest) = state.export_plan.web_manifest.as_ref() else {
        // Off: show only the toggle. Revealing all fields upfront
        // adds cognitive load; expand only when on.
        return column![toggle].spacing(8).into();
    };

    // On: four fields stacked vertically.
    // Each field is a label + text_input row.
    // Fixed-width labels would look cleaner, but v1.8 uses labeled_input for simplicity.
    column![
        toggle,
        labeled_input(
            &t.t(MessageKey::WebManifestNameLabel),
            &manifest.name,
            &t.t(MessageKey::WebManifestNamePlaceholder),
            Message::WebManifestNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestShortNameLabel),
            &manifest.short_name,
            &t.t(MessageKey::WebManifestShortNamePlaceholder),
            Message::WebManifestShortNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestThemeColorLabel),
            &manifest.theme_color,
            "#RRGGBB",
            Message::WebManifestThemeColorChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestBackgroundColorLabel),
            &manifest.background_color,
            "#RRGGBB",
            Message::WebManifestBackgroundColorChanged,
        ),
    ]
    .spacing(8)
    .into()
}

/// One-row label + text_input field. Extracted as a helper for v1.8 form inputs.
/// Fixed label width (140 px) aligned with the text input.
fn labeled_input<'a>(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(13).width(Length::Fixed(140.0)),
        text_input(placeholder, value)
            .on_input(on_change)
            .size(13)
            .width(Length::Fixed(220.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
