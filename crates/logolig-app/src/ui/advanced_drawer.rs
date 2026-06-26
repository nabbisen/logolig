//! Customize page content (§5.3).
//!
//! Contains all export settings. Originally a right-side sheet ("drawer"),
//! promoted to a full-page view in v1.22.0 via the Customize nav item.
//!
//! ## v1.10.0: information architecture revision
//!
//! Sections are arranged in the user's natural decision order:
//!
//! 1. **What to export** — artifact types and size groups
//! 2. **Extras** — Web manifest, monochrome (secondary, quieter visual weight)
//! 3. **Rendering quality** — resize algorithm
//!
//! Advanced (rarely-touched) settings are collapsed under a disclosure
//! chevron. The default PNG / ICO sizes are shown but visually de-emphasised
//! to signal "you probably don't need to change these".

use iced::widget::{button, checkbox, column, container, pick_list, row, text, text_input};
use iced::{Alignment, Color, Element, Length, Padding};

use logolig_core::{MessageKey, ResizeAlgorithm, VtracerPreset};

use crate::app::{AppState, Message, resolve_theme};
use crate::ui::colors;

// v1.14.0: HEADING_COLOR / MUTED_COLOR / BADGE_MUTED_BG hardcoded constants
// moved to theme-aware helpers in `crate::ui::colors`.
// Fixes "grey text invisible on grey background" in dark mode.

