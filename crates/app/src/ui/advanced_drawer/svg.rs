//! The SVG section of the Customize page: the SVG output option with its vectorisation toggle,
//! and the resize algorithm group.

use iced::widget::{column, container, text};
use iced::{Element, Padding};

use logolig::{MessageKey, ResizeAlgorithm};

use crate::app::{AppState, Message};
use crate::ui::choice::{checkbox, radio_group};
use crate::ui::colors;
use crate::ui::type_scale;

use super::choices::algorithm_message_key;

/// SVG section: checkbox + the indented vectorisation option.
///
/// The trace preset is not here any more: it is the "SVG conversion" group of the main section
/// (RFC 021), the one control for that setting.
pub(super) fn svg_subsection<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let main_toggle = checkbox(state.export_plan.include_svg)
        .label(t.t(MessageKey::IncludeSvgLabel))
        .on_toggle(Message::IncludeSvgToggled);

    // When include_svg is off, no need to show the sub-option
    if !state.export_plan.include_svg {
        return main_toggle.into();
    }

    // Sub-option: the vectorise toggle, indented to show hierarchy.
    let nested = column![
        checkbox(state.export_plan.vectorize_on_raster)
            .label(t.t(MessageKey::VectorizeOnRasterLabel))
            .on_toggle(Message::VectorizeOnRasterToggled),
    ]
    .spacing(6);

    // Indentation: left padding + small top/bottom padding
    column![
        main_toggle,
        container(nested).padding(Padding::default().left(20).top(4)),
    ]
    .spacing(4)
    .into()
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

/// The resize algorithm: its label above a group of the five algorithms (D2: the label is
/// always above the group; the group is vertical).
pub(super) fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    column![
        text(t.t(MessageKey::AlgorithmLabel))
            .size(type_scale::BODY)
            .color(colors::muted_text(&crate::app::resolve_theme(state))),
        radio_group(
            ResizeAlgorithm::all().iter().copied(),
            &state.export_plan.algorithm,
            |algorithm| t.t(algorithm_message_key(*algorithm)),
            Message::AlgorithmChanged,
        ),
    ]
    .spacing(4)
    .into()
}
