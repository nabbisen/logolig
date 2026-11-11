//! `update` tests: file drop, ingest and the advisory toasts (non-square, SVG).

use super::*;

use super::test_support::{
    SVG_BOTH, SVG_IMAGE, SVG_PLAIN, SVG_TEXT, analyzed, dummy_result_assets, dummy_source_asset,
    ingest_ok, state_with_svg_source, svg_asset,
};

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
    assert!(state.result_assets.is_none());
    assert!(!state.result_preview_open);
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
    let task = update(
        &mut png,
        Message::IngestCompleted(0, Ok(dummy_source_asset())),
    );
    assert_eq!(task.units(), 2, "PNG source: no analysis");
    let mut svg = AppState::default();
    let task = update(
        &mut svg,
        Message::IngestCompleted(0, Ok(svg_asset("logo.svg"))),
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
