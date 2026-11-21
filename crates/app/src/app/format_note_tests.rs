//! `update` tests: the one notice for a file whose name contradicts its content (RFC 014, D1).
//!
//! A mislabelled JPEG is explained **once**: the notice carries the JPEG advisory's sentence, and
//! the advisory that `PreviewBuilt` would push for a JPEG is not pushed again. A mislabelled PNG
//! gets exactly one notice. A file with an unknown extension, or a matching one, gets none.

use super::test_support::{dummy_source_asset, noted, opaque_preview_cache, plain};
use super::*;

use logolig::SourceKind;

fn jpeg_asset(name: &str) -> SourceAsset {
    SourceAsset {
        path: PathBuf::from(name),
        kind: SourceKind::Jpeg,
        ..dummy_source_asset()
    }
}

/// Drops `name`, completes the ingest with `ingested` and the preview: the whole flow that can raise a toast.
fn load(state: &mut AppState, name: &str, ingested: logolig::Ingested) {
    let _ = update(state, Message::FileDropped(PathBuf::from(name)));
    let job = state.job_id;
    let _ = update(state, Message::IngestCompleted(job, Ok(ingested)));
    let cache = opaque_preview_cache(name, state.export_plan.algorithm);
    let _ = update(state, Message::PreviewBuilt(job, Ok(cache)));
}

#[test]
fn a_jpeg_named_png_gets_exactly_one_toast_that_holds_the_notice_and_the_advisory() {
    let mut state = AppState::default();
    load(
        &mut state,
        "logo.png",
        noted(jpeg_asset("logo.png"), SourceKind::Png, SourceKind::Jpeg),
    );
    assert_eq!(
        state.toasts.len(),
        1,
        "one toast, not a notice and an advisory"
    );
    let toast = &state.toasts[0];
    assert_eq!(
        toast.title,
        state.translator.t(MessageKey::ToastFormatMismatchTitle)
    );
    assert!(
        toast.message.contains("JPEG") && toast.message.contains("PNG"),
        "{}",
        toast.message
    );
    assert!(
        toast
            .message
            .contains(&state.translator.t(MessageKey::ToastFormatMismatchJpegTail)),
        "the JPEG advisory's news is in it: {}",
        toast.message
    );
    // The shortened body (RFC 014 review): one paragraph, without the advisory's opening sentence.
    assert!(!toast.message.contains('\n'), "{}", toast.message);
    assert!(
        !toast.message.contains("will use this image"),
        "the redundant opening sentence is gone: {}",
        toast.message
    );
    assert!(state.source_format_note.is_some());
}

#[test]
fn a_png_named_jpg_gets_exactly_one_toast_without_the_jpeg_sentence() {
    let mut state = AppState::default();
    load(
        &mut state,
        "logo.jpg",
        noted(dummy_source_asset(), SourceKind::Jpeg, SourceKind::Png),
    );
    // The preview of an opaque raster also triggers the generic "no transparency" advisory: that
    // is a different message about different news, and it is pushed by `PreviewBuilt` as before.
    let notices: Vec<_> = state
        .toasts
        .iter()
        .filter(|t| t.title == state.translator.t(MessageKey::ToastFormatMismatchTitle))
        .collect();
    assert_eq!(notices.len(), 1, "exactly one name notice");
    assert!(
        !notices[0]
            .message
            .contains(&state.translator.t(MessageKey::ToastFormatMismatchJpegTail))
    );
}

#[test]
fn an_unknown_extension_and_a_matching_one_get_no_name_notice() {
    for (name, asset) in [
        ("logo", dummy_source_asset()),
        ("logo.png", dummy_source_asset()),
    ] {
        let mut state = AppState::default();
        load(&mut state, name, plain(asset));
        assert!(
            state
                .toasts
                .iter()
                .all(|t| t.title != state.translator.t(MessageKey::ToastFormatMismatchTitle)),
            "{name}: no notice"
        );
        assert!(state.source_format_note.is_none(), "{name}");
    }
}

#[test]
fn a_matching_jpeg_still_gets_its_own_advisory_once() {
    let mut state = AppState::default();
    load(&mut state, "photo.jpg", plain(jpeg_asset("photo.jpg")));
    assert_eq!(state.toasts.len(), 1);
    assert_eq!(
        state.toasts[0].title,
        state.translator.t(MessageKey::ToastJpegInputTitle)
    );
}

#[test]
fn the_next_file_forgets_the_note_and_a_second_mislabelled_file_gets_its_own_notice() {
    let mut state = AppState::default();
    load(
        &mut state,
        "a.png",
        noted(jpeg_asset("a.png"), SourceKind::Png, SourceKind::Jpeg),
    );
    assert!(state.source_format_note.is_some());
    let _ = update(&mut state, Message::FileDropped(PathBuf::from("b.png")));
    assert!(
        state.source_format_note.is_none(),
        "a new file starts without a note"
    );
    state.toasts.clear();
    let job = state.job_id;
    let _ = update(
        &mut state,
        Message::IngestCompleted(
            job,
            Ok(noted(
                jpeg_asset("b.png"),
                SourceKind::Png,
                SourceKind::Jpeg,
            )),
        ),
    );
    assert_eq!(state.toasts.len(), 1);
}

#[test]
fn the_notice_is_in_japanese_when_the_language_is() {
    let mut state = AppState {
        translator: logolig_i18n::Translator::for_locale(logolig_i18n::Locale::Ja),
        ..AppState::default()
    };
    load(
        &mut state,
        "logo.png",
        noted(jpeg_asset("logo.png"), SourceKind::Png, SourceKind::Jpeg),
    );
    assert_eq!(state.toasts.len(), 1);
    assert!(
        state.toasts[0].title.contains("ファイル名"),
        "{}",
        state.toasts[0].title
    );
}
