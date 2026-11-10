use super::*;
use std::sync::Arc;

use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

fn dummy_source_asset() -> SourceAsset {
    SourceAsset {
        path: PathBuf::from("logo.png"),
        kind: logolig::SourceKind::Png,
        raw: Arc::<[u8]>::from([0x89, b'P', b'N', b'G'].as_slice()),
        intrinsic_size: Some((1, 1)),
    }
}

fn dummy_result_assets() -> ResultAssets {
    ResultAssets {
        items: vec![ResultAssetItem {
            file_name: "favicon-snippet.html".to_string(),
            relative_path: "favicon-snippet.html".to_string(),
            bytes: b"<link rel=\"icon\" href=\"/favicon.ico\">".to_vec(),
            kind: ResultAssetKind::HtmlSnippet,
            dimensions: None,
            thumbnail: None,
        }],
    }
}

#[test]
fn nav_page_selected_switches_page() {
    let mut state = AppState::default();
    assert_eq!(state.nav_page, NavPage::Home); // precondition

    let _ = update(&mut state, Message::NavPageSelected(NavPage::Settings));
    assert_eq!(state.nav_page, NavPage::Settings);

    let _ = update(&mut state, Message::NavPageSelected(NavPage::Home));
    assert_eq!(state.nav_page, NavPage::Home);
}

#[test]
fn advanced_toggled_routes_to_customize() {
    // v1.22.0 behaviour pin: the former settings-drawer toggle now
    // navigates to the Customize page rather than flipping a bool.
    let mut state = AppState::default();
    let _ = update(&mut state, Message::AdvancedToggled);
    assert_eq!(state.nav_page, NavPage::Customize);
}

#[test]
fn theme_picked_updates_theme() {
    let mut state = AppState::default();
    assert_eq!(state.theme, ThemeMode::System); // default
    let _ = update(&mut state, Message::ThemePicked(ThemeMode::Dark));
    assert_eq!(state.theme, ThemeMode::Dark);
}

#[test]
fn keep_transparency_toggle_sets_plan() {
    let mut state = AppState::default();
    assert!(state.export_plan.keep_transparency); // default true
    let _ = update(&mut state, Message::KeepTransparencyToggled(false));
    assert!(!state.export_plan.keep_transparency);
    let _ = update(&mut state, Message::KeepTransparencyToggled(true));
    assert!(state.export_plan.keep_transparency);
}

#[test]
fn png_preset_size_add_inserts_sorted() {
    let mut state = AppState::default();
    assert_eq!(state.export_plan.png_sizes, vec![32, 192, 512]); // default
    let _ = update(&mut state, Message::PngPresetSizeAdded(64));
    assert_eq!(state.export_plan.png_sizes, vec![32, 64, 192, 512]);
}

#[test]
fn edit_cancelled_preserves_last_result_history() {
    let mut state = AppState {
        screen: Screen::Result,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        result_preview_open: true,
        file_drag_hovering: true,
        ..AppState::default()
    };

    let _ = update(&mut state, Message::EditCancelled);

    assert_eq!(state.screen, Screen::Empty);
    assert!(state.source_asset.is_some());
    assert!(state.result_assets.is_some());
    assert!(state.preview.is_none());
    assert!(state.preview_cache.is_none());
    assert!(state.transparency.is_none());
    assert!(!state.result_preview_open);
    assert!(!state.file_drag_hovering);
}

#[test]
fn show_last_result_requested_reopens_result_without_reconversion() {
    let mut state = AppState {
        screen: Screen::Empty,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    };

    let _ = update(&mut state, Message::ShowLastResultRequested);

    assert_eq!(state.screen, Screen::Result);
    assert_eq!(
        state.result_assets.as_ref().map(ResultAssets::count),
        Some(1)
    );
    assert_eq!(
        state
            .source_asset
            .as_ref()
            .map(|asset| asset.display_name()),
        Some("logo.png".to_string())
    );
}

