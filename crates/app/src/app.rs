//! Application core (§11).
//!
//! - holds `AppState`
//! - receives `Message` and drives state transitions
//! - wires `view` / `update` / `theme` / `subscription` into iced
//! - errors are surfaced as snora `Toast` notifications, not screen transitions
//!   (aligns with §12 ABDD: state is not expressed through colour alone)
//! - heavy work is offloaded via `iced::Task` to avoid blocking the UI thread (§2.4)

mod message;
mod persist;
mod state;
mod toast;
mod update;

pub use message::Message;
pub use state::{AppState, NavPage, Screen};
pub(crate) use state::{is_mobile, resolve_theme};

use iced::{Element, Subscription, Theme};

use logolig::MessageKey;

use update::update;

// Names the tests under `app/` reach through `use super::*` (the items they test moved out of
// this file; the tests themselves are unchanged).
#[cfg(test)]
use iced::Task;
#[cfg(test)]
use logolig::{AppError, SourceAsset, ThemeMode};
#[cfg(test)]
use snora::Toast;
#[cfg(test)]
use std::{path::PathBuf, time::Instant};
#[cfg(test)]
use toast::{
    is_non_square, push_error_toast, push_jpeg_input_warning, push_long_success_toast,
    push_warning_toast,
};

// ----------------------------------------------------------------------
// Entry point
// ----------------------------------------------------------------------

/// Entry function called from `main`.
///
/// iced 0.14's `application` takes a boot function as its first argument.
/// The window title is supplied via the `.title(...)` builder method.
/// v1.5.0: title is localised via `state.translator.t(MessageKey::AppTitle)`
/// so it updates automatically on locale change.
pub fn run() -> iced::Result {
    iced::application(AppState::boot, update, view)
        .font(snora::lucide::LUCIDE_FONT_BYTES)
        .window(crate::window::settings())
        .title(window_title)
        .theme(theme)
        .subscription(subscription)
        .run()
}

fn window_title(state: &AppState) -> String {
    state.translator.t(MessageKey::AppTitle)
}

/// Thin wrapper matching the `application().theme()` signature.
fn theme(state: &AppState) -> Theme {
    resolve_theme(state)
}

fn subscription(state: &AppState) -> Subscription<Message> {
    // Combine two subscriptions:
    //   (a) snora Toast tick — auto-dismisses transient toasts
    //   (b) iced runtime events — window resize, native file drag/drop and global shortcuts.
    //
    // v1.26.1: use `iced::event::listen_with` instead of `window::events()`.
    // It receives the same window events but keeps all event routing in one
    // runtime-event subscription and lets us observe FileHovered /
    // FilesHoveredLeft for visible drop-zone feedback.
    let toasts = snora::toast::subscription(&state.toasts, || Message::ToastTick);

    let runtime_events = iced::event::listen_with(|event, status, _id| match event {
        // RFC 019 phase 2: global shortcuts. A key a widget used (text typed into a field,
        // its own editing combinations) is not a shortcut.
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key,
            physical_key,
            modifiers,
            repeat,
            ..
        }) if status == iced::event::Status::Ignored => {
            crate::shortcuts::shortcut_for(&key, physical_key, modifiers, repeat)
                .map(Message::ShortcutPressed)
        }
        iced::Event::Window(iced::window::Event::FileHovered(_)) => Some(Message::FileDragHovered),
        iced::Event::Window(iced::window::Event::FilesHoveredLeft) => Some(Message::FileDragLeft),
        iced::Event::Window(iced::window::Event::FileDropped(path)) => {
            Some(Message::FileDropped(path))
        }
        iced::Event::Window(iced::window::Event::Resized(size)) => {
            Some(Message::WindowResized(size))
        }
        // v1.17.0: also respond to Opened so window_size is set at startup,
        // not just after the first explicit resize.
        iced::Event::Window(iced::window::Event::Opened { size, .. }) => {
            Some(Message::WindowResized(size))
        }
        _ => None,
    });

    Subscription::batch([toasts, runtime_events])
}

fn view(state: &AppState) -> Element<'_, Message> {
    crate::shell::view(state)
}

// ----------------------------------------------------------------------
// Tests
// ----------------------------------------------------------------------
//
// Smoke tests for the `update` state machine. They exercise the
// synchronous state transitions only; the returned `Task` is dropped,
// since each handler tested here either returns `Task::none()` or applies
// its state change synchronously before returning.
//
// No GUI test harness (e.g. iced_test) is used. `update` takes
// `&mut AppState` and `AppState: Default`, so plain construction plus
// assertion is sufficient. `persist_settings` is a no-op when
// `state.store` is `None` (the default), so these tests never touch disk.

#[cfg(test)]
mod toast_hold_tests;

#[cfg(test)]
mod tests;
