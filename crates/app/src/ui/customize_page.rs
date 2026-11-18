//! Customize page (v1.22.0).
//!
//! When the user selects "Customize" in the side nav, the main body switches
//! to this page. It wraps the same controls as the former right-side settings
//! drawer (`advanced_drawer`) but with no width constraint and no "Settings"
//! heading — the full window width is available.
//!
//! The page scrolls as **one** area (RFC 027 J14a): the sections are in it, "Reset to defaults"
//! sits below it and stays in view. The paddings keep the content where it was when the sections
//! had a scroll area of their own: its left edge is 52 px from the side bar, its top 44 px from
//! the top, and a focus ring (2 px outside a control) has room inside the area.

use iced::widget::{column, container, scrollable};
use iced::{Element, Length, Padding};

use crate::app::{AppState, Message};

/// Root view. Called from `shell::body()`.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let sections = container(crate::ui::advanced_drawer::body(state))
        .width(Length::Fill)
        .max_width(720)
        .padding(Padding {
            top: 44.0,
            right: 52.0,
            bottom: 20.0,
            left: 52.0,
        });
    let footer = container(crate::ui::advanced_drawer::footer(state)).padding(Padding {
        top: 14.0,
        right: 52.0,
        bottom: 20.0,
        left: 52.0,
    });

    column![
        scrollable(sections)
            .width(Length::Fill)
            .height(Length::Fill),
        footer
    ]
    .height(Length::Fill)
    .into()
}
