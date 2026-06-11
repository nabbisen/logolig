//! Top-level layout assembly (v1.22.0).
//!
//! ## Layout
//!
//! ```text
//!  desktop                          mobile
//!  ┌──┬──────────────────────┐      ┌──────────────────────┐
//!  │  │                      │      │                      │
//!  │  │     body (page)      │      │     body (page)      │
//!  │  │                      │      │                      │
//!  │  ├──────────────────────┤      ├──────────────────────┤
//!  │  │ app name  · tagline  │      │  Home  Customize  …  │
//!  └──┴──────────────────────┘      └──────────────────────┘
//!  ↑side_bar   ↑footer (app name)   ↑footer (nav)
//! ```
//!
//! No header. On desktop the app name and tagline appear in a small,
//! always-visible footer. On mobile the footer slot is taken by the nav bar.

use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};
use snora::{AppLayout, LayoutDirection, render};

use logolig_core::MessageKey;

use crate::app::{is_mobile, resolve_theme, AppState, Message, NavPage, Screen};
use crate::ui::{colors, drop_zone, sidebar};

pub fn view(state: &AppState) -> Element<'_, Message> {
    let mobile = is_mobile(state);

    // No .header() call — header has been removed.
    let mut layout = AppLayout::new(body(state))
        .direction(LayoutDirection::Ltr)
        .toasts(state.toasts.clone())
        .on_close_modals(Message::CloseModals);

    if mobile {
        // Mobile: footer holds the navigation bar.
        layout = layout.footer(sidebar::bottom_view(state));
    } else {
        // Desktop: sidebar for nav, footer for app name.
        layout = layout
            .side_bar(sidebar::view(state))
            .footer(app_name_footer(state));
    }

    render(layout)
}

/// Routes the main body to the page selected in the sidebar nav.
fn body(state: &AppState) -> Element<'_, Message> {
    match state.nav_page {
        // Home: the core app flow (drop zone → converting → result).
        NavPage::Home => match state.screen {
            Screen::Empty => drop_zone::view(state),
            Screen::Converting => crate::ui::converting::view(state),
            Screen::Result => crate::ui::result_view::view(state),
        },
        // Customize: full-page output settings (formerly the right-side drawer).
        NavPage::Customize => crate::ui::customize_page::view(state),
        // Settings: full-page language and theme selection.
        NavPage::Settings => crate::ui::settings_page::view(state),
    }
}

/// Desktop-only small footer: app name + tagline, always visible.
fn app_name_footer(state: &AppState) -> Element<'_, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    container(
        row![
            text(t.t(MessageKey::AppTitle))
                .size(12)
                .color(colors::app_name(&theme)),
            text(format!("— {}", t.t(MessageKey::AppTagline)))
                .size(11)
                .color(colors::muted_text(&theme)),
        ]
        .spacing(6)
        .align_y(Alignment::Center),
    )
    .padding([6, 16])
    .width(Length::Fill)
    .into()
}
