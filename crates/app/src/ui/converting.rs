//! Converting screen (v1.16.0; replaces the old Importing / Exporting split).
//!
//! Shown after file ingestion while the export runs in the background.
//! Design decisions (F2):
//! - Indeterminate progress spinner only — no per-step accuracy.
//!   logolig's processing is fast enough (< 1 s for typical inputs) that
//!   a detailed progress bar would paradoxically make the wait feel longer.
//! - Show the source file name so the user knows what is being processed.

use iced::widget::{column, container, text};
use iced::{Element, Length};

use logolig::MessageKey;

use crate::app::{AppState, Message};
use crate::ui::cues;

/// View for the Converting screen.
pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;

    // Central label area. iced 0.14 has no built-in circular progress widget;
    // use a "busy icon + text" pattern for now (`ui/cues.rs`).
    // Per ABDD §12, convey "in progress" via a shape (not colour alone), from the bundled icon
    // font (RFC 021).
    //
    let main_label = cues::busy(
        t.t(MessageKey::ImportingMessage),
        crate::ui::type_scale::TITLE,
    );

    // Sub-message: show the source file name so the user knows what is being converted.
    // Defensively handle the case where the file name is not yet available:
    // show only the "please wait" message.
    let processing_subtext: Element<'a, Message> = if let Some(asset) = &state.source_asset {
        let label = format!(
            "{}: {}",
            t.t(MessageKey::PreviewSourceLabel),
            asset.display_name()
        );
        text(label).size(crate::ui::type_scale::BODY).into()
    } else {
        text(t.t(MessageKey::ImportingPleaseWait))
            .size(crate::ui::type_scale::BODY)
            .into()
    };

    let inner = column![main_label, processing_subtext]
        .spacing(12)
        .align_x(iced::alignment::Horizontal::Center);

    container(inner)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