#[test]
fn file_drag_hover_messages_toggle_drop_zone_feedback() {
    let mut state = AppState::default();

    let _ = update(&mut state, Message::FileDragHovered);
    assert!(state.file_drag_hovering);

    let _ = update(&mut state, Message::FileDragLeft);
    assert!(!state.file_drag_hovering);
}

#[test]
fn file_drop_clears_hover_and_starts_converting() {
    let mut state = AppState {
        file_drag_hovering: true,
        ..AppState::default()
    };
    let path = PathBuf::from("new-logo.png");

    let _ = update(&mut state, Message::FileDropped(path.clone()));

    assert!(!state.file_drag_hovering);
    assert_eq!(state.source_path, Some(path));
    assert_eq!(state.screen, Screen::Converting);
    assert!(state.busy);
    assert!(state.result_assets.is_none());
    assert!(!state.result_preview_open);
}
fn ingest_ok(state: &mut AppState, size: Option<(u32, u32)>) {
    let asset = SourceAsset {
        intrinsic_size: size,
        ..dummy_source_asset()
    };
    let _ = update(state, Message::IngestCompleted(Ok(asset)));
}

#[test]
fn non_square_ingest_adds_exactly_one_advisory_toast() {
    let mut state = AppState::default();
    ingest_ok(&mut state, Some((64, 32)));
    assert_eq!(state.toasts.len(), 1);
    let expected = state.translator.t(MessageKey::ToastNonSquareInputTitle);
    assert_eq!(state.toasts[0].title, expected);
}

#[test]
fn square_ingest_adds_no_toast() {
    let mut state = AppState::default();
    ingest_ok(&mut state, Some((100, 100)));
    assert!(state.toasts.is_empty());
}

#[test]
fn nearly_square_ingest_inside_the_tolerance_adds_no_toast() {
    let mut state = AppState::default();
    ingest_ok(&mut state, Some((101, 100)));
    assert!(state.toasts.is_empty());
}

#[test]
fn unknown_size_ingest_adds_no_toast() {
    let mut state = AppState::default();
    ingest_ok(&mut state, None);
    assert!(state.toasts.is_empty());
}

#[test]
fn showing_the_last_result_again_does_not_repeat_the_advisory() {
    let mut state = AppState::default();
    ingest_ok(&mut state, Some((64, 32)));
    assert_eq!(state.toasts.len(), 1);
    state.result_assets = Some(dummy_result_assets());
    let _ = update(&mut state, Message::EditCancelled);
    let _ = update(&mut state, Message::ShowLastResultRequested);
    assert_eq!(state.toasts.len(), 1, "no second toast");
}

#[test]
fn non_square_tolerance_boundary_is_two_percent() {
    assert!(!is_non_square(Some((100, 102)))); // 1.96 %: inside
    assert!(is_non_square(Some((100, 103)))); // 2.9 %: outside
    assert!(!is_non_square(Some((0, 0))));
    assert!(is_non_square(Some((1000, 1))));
}
// ---- RFC 015 phase 2a: SVG advisory ------------------------------------

const SVG_TEXT: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><text x="2" y="30" font-size="20">Hi</text></svg>"#;
const SVG_IMAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/></svg>"#;
const SVG_BOTH: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="64" height="64"><image width="8" height="8" xlink:href="/some/file.png"/><text x="2" y="30" font-size="20">Hi</text></svg>"#;
const SVG_PLAIN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64"><rect width="8" height="8"/></svg>"#;

fn svg_asset(path: &str) -> SourceAsset {
    SourceAsset {
        path: PathBuf::from(path),
        kind: logolig::SourceKind::Svg,
        raw: Arc::<[u8]>::from(b"<svg/>".as_slice()),
        intrinsic_size: Some((64, 64)),
    }
}

/// State with `logo.svg` as the current source, as after a successful ingest.
fn state_with_svg_source() -> AppState {
    AppState {
        source_asset: Some(svg_asset("logo.svg")),
        ..AppState::default()
    }
}

