//! Toast helpers: the push functions, the toast id counter and the advisory toasts
//! (RFC 017: moved out of `app.rs`, unchanged apart from visibility).

use snora::{Toast, ToastIntent, ToastLifetime};

use logolig::{AppError, MessageKey, services::transparency_audit::TransparencyReport};

use super::{AppState, Message};

/// Pushes a persistent error toast. Stays until the user dismisses it.
/// v1.5.0: translates the error via `AppError::key()` + `args()`.
pub(super) fn push_error_toast(state: &mut AppState, err: AppError) {
    let id = next_id(state);
    let body = state.translator.translate_error(&err);
    // Title is a generic 'operation failed' string.
    // Currently hardcoded to English; a dedicated MessageKey could be added later.

    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Error,
            "Operation failed",
            body,
            Message::DismissToast(id),
        )
        .persistent(),
    );
}

/// Pushes a short-lived success toast; its lifetime is proportional to its text (RFC 029).
pub(super) fn push_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Success,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        .with_lifetime(crate::toast_time::lifetime_for(title, body)),
    );
}

/// Pushes a longer (7-second) transient success toast (v1.4.2).
///
/// v1.4.1 originally used a persistent toast that required manual dismiss.
/// User feedback showed persistent was overkill but 4 s was too short.
/// v1.4.2 settled on a 7-second transient.
///
/// 7 s rationale: snora's default 4 s suits short messages; export notifications
/// can include long paths and need extra read time. 10+ s feels intrusive,
/// so 7 s is the pragmatic middle ground.
///
/// Toast position is bottom-right (snora default). Improving this is tracked
/// as a future snora upstream request.
pub(super) fn push_long_success_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Success,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        // 7 s is a minimum: a long saved path may need more (RFC 029).
        .with_lifetime(ToastLifetime::seconds(
            crate::toast_time::seconds_for(title, body).max(7),
        )),
    );
}

fn next_id(state: &mut AppState) -> u64 {
    let id = state.next_toast_id;
    state.next_toast_id += 1;
    id
}

/// Pushes a transient warning toast for input validation failures.
/// Sits between push_error_toast (persistent) and push_success_toast in severity.
pub(super) fn push_warning_toast(state: &mut AppState, title: &str, body: &str) {
    let id = next_id(state);
    state.toasts.push(
        Toast::new(
            id,
            ToastIntent::Warning,
            title.to_string(),
            body.to_string(),
            Message::DismissToast(id),
        )
        .with_lifetime(crate::toast_time::lifetime_for(title, body)),
    );
}

/// v1.7.0: transparency audit warning. Shown when the loaded image is fully opaque
/// or fully transparent. Uses the standard transient lifetime — this is an advisory
/// notice, not a blocking error.
pub(super) fn push_transparency_warning(state: &mut AppState, report: TransparencyReport) {
    let (title_key, body_key) = match report {
        TransparencyReport::FullyOpaque => (
            MessageKey::ToastFullyOpaqueTitle,
            MessageKey::ToastFullyOpaqueBody,
        ),
        TransparencyReport::FullyTransparent => (
            MessageKey::ToastFullyTransparentTitle,
            MessageKey::ToastFullyTransparentBody,
        ),
        TransparencyReport::HasTransparency => {
            // needs_warning() == false here; defensive no-op.
            return;
        }
    };
    let title = state.translator.t(title_key);
    let body = state.translator.t(body_key);
    push_warning_toast(state, &title, &body);
}

/// RFC 010: `true` when the intrinsic sides differ by more than 2 %
/// (`abs(w - h) * 100 > 2 * max(w, h)`). The tolerance avoids false alarms from
/// the `ceil()` rounding in `parse_svg_size`. Unknown size is never non-square.
pub(super) fn is_non_square(size: Option<(u32, u32)>) -> bool {
    match size {
        Some((w, h)) => u64::from(w.abs_diff(h)) * 100 > 2 * u64::from(w.max(h)),
        None => false,
    }
}

/// RFC 010: educational warning for a non-square source. The logo is shown in
/// full with transparent padding, so the icon may look small. Transient, non-blocking.
pub(super) fn push_non_square_warning(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastNonSquareInputTitle);
    let body = state.translator.t(MessageKey::ToastNonSquareInputBody);
    push_warning_toast(state, &title, &body);
}

/// RFC 015 phase 2a: one combined warning for an SVG whose parts are not shown. The body holds
/// the applicable sentences (images first, then text), one per line, so there is never more than
/// one toast per loaded SVG. Transient and non-blocking.
pub(super) fn push_svg_parts_warning(state: &mut AppState, report: logolig::SvgReport) {
    let title = state.translator.t(MessageKey::ToastSvgPartsTitle);
    let mut sentences: Vec<String> = Vec::with_capacity(2);
    if report.ignored_external_images > 0 {
        sentences.push(state.translator.t(MessageKey::ToastSvgImagesBody));
    }
    if report.has_text {
        sentences.push(state.translator.t(MessageKey::ToastSvgTextBody));
    }
    push_warning_toast(state, &title, &sentences.join("\n"));
}

/// v1.11.0: educational warning for JPEG input.
///
/// JPEG cannot carry alpha, so the v1.7 transparency audit always returns
/// FullyOpaque. Unlike the generic opaque warning (forgot to remove background),
/// this is a format constraint. A dedicated educational message suggests
/// using PNG for better favicon quality.
pub(super) fn push_jpeg_input_warning(state: &mut AppState) {
    let title = state.translator.t(MessageKey::ToastJpegInputTitle);
    let body = state.translator.t(MessageKey::ToastJpegInputBody);
    push_warning_toast(state, &title, &body);
}
