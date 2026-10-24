//! Global keyboard shortcuts (RFC 019 phase 2).

use iced::keyboard::key::Named;
use iced::keyboard::{Key, Modifiers, key};

use crate::app::NavPage;

/// What a recognised key combination stands for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Shortcut {
    ChooseFile,
    DownloadAll,
    EscapeOrBack,
    Page(NavPage),
    Help,
}

/// Map a key press to a shortcut. Pure: whether the shortcut may act now is decided by the handler.
///
/// A key repeat never triggers anything. Letters and digits are matched layout-independently
/// (`Key::to_latin`, which falls back to the physical key for non-Latin layouts, such as a
/// Japanese one) and need **exactly** the command modifier: Shift or Alt on top make it a
/// different combination. Escape and F1 need no modifier at all. Nothing else is bound.
pub(crate) fn shortcut_for(
    key: &Key,
    physical: key::Physical,
    modifiers: Modifiers,
    repeat: bool,
) -> Option<Shortcut> {
    if repeat {
        return None;
    }

    match key {
        Key::Named(Named::Escape) if modifiers.is_empty() => Some(Shortcut::EscapeOrBack),
        Key::Named(Named::F1) if modifiers.is_empty() => Some(Shortcut::Help),
        _ if modifiers == Modifiers::COMMAND => {
            // Caps Lock gives an upper-case logical key without Shift.
            match key.to_latin(physical)?.to_ascii_lowercase() {
                'o' => Some(Shortcut::ChooseFile),
                's' => Some(Shortcut::DownloadAll),
                '1' => Some(Shortcut::Page(NavPage::Home)),
                '2' => Some(Shortcut::Page(NavPage::Customize)),
                '3' => Some(Shortcut::Page(NavPage::Settings)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Name of the command key as shown to the user (not translated).
pub(crate) const fn command_key_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd"
    } else {
        "Ctrl"
    }
}

#[cfg(test)]
mod tests;
