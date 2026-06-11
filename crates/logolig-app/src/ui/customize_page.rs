//! Customize page (v1.22.0).
//!
//! When the user selects "Customize" in the side nav, the main body switches
//! to this page. It wraps the same controls as the former right-side settings
//! drawer (`advanced_drawer`) but with no width constraint and no "Settings"
//! heading — the full window width is available.

use iced::widget::{container, scrollable};
use iced::{Element, Length};

use crate::app::{AppState, Message};

/// Root view. Called from `shell::body()`.
pub fn view(state: &AppState) -> Element<'_, Message> {
    scrollable(
        container(crate::ui::advanced_drawer::view(state))
            .width(Length::Fill)
            .max_width(720)
            .padding([24, 32]),
    )
    .into()
}
