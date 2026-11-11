//! Button styles of the Customize page: reset, secondary and the extras header.

use iced::Color;

// ---------------------------------------------------------------------------
// v1.15.0: footer button styles
// ---------------------------------------------------------------------------

/// Reset button (left-aligned, moderately destructive).
///
/// "Restore defaults" destroys configuration, so it needs slightly more
/// visual warning than a normal secondary button, but less than a fully
/// destructive `danger` style (no irreversible file deletion).
///
/// Compromise:
/// - Transparent background + danger.weak.color border to signal caution
/// - text_color matches normal text (readable)
/// - On hover: light danger.weak fill to confirm it is pressable
pub(super) fn reset_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => Color {
            a: 0.15,
            ..palette.danger.weak.color
        },
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.base.text,
        border: iced::Border {
            color: palette.danger.weak.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// v1.17.0: Successor to `close_button_style`. Used for auxiliary buttons
/// (e.g. "Add custom size"). Transparent background + neutral border.
pub(super) fn secondary_drawer_button_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.weak.text,
        border: iced::Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..Default::default()
    }
}

/// v1.17.0: Collapsible "▶ Advanced settings" section header button style.
///
/// Full-width click target, subtle hover background, no border.
/// Behaves like an accordion chevron control.
pub(super) fn extras_header_style(
    theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let palette = theme.extended_palette();
    let bg = match status {
        iced::widget::button::Status::Hovered => palette.background.weak.color,
        _ => Color::TRANSPARENT,
    };
    iced::widget::button::Style {
        background: Some(iced::Background::Color(bg)),
        text_color: palette.background.base.text,
        border: iced::Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}