fn analyzed(state: &mut AppState, path: &str, svg: &str) {
    let report = logolig::services::svg::analyze(svg.as_bytes());
    let _ = update(state, Message::SvgAnalyzed(PathBuf::from(path), report));
}

#[test]
fn svg_with_text_and_images_gets_one_toast_with_both_sentences_images_first() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_BOTH);
    assert_eq!(
        state.toasts.len(),
        1,
        "exactly one toast, never one per issue"
    );
    let t = &state.translator;
    assert_eq!(state.toasts[0].title, t.t(MessageKey::ToastSvgPartsTitle));
    let expected = format!(
        "{}\n{}",
        t.t(MessageKey::ToastSvgImagesBody),
        t.t(MessageKey::ToastSvgTextBody)
    );
    assert_eq!(state.toasts[0].message, expected);
}

#[test]
fn svg_with_only_images_gets_only_the_images_sentence() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_IMAGE);
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(
        state.toasts[0].message,
        state.translator.t(MessageKey::ToastSvgImagesBody)
    );
}

#[test]
fn svg_with_only_text_gets_only_the_text_sentence() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_TEXT);
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(
        state.toasts[0].message,
        state.translator.t(MessageKey::ToastSvgTextBody)
    );
}

#[test]
fn plain_svg_gets_no_toast() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_PLAIN);
    assert!(state.toasts.is_empty());
}

#[test]
fn stale_analysis_result_is_ignored() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "an-older-logo.svg", SVG_BOTH);
    assert!(
        state.toasts.is_empty(),
        "result for a file that is no longer current"
    );
    let mut no_source = AppState::default();
    analyzed(&mut no_source, "logo.svg", SVG_BOTH);
    assert!(no_source.toasts.is_empty(), "no current source at all");
}

#[test]
fn analysis_is_started_only_for_svg_sources() {
    // Preview and conversion are always started (2 units); an SVG adds the analysis (3).
    let mut png = AppState::default();
    let task = update(&mut png, Message::IngestCompleted(Ok(dummy_source_asset())));
    assert_eq!(task.units(), 2, "PNG source: no analysis");
    let mut svg = AppState::default();
    let task = update(
        &mut svg,
        Message::IngestCompleted(Ok(svg_asset("logo.svg"))),
    );
    assert_eq!(task.units(), 3, "SVG source: analysis started");
}

#[test]
fn going_back_and_showing_the_last_result_does_not_repeat_the_svg_toast() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_BOTH);
    assert_eq!(state.toasts.len(), 1);
    state.result_assets = Some(dummy_result_assets());
    let _ = update(&mut state, Message::EditCancelled);
    let _ = update(&mut state, Message::ShowLastResultRequested);
    assert_eq!(state.toasts.len(), 1, "no second toast");
}
// ---- RFC 029: toast lifetime proportional to the content ---------------

use crate::toast_time::seconds_for;
use snora::ToastLifetime;

/// The lifetime a toast must have: transient, for the seconds its own text needs.
fn expect_content_lifetime(toast: &Toast<Message>) {
    assert_eq!(
        toast.lifetime,
        ToastLifetime::seconds(seconds_for(&toast.title, &toast.message)),
        "toast {:?} / {:?}",
        toast.title,
        toast.message
    );
}

#[test]
fn non_square_advisory_lives_as_long_as_its_text_needs() {
    let mut state = AppState::default();
    ingest_ok(&mut state, Some((64, 32)));
    assert_eq!(state.toasts.len(), 1);
    expect_content_lifetime(&state.toasts[0]);
    assert_eq!(
        state.toasts[0].lifetime,
        ToastLifetime::seconds(12),
        "English text"
    );
}

#[test]
fn svg_parts_advisory_lives_as_long_as_its_text_needs() {
    let mut state = state_with_svg_source();
    analyzed(&mut state, "logo.svg", SVG_BOTH);
    assert_eq!(state.toasts.len(), 1);
    expect_content_lifetime(&state.toasts[0]);
    assert_eq!(
        state.toasts[0].lifetime,
        ToastLifetime::seconds(20),
        "English text, at the cap"
    );
}

