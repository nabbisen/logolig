//! `update` tests: toast lifetime proportional to the content (RFC 029).

use super::*;

use super::test_support::{SVG_BOTH, analyzed, ingest_ok, state_with_svg_source};

use crate::toast_time::seconds_for;
use snora::ToastLifetime;

// ---- RFC 029: toast lifetime proportional to the content ---------------

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