pub fn view<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let theme = resolve_theme(state);

    // ----- v1.17.0: right-sheet layout + flat section structure -----
    //
    // Section layout per design spec:
    //  1. Title row ("Settings" + × close button)  [removed in v1.22.0]
    //  2. PNG output sizes: checkboxes + "add custom size"
    //  3. SVG conversion: 3-position slider (Sharp / Default / PhotoRich)
    //  4. Misc: keep-transparency toggle
    //  5. ▶ Advanced settings (collapsible)
    //  6. Footer: ↻ Reset (left-aligned)
    //
    // The v1.15 three-accordion layout (`accordion_group` × 3) is replaced by
    // a flat vertical layout. Only "Advanced" remains collapsible,
    // as a place to hide settings that are rarely needed but should
    // remain accessible (chevron `▶` / `▼`).
    //
    // ICO generation section removed in v1.17.0 (confirmed in Q4). favicon.ico
    // is always output at default sizes (16/24/32/48 multi-frame); no user toggle.
    // Old ICO size editor removed entirely.

    // ─────────────────────────────────────────────────────────────
    // (1) PNG output sizes: 6 fixed checkboxes + custom size input
    // ─────────────────────────────────────────────────────────────
    //
    // Six preset PNG sizes (16/32/48/96/192/512).
    // 180 (apple-touch) is controlled separately; 256 is available via custom size.
    //
    // Each checkbox reflects whether the size is in export_plan.png_sizes:
    // checking adds it, unchecking removes it. Uses existing
    // PngSizeAddRequested / PngSizeRemoveRequested messages.
    let preset_sizes: [u32; 6] = [16, 32, 48, 96, 192, 512];
    let mut sizes_col = column![
        text(t.t(MessageKey::SectionPngOutputSizes))
            .size(17)
            .color(colors::section_label(&theme)),
    ]
    .spacing(8);
    for px in preset_sizes {
        let checked = state.export_plan.png_sizes.contains(&px);
        let label = format!("{} × {}", px, px);
        sizes_col = sizes_col.push(
            checkbox(checked)
                .label(label)
                .on_toggle(move |on| {
                    if on {
                        Message::PngPresetSizeAdded(px)
                    } else {
                        Message::PngSizeRemoveRequested(px)
                    }
                })
                .text_size(13),
        );
    }

    // Show any user-added sizes not in the preset list as removable custom chips.
    let mut custom_present: Vec<u32> = state
        .export_plan
        .png_sizes
        .iter()
        .copied()
        .filter(|p| !preset_sizes.contains(p))
        .collect();
    custom_present.sort();
    for px in &custom_present {
        let label = format!("{} × {}", px, px);
        let removed = *px;
        sizes_col = sizes_col.push(
            checkbox(true)
                .label(label)
                .on_toggle(move |_| Message::PngSizeRemoveRequested(removed))
                .text_size(13),
        );
    }

    // Custom size input: text_input + Add button in one row.

    let custom_input = row![
        text_input("e.g. 256", &state.png_size_input)
            .on_input(Message::PngSizeInputChanged)
            .on_submit(Message::PngSizeAddRequested)
            .size(12)
            .padding(4)
            .width(Length::Fixed(80.0)),
        button(text(t.t(MessageKey::AddCustomSize)).size(12))
            .padding([4, 10])
            .on_press(Message::PngSizeAddRequested)
            .style(secondary_drawer_button_style),
    ]
    .spacing(6)
    .align_y(Alignment::Center);
    sizes_col = sizes_col.push(custom_input);

    // ─────────────────────────────────────────────────────────────
    // (2) SVG conversion mode: 3-position discrete slider
    // ─────────────────────────────────────────────────────────────
    //
    // Maps vtracer presets (Sharp / Default / PhotoRich) to a Simple ↔
    // Detailed intuitive axis.
    // iced 0.14 slider is continuous; treat step as integer for discrete selection:
    //   0 → Sharp (Simple)
    //   1 → Default (Balanced)
    //   2 → PhotoRich (Detailed)
    //
    // "Vectorise raster" toggle and "favicon.svg on/off" are hidden in
    // Advanced settings (this section is only about "which mode").
    // Vectorisation reuses the existing vectorize_on_raster value
    // (default false).
    let preset_idx: i32 = match state.export_plan.vtracer_preset {
        VtracerPreset::Sharp => 0,
        VtracerPreset::Default => 1,
        VtracerPreset::PhotoRich => 2,
    };
    let svg_section = column![
        text(t.t(MessageKey::SectionSvgConversion))
            .size(17)
            .color(colors::section_label(&theme)),
        // Labels below the slider (Simple / Detailed)
        row![
            text(t.t(MessageKey::SvgConversionSimple))
                .size(11)
                .color(colors::muted_text(&theme)),
            iced::widget::Space::new().width(Length::Fill),
            text(t.t(MessageKey::SvgConversionDetailed))
                .size(11)
                .color(colors::muted_text(&theme)),
        ],
        iced::widget::slider(0..=2, preset_idx, |v| {
            let preset = match v {
                0 => VtracerPreset::Sharp,
                1 => VtracerPreset::Default,
                _ => VtracerPreset::PhotoRich,
            };
            Message::VtracerPresetChanged(preset)
        })
        .step(1i32),
    ]
    .spacing(6);

    // ─────────────────────────────────────────────────────────────
    // (3) Misc: keep-transparency toggle
    // ─────────────────────────────────────────────────────────────
    //
    // Implemented in v1.21.0. When off, raster outputs (PNG/ICO/mono) are
    // flattened against white via services::flatten. SVG outputs are unaffected.
    let misc_section = column![
        text(t.t(MessageKey::SectionMisc))
            .size(17)
            .color(colors::section_label(&theme)),
        checkbox(state.export_plan.keep_transparency)
            .label(t.t(MessageKey::KeepTransparency))
            .on_toggle(Message::KeepTransparencyToggled)
            .text_size(13),
    ]
    .spacing(8);

    // ─────────────────────────────────────────────────────────────
    // (4) ▶ Advanced settings (collapsible): rarely-needed settings
    // ─────────────────────────────────────────────────────────────
    //
    // Advanced settings not shown in the main sections:
    // - Apple touch icon (180×180 PNG)
    // - HTML snippet output
    // - Web manifest + its fields
    // - Monochrome (BT.709 greyscale)
    // - Resize algorithm
    // - Vectorise raster (vectorize_on_raster)
    let extras_chevron = if state.advanced_extras_open {
        "▼"
    } else {
        "▶"
    };
    let extras_header = button(
        row![
            text(extras_chevron)
                .size(11)
                .color(colors::group_heading(&theme)),
            text(t.t(MessageKey::AdvancedExtrasSection))
                .size(13)
                .color(colors::group_heading(&theme)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(Padding::default().top(8).bottom(8))
    .width(Length::Fill)
    .on_press(Message::AdvancedExtrasToggled)
    .style(extras_header_style);

    let extras_body: Element<'a, Message> = if state.advanced_extras_open {
        column![
            // Apple touch icon
            checkbox(state.export_plan.include_apple_touch)
                .label(t.t(MessageKey::IncludeAppleTouchLabel))
                .on_toggle(Message::IncludeAppleTouchToggled)
                .text_size(13),
            // HTML snippet
            checkbox(state.export_plan.include_html_snippet)
                .label(t.t(MessageKey::IncludeHtmlSnippetLabel))
                .on_toggle(Message::IncludeHtmlSnippetToggled)
                .text_size(13),
            // SVG output toggle + vectorise-raster (grouped)
            svg_subsection(state),
            // Web manifest (with child fields)
            subsection(
                &t.t(MessageKey::SectionWebManifest),
                Some(&t.t(MessageKey::SectionWebManifestBlurb)),
                web_manifest_body(state),
                colors::muted_text(&theme),
            ),
            // Monochrome
            subsection(
                &t.t(MessageKey::SectionMonochrome),
                Some(&t.t(MessageKey::SectionMonochromeBlurb)),
                checkbox(state.export_plan.monochrome)
                    .label(t.t(MessageKey::IncludeMonochromeLabel))
                    .on_toggle(Message::IncludeMonochromeToggled)
                    .text_size(13)
                    .into(),
                colors::muted_text(&theme),
            ),
            // Resize algorithm
            subsection(
                &t.t(MessageKey::SectionResize),
                Some(&t.t(MessageKey::SectionResizeBlurb)),
                algorithm_row(state),
                colors::muted_text(&theme),
            ),
        ]
        .spacing(12)
        .into()
    } else {
        iced::widget::Space::new().height(Length::Shrink).into()
    };

    let extras_section = column![extras_header, extras_body].spacing(2);

    // ─────────────────────────────────────────────────────────────
    // Main body: scrollable area
    // ─────────────────────────────────────────────────────────────
    let scroll_content = column![sizes_col, svg_section, misc_section, extras_section,].spacing(20);

    let scrollable_body = container(
        iced::widget::scrollable(scroll_content)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .height(Length::FillPortion(1))
    .width(Length::Fill);

    // ─────────────────────────────────────────────────────────────
    // (5) Footer: ↻ Reset only
    // ─────────────────────────────────────────────────────────────
    let footer = row![
        button(
            row![
                text("↻").size(13),
                text(t.t(MessageKey::ResetButton)).size(13),
            ]
            .spacing(6)
            .align_y(Alignment::Center),
        )
        .padding([8, 16])
        .on_press(Message::ExportPlanResetRequested)
        .style(reset_button_style),
        iced::widget::Space::new().width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    column![scrollable_body, footer]
        .spacing(14)
        .padding(20)
        .height(Length::Fill)
        .into()
}

/// SVG section: checkbox + indented sub-options.
fn svg_subsection<'a>(state: &'a AppState) -> Element<'a, Message> {
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

/// Sub-section within a group: title + optional blurb + body.
/// Successor to the old `section` helper (renamed when the group hierarchy grew).

///
/// v1.14.0: `muted_color` is resolved by the caller and passed in.
fn subsection<'a>(
    title: &str,
    blurb: Option<&str>,
    body: Element<'a, Message>,
    muted_color: Color,
) -> Element<'a, Message> {
    let mut col = column![text(title.to_string()).size(15),].spacing(2);
    if let Some(b) = blurb {
        col = col.push(text(b.to_string()).size(11).color(muted_color));
    }
    col = col.push(container(body).padding(Padding::default().top(4)));
    container(col)
        .padding(Padding::default().top(2).bottom(2))
        .into()
}

