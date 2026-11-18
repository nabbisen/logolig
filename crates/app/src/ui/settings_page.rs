//! Settings page (v1.22.0).
//!
//! When the user selects "Settings" in the side nav, the main body switches
//! to this page. It replaces the v1.18.0 locale and theme popup overlays with
//! a full-page layout — more readable, one click to reach.

use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Theme};

use logolig::{MessageKey, ThemeMode};
use logolig_i18n::Locale;

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::choice::radio_group;
use crate::ui::colors;

/// Root view. Called from `shell::body()`.
pub fn view(state: &AppState) -> Element<'_, Message> {
    let theme = resolve_theme(state);
    let t = &state.translator;

    let language_section = section(
        t.t(MessageKey::SidebarLabelLocale),
        radio_group(
            [Some(Locale::Ja), Some(Locale::En), None],
            &state.locale_override,
            |locale| match locale {
                Some(Locale::Ja) => t.t(MessageKey::LocaleNameJa),
                Some(Locale::En) => t.t(MessageKey::LocaleNameEn),
                None => t.t(MessageKey::LocaleSystem),
            },
            Message::LocalePicked,
        )
        .into(),
        &theme,
    );

    let theme_section = section(
        t.t(MessageKey::SidebarLabelTheme),
        radio_group(
            [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System],
            &state.theme,
            |mode| match mode {
                ThemeMode::Light => t.t(MessageKey::ThemeNameLight),
                ThemeMode::Dark => t.t(MessageKey::ThemeNameDark),
                ThemeMode::System => t.t(MessageKey::ThemeSystem),
            },
            Message::ThemePicked,
        )
        .into(),
        &theme,
    );

    let shortcuts_section = shortcuts_section(state, &theme);

    // The page scrolls as one (RFC 026), like Customize: at a height or width that does not hold
    // the content nothing is squeezed out; the content and its order do not change.
    scrollable(
        container(
            column![language_section, theme_section, shortcuts_section]
                .spacing(32)
                .padding([24, 32])
                .max_width(480),
        )
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ---------------------------------------------------------------------------
// Section
// ---------------------------------------------------------------------------

fn section<'a>(
    title: String,
    content: Element<'a, Message>,
    theme: &Theme,
) -> Element<'a, Message> {
    let heading = text(title).size(15).color(colors::page_title(theme));

    column![heading, content].spacing(12).into()
}

// ---------------------------------------------------------------------------
// Keyboard shortcuts (RFC 019 phase 2)
// ---------------------------------------------------------------------------

/// Width of the column that holds the key combinations.
const SHORTCUT_KEYS_WIDTH: f32 = 140.0;

/// The list of global shortcuts and the honest note about what the keyboard cannot do yet.
///
/// The key labels are not translated; the command key is `Ctrl` (`Cmd` on macOS).
fn shortcuts_section<'a>(state: &'a AppState, theme: &Theme) -> Element<'a, Message> {
    let t = &state.translator;
    let cmd = crate::shortcuts::command_key_label();

    let rows = [
        (format!("{cmd}+O"), MessageKey::ShortcutChooseFile),
        (format!("{cmd}+S"), MessageKey::ShortcutDownloadAll),
        ("Esc".to_string(), MessageKey::ShortcutEscape),
        (format!("{cmd}+1 / 2 / 3"), MessageKey::ShortcutPages),
        ("F1".to_string(), MessageKey::ShortcutHelp),
    ];
    let text_color = theme.extended_palette().background.base.text;

    let list = rows
        .into_iter()
        .fold(column![].spacing(8), |list, (keys, action)| {
            list.push(
                row![
                    text(keys)
                        .size(14)
                        .color(colors::page_title(theme))
                        .width(Length::Fixed(SHORTCUT_KEYS_WIDTH)),
                    text(t.t(action)).size(14).color(text_color),
                ]
                .spacing(12),
            )
        });

    let note = text(t.t(MessageKey::ShortcutsLimitNote))
        .size(13)
        .color(colors::muted_text(theme));

    section(
        t.t(MessageKey::ShortcutsSectionTitle),
        column![list, note].spacing(12).into(),
        theme,
    )
}