#[test]
fn jpeg_advisory_lives_as_long_as_its_text_needs() {
    let mut state = AppState::default();
    push_jpeg_input_warning(&mut state);
    assert_eq!(state.toasts.len(), 1);
    expect_content_lifetime(&state.toasts[0]);
    assert_eq!(
        state.toasts[0].lifetime,
        ToastLifetime::seconds(17),
        "English text"
    );
}

#[test]
fn invalid_size_warning_lives_as_long_as_its_text_needs() {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::PngSizeInputChanged("5".to_string()));
    let _ = update(&mut state, Message::PngSizeAddRequested);
    assert_eq!(state.toasts.len(), 1, "out-of-range size warns once");
    expect_content_lifetime(&state.toasts[0]);
}

#[test]
fn short_success_toast_follows_the_content_rule() {
    let mut state = AppState::default();
    let _ = update(&mut state, Message::ExportPlanResetRequested);
    assert_eq!(
        state.toasts.len(),
        1,
        "reset confirms with one success toast"
    );
    expect_content_lifetime(&state.toasts[0]);
}

#[test]
fn long_success_toast_keeps_seven_seconds_as_a_minimum() {
    let mut state = AppState::default();
    push_long_success_toast(&mut state, "Saved", "/tmp/a.png");
    assert_eq!(
        state.toasts[0].lifetime,
        ToastLifetime::seconds(7),
        "short text: the minimum"
    );
    // A very long path needs more than 7 s and gets it.
    let long_path = format!("/tmp/{}", "d".repeat(200));
    push_long_success_toast(&mut state, "Saved", &long_path);
    let needed = seconds_for("Saved", &long_path);
    assert!(needed > 7, "the fixture must need more than the minimum");
    assert_eq!(state.toasts[1].lifetime, ToastLifetime::seconds(needed));
}

#[test]
fn error_toast_stays_until_dismissed() {
    let mut state = AppState::default();
    push_error_toast(&mut state, AppError::unsupported_file("x.png"));
    assert_eq!(state.toasts[0].lifetime, ToastLifetime::Persistent);
}

// ---- RFC 019 phase 2: global shortcuts -------------------------------

use crate::shortcuts::Shortcut;

fn result_state() -> AppState {
    AppState {
        screen: Screen::Result,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    }
}

fn press(state: &mut AppState, shortcut: Shortcut) -> Task<Message> {
    update(state, Message::ShortcutPressed(shortcut))
}

/// The task the mouse path would start for `message`, as a count of started units.
fn units_of(state: &mut AppState, message: Message) -> usize {
    update(state, message).units()
}

#[test]
fn choose_file_starts_the_same_task_as_the_button_on_empty_and_result() {
    for screen in [Screen::Empty, Screen::Result] {
        let mut state = AppState {
            screen,
            ..result_state()
        };
        let mouse = units_of(&mut state, Message::PickFileRequested);
        assert!(mouse > 0, "the mouse path starts a task");
        let key = press(&mut state, Shortcut::ChooseFile).units();
        assert_eq!(key, mouse, "{screen:?}");
    }
}

#[test]
fn choose_file_does_nothing_while_converting_or_off_the_home_page() {
    let mut converting = AppState {
        screen: Screen::Converting,
        ..AppState::default()
    };
    assert_eq!(press(&mut converting, Shortcut::ChooseFile).units(), 0);
    for page in [NavPage::Customize, NavPage::Settings] {
        let mut state = AppState {
            nav_page: page,
            ..AppState::default()
        };
        assert_eq!(
            press(&mut state, Shortcut::ChooseFile).units(),
            0,
            "{page:?}"
        );
    }
}

