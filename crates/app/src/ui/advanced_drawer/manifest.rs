//! The sub-section layout helpers and the web manifest form of the Customize page.

use iced::widget::{Space, column, container, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

use snora::Icon;
use snora::lucide;
use snora::widget::icon_element_sized;

use logolig::MessageKey;

use crate::app::{AppState, ManifestField, Message};
use crate::ui::choice::checkbox;
use crate::ui::type_scale;

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
    let mut col = column![text(title.to_string()).size(type_scale::BODY),].spacing(2);
    if let Some(b) = blurb {
        col = col.push(
            text(b.to_string())
                .size(type_scale::SECONDARY)
                .color(muted_color),
        );
    }
    col = col.push(container(body).padding(Padding::default().top(4)));
    container(col)
        .padding(Padding::default().top(2).bottom(2))
        .into()
}

/// v1.8.0: Web manifest section body.
///
/// When off: toggle only. When on: four text inputs (name / short_name / theme_color /
/// background_color) stacked vertically, each with a line under it for its error.
///
/// RFC 012 (K2): the fields show their **edit buffers** (`state.manifest_inputs`); the plan only
/// holds valid values. A field shows its error only once it was typed in or submitted: a lucide
/// warning icon and a plain sentence that says what is accepted, never colour alone. Enter commits
/// (`WebManifestSubmitted`) and shows the canonical form.
pub(super) fn web_manifest_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let toggle = checkbox(state.export_plan.web_manifest.is_some())
        .label(t.t(MessageKey::IncludeWebManifestLabel))
        .on_toggle(Message::IncludeWebManifestToggled);

    if state.export_plan.web_manifest.is_none() {
        // Off: show only the toggle. Revealing all fields upfront
        // adds cognitive load; expand only when on.
        return column![toggle].spacing(8).into();
    }

    let field = |field: ManifestField,
                 label: MessageKey,
                 placeholder: String,
                 on_change: fn(String) -> Message| {
        let error = state.manifest_inputs.shows_error(field).then(|| {
            t.t(if field.is_color() {
                MessageKey::WebManifestColorError
            } else {
                MessageKey::WebManifestNameError
            })
        });
        labeled_input(
            &t.t(label),
            state.manifest_inputs.get(field),
            &placeholder,
            on_change,
            Message::WebManifestSubmitted(field),
            error,
        )
    };
    column![
        toggle,
        field(
            ManifestField::Name,
            MessageKey::WebManifestNameLabel,
            t.t(MessageKey::WebManifestNamePlaceholder),
            Message::WebManifestNameChanged,
        ),
        field(
            ManifestField::ShortName,
            MessageKey::WebManifestShortNameLabel,
            t.t(MessageKey::WebManifestShortNamePlaceholder),
            Message::WebManifestShortNameChanged,
        ),
        field(
            ManifestField::ThemeColor,
            MessageKey::WebManifestThemeColorLabel,
            "#RRGGBB".to_string(),
            Message::WebManifestThemeColorChanged,
        ),
        field(
            ManifestField::BackgroundColor,
            MessageKey::WebManifestBackgroundColorLabel,
            "#RRGGBB".to_string(),
            Message::WebManifestBackgroundColorChanged,
        ),
    ]
    .spacing(8)
    .into()
}

const LABEL_WIDTH: f32 = 140.0;
const INPUT_WIDTH: f32 = 220.0;
const ROW_SPACING: f32 = 8.0;

/// One label + text_input field, and under the input a line for its error (RFC 012). The line is
/// **always** in the tree (an empty one when there is no error), so an error appearing does not
/// reshape the tree and the field being typed in keeps its focus.
fn labeled_input<'a>(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: fn(String) -> Message,
    on_submit: Message,
    error: Option<String>,
) -> Element<'a, Message> {
    let input_row = row![
        text(label.to_string())
            .size(type_scale::BODY)
            .width(Length::Fixed(LABEL_WIDTH)),
        text_input(placeholder, value)
            .on_input(on_change)
            .on_submit(on_submit)
            .size(type_scale::BODY)
            .padding(6)
            .style(crate::ui::styles::input_style)
            .width(Length::Fixed(INPUT_WIDTH)),
    ]
    .spacing(ROW_SPACING)
    .align_y(Alignment::Center);
    let message: Element<'a, Message> = match error {
        Some(sentence) => container(
            row![
                icon_element_sized::<Message>(
                    &Icon::Lucide(lucide::AlertTriangle),
                    type_scale::ICON_SMALL
                ),
                text(sentence).size(type_scale::BODY),
            ]
            .spacing(6)
            .align_y(Alignment::Start),
        )
        .width(Length::Fixed(LABEL_WIDTH + ROW_SPACING + INPUT_WIDTH))
        .padding(Padding::default().left(LABEL_WIDTH + ROW_SPACING))
        .into(),
        None => Space::new().into(),
    };
    // The field and its line are one thing for `keep_in_view`: the explanation is in view with the field.
    container(column![input_row, message].spacing(2))
        .id(crate::focus::with_its_line())
        .into()
}
