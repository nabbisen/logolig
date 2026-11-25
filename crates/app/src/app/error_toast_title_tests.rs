//! `update` tests: the title of the error toast is translated (RFC 016, job K4).

use super::*;

use logolig_i18n::Locale;

fn title_in(locale: Locale) -> String {
    let mut state = AppState {
        translator: logolig_i18n::Translator::for_locale(locale),
        ..AppState::default()
    };
    toast::push_error_toast(&mut state, logolig::AppError::export("disk full"));
    assert_eq!(state.toasts.len(), 1, "one error toast");
    state.toasts[0].title.clone()
}

#[test]
fn the_error_toast_title_follows_the_language() {
    assert_eq!(title_in(Locale::En), "Operation failed");
    assert_eq!(title_in(Locale::Ja), "処理に失敗しました");
}