#[test]
fn download_all_works_only_on_the_result_screen_with_assets() {
    let mut state = result_state();
    let mouse = units_of(&mut state, Message::DownloadAllRequested);
    assert!(mouse > 0);
    assert_eq!(press(&mut state, Shortcut::DownloadAll).units(), mouse);

    let mut empty = AppState::default();
    assert_eq!(press(&mut empty, Shortcut::DownloadAll).units(), 0, "Empty");

    let mut converting = AppState {
        screen: Screen::Converting,
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    };
    assert_eq!(
        press(&mut converting, Shortcut::DownloadAll).units(),
        0,
        "Converting"
    );

    let mut no_assets = AppState {
        screen: Screen::Result,
        ..AppState::default()
    };
    assert_eq!(
        press(&mut no_assets, Shortcut::DownloadAll).units(),
        0,
        "no assets"
    );

    for page in [NavPage::Customize, NavPage::Settings] {
        let mut state = AppState {
            nav_page: page,
            ..result_state()
        };
        assert_eq!(
            press(&mut state, Shortcut::DownloadAll).units(),
            0,
            "{page:?}"
        );
    }
}

#[test]
fn escape_dismisses_the_newest_toast_first_and_does_not_navigate() {
    let mut state = result_state();
    push_warning_toast(&mut state, "first", "one");
    push_warning_toast(&mut state, "second", "two");
    let (first_id, second_id) = (state.toasts[0].id, state.toasts[1].id);

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(state.toasts[0].id, first_id, "the newest one went first");
    assert_ne!(first_id, second_id);
    assert_eq!(
        state.screen,
        Screen::Result,
        "no navigation while a toast was open"
    );

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert!(state.toasts.is_empty());
    assert_eq!(state.screen, Screen::Result);

    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.screen, Screen::Empty, "with no toast, Esc goes back");
}

#[test]
fn escape_dismisses_a_toast_on_any_page_and_screen() {
    for page in [NavPage::Home, NavPage::Customize, NavPage::Settings] {
        for screen in [Screen::Empty, Screen::Converting, Screen::Result] {
            let mut state = AppState {
                nav_page: page,
                screen,
                ..AppState::default()
            };
            push_error_toast(&mut state, AppError::unsupported_file("x.png"));
            let _ = press(&mut state, Shortcut::EscapeOrBack);
            assert!(state.toasts.is_empty(), "{page:?} {screen:?}");
            assert_eq!(state.screen, screen);
            assert_eq!(state.nav_page, page);
        }
    }
}

#[test]
fn escape_without_a_toast_goes_back_only_from_the_home_result_screen() {
    for (page, screen) in [
        (NavPage::Home, Screen::Empty),
        (NavPage::Home, Screen::Converting),
        (NavPage::Customize, Screen::Result),
        (NavPage::Settings, Screen::Result),
    ] {
        let mut state = AppState {
            nav_page: page,
            screen,
            ..result_state()
        };
        let _ = press(&mut state, Shortcut::EscapeOrBack);
        assert_eq!(state.screen, screen, "{page:?} {screen:?}");
        assert_eq!(state.nav_page, page);
    }
    let mut state = result_state();
    let _ = press(&mut state, Shortcut::EscapeOrBack);
    assert_eq!(state.screen, Screen::Empty);
    assert!(
        state.result_assets.is_some(),
        "history is kept, as for the Back button"
    );
}

#[test]
fn page_shortcuts_and_help_set_the_nav_page_from_anywhere() {
    let mut state = AppState::default();
    for (shortcut, page) in [
        (Shortcut::Page(NavPage::Customize), NavPage::Customize),
        (Shortcut::Page(NavPage::Settings), NavPage::Settings),
        (Shortcut::Page(NavPage::Home), NavPage::Home),
        (Shortcut::Help, NavPage::Settings),
    ] {
        let _ = press(&mut state, shortcut);
        assert_eq!(state.nav_page, page, "{shortcut:?}");
    }
    // While converting, switching pages is allowed (the sidebar allows it too).
    let mut converting = AppState {
        screen: Screen::Converting,
        ..AppState::default()
    };
    let _ = press(&mut converting, Shortcut::Page(NavPage::Customize));
    assert_eq!(converting.nav_page, NavPage::Customize);
    assert_eq!(converting.screen, Screen::Converting);
}
