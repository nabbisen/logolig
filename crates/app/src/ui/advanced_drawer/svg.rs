//! The SVG section of the Customize page: vectorisation, resize algorithm and trace preset rows.

use iced::widget::{checkbox, column, container, pick_list, row, text};
use iced::{Alignment, Element, Padding};

use logolig::{MessageKey, ResizeAlgorithm, VtracerPreset};

use crate::app::{AppState, Message};

use super::choices::{
    LocalizedAlgorithm, LocalizedPreset, algorithm_message_key, preset_message_key,
};

/// SVG section: checkbox + indented sub-options.
pub(super) fn svg_subsection<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let main_toggle = checkbox(state.export_plan.include_svg)
        .label(t.t(MessageKey::IncludeSvgLabel))
        .on_toggle(Message::IncludeSvgToggled)
        .text_size(13);

    // When include_svg is off, no need to show sub-options
    if !state.export_plan.include_svg {
        return main_toggle.into();
    }

    // Sub-options: vectorise toggle + preset pick_list, indented to show hierarchy.
    let nested = column![
        checkbox(state.export_plan.vectorize_on_raster)
            .label(t.t(MessageKey::VectorizeOnRasterLabel))
            .on_toggle(Message::VectorizeOnRasterToggled)
            .text_size(13),
        vtracer_preset_row(state),
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

pub(super) fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    // i18n wrapper. Avoid implementing Display on ResizeAlgorithm directly
    // (core must not depend on i18n); wrap it in the UI layer instead.
    let options: Vec<LocalizedAlgorithm> = ResizeAlgorithm::all()
        .iter()
        .map(|a| LocalizedAlgorithm {
            value: *a,
            label: state.translator.t(algorithm_message_key(*a)),
        })
        .collect();
    let selected = options
        .iter()
        .find(|opt| opt.value == state.export_plan.algorithm)
        .cloned();
    let picker = pick_list(options, selected, |opt: LocalizedAlgorithm| {
        Message::AlgorithmChanged(opt.value)
    })
    .text_size(13);

    row![text(t.t(MessageKey::AlgorithmLabel)).size(13), picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn vtracer_preset_row<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let options: Vec<LocalizedPreset> = VtracerPreset::all()
        .iter()
        .map(|p| LocalizedPreset {
            value: *p,
            label: state.translator.t(preset_message_key(*p)),
        })
        .collect();
    let selected = options
        .iter()
        .find(|opt| opt.value == state.export_plan.vtracer_preset)
        .cloned();
    let picker = pick_list(options, selected, |opt: LocalizedPreset| {
        Message::VtracerPresetChanged(opt.value)
    })
    .text_size(13);

    row![text(t.t(MessageKey::PresetLabel)).size(13), picker]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}