/// v1.8.0: Web manifest section body.
///
/// When off: toggle only. When on: four text inputs (name /
/// short_name / theme_color / background_color) stacked vertically.
///
/// Design decisions:
/// - No validation during typing (avoids mid-input warning UX)
/// - Validation runs at export time
/// - Persist on every keystroke (`persist_settings`) — consistent with other inputs

fn web_manifest_body<'a>(state: &'a AppState) -> Element<'a, Message> {
    let t = &state.translator;
    let toggle = checkbox(state.export_plan.web_manifest.is_some())
        .label(t.t(MessageKey::IncludeWebManifestLabel))
        .on_toggle(Message::IncludeWebManifestToggled)
        .text_size(13);

    let Some(manifest) = state.export_plan.web_manifest.as_ref() else {
        // Off: show only the toggle. Revealing all fields upfront
        // adds cognitive load; expand only when on.
        return column![toggle].spacing(8).into();
    };

    // On: four fields stacked vertically.
    // Each field is a label + text_input row.
    // Fixed-width labels would look cleaner, but v1.8 uses labeled_input for simplicity.
    column![
        toggle,
        labeled_input(
            &t.t(MessageKey::WebManifestNameLabel),
            &manifest.name,
            &t.t(MessageKey::WebManifestNamePlaceholder),
            Message::WebManifestNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestShortNameLabel),
            &manifest.short_name,
            &t.t(MessageKey::WebManifestShortNamePlaceholder),
            Message::WebManifestShortNameChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestThemeColorLabel),
            &manifest.theme_color,
            "#RRGGBB",
            Message::WebManifestThemeColorChanged,
        ),
        labeled_input(
            &t.t(MessageKey::WebManifestBackgroundColorLabel),
            &manifest.background_color,
            "#RRGGBB",
            Message::WebManifestBackgroundColorChanged,
        ),
    ]
    .spacing(8)
    .into()
}

/// One-row label + text_input field. Extracted as a helper for v1.8 form inputs.
/// Fixed label width (140 px) aligned with the text input.
fn labeled_input<'a>(
    label: &str,
    value: &str,
    placeholder: &str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(13).width(Length::Fixed(140.0)),
        text_input(placeholder, value)
            .on_input(on_change)
            .size(13)
            .width(Length::Fixed(220.0)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

fn algorithm_row<'a>(state: &'a AppState) -> Element<'a, Message> {
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

// ---------------------------------------------------------------------------
// i18n wrappers for pick_list (ResizeAlgorithm / VtracerPreset)
// ---------------------------------------------------------------------------
//
// `pick_list` requires `Display` on options, but implementing it on
// ResizeAlgorithm / VtracerPreset directly would leak i18n into core.
// Instead: UI-layer wrappers that carry a translated label string.

#[derive(Debug, Clone)]
struct LocalizedAlgorithm {
    value: ResizeAlgorithm,
    label: String,
}

impl PartialEq for LocalizedAlgorithm {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedAlgorithm {}

impl std::fmt::Display for LocalizedAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Debug, Clone)]
struct LocalizedPreset {
    value: VtracerPreset,
    label: String,
}

impl PartialEq for LocalizedPreset {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for LocalizedPreset {}

impl std::fmt::Display for LocalizedPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

// ---------------------------------------------------------------------------
// enum → MessageKey map (UI layer only, to keep core independent of i18n)
// ---------------------------------------------------------------------------

fn algorithm_message_key(alg: ResizeAlgorithm) -> MessageKey {
    match alg {
        ResizeAlgorithm::Lanczos3 => MessageKey::AlgorithmLanczos3,
        ResizeAlgorithm::MitchellNetravali => MessageKey::AlgorithmMitchellNetravali,
        ResizeAlgorithm::CatmullRom => MessageKey::AlgorithmCatmullRom,
        ResizeAlgorithm::Bilinear => MessageKey::AlgorithmBilinear,
        ResizeAlgorithm::Nearest => MessageKey::AlgorithmNearest,
    }
}

fn preset_message_key(preset: VtracerPreset) -> MessageKey {
    match preset {
        VtracerPreset::Sharp => MessageKey::VtracerPresetSharp,
        VtracerPreset::Default => MessageKey::VtracerPresetDefault,
        VtracerPreset::PhotoRich => MessageKey::VtracerPresetPhotoRich,
    }
}

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
fn reset_button_style(
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
fn secondary_drawer_button_style(
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

/// v1.17.0: The × button that was at the top-right of the drawer title row.
///
/// Light fill on hover only; minimal decoration (no border) — matches
/// the common modern "title-bar × button" appearance.
fn title_close_button_style(
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
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

/// v1.17.0: Collapsible "▶ Advanced settings" section header button style.
///
/// Full-width click target, subtle hover background, no border.
/// Behaves like an accordion chevron control.
fn extras_header_style(
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
