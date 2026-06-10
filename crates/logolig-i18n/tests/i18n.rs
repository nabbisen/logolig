//! `logolig-i18n` の統合テスト (v1.5.0)。
//!
//! 主な検証項目:
//! - バンドル済み英語辞書がパースできること(キー漏れがないこと)
//! - `Translator::t()` がキーを正しい文字列にマッピングすること
//! - `Translator::t_args()` のプレースホルダ置換が動くこと
//! - `Translator::translate_error()` が `AppError::args()` と整合すること
//! - `Locale::from_bcp47` の挙動

use logolig_core::{AppError, MessageKey};
use logolig_i18n::{Locale, Translator};

#[test]
fn english_dictionary_loads_without_panic() {
    let t = Translator::for_locale(Locale::En);
    assert_eq!(t.t(MessageKey::AppTitle), "Logolig");
}

#[test]
fn translator_returns_template_text_for_simple_key() {
    let t = Translator::for_locale(Locale::En);
    assert_eq!(t.t(MessageKey::CloseButton), "Close");
    assert_eq!(t.t(MessageKey::ResetButton), "Reset to defaults");
    assert_eq!(t.t(MessageKey::ExportButton), "Export");
}

#[test]
fn translator_substitutes_placeholders() {
    let t = Translator::for_locale(Locale::En);
    let result = t.t_args(
        MessageKey::ToastExportBody,
        &[("count", "7"), ("dir", "/tmp/out")],
    );
    assert!(result.contains("7"), "count placeholder should be substituted");
    assert!(
        result.contains("/tmp/out"),
        "dir placeholder should be substituted"
    );
    assert!(
        !result.contains("{count}"),
        "raw placeholder should not remain"
    );
    assert!(!result.contains("{dir}"));
}

#[test]
fn translator_leaves_unknown_placeholders_intact() {
    // 未指定のプレースホルダは原文のまま残る (デバッグしやすい挙動)。
    let t = Translator::for_locale(Locale::En);
    let result = t.t_args(MessageKey::ToastExportBody, &[("count", "1")]);
    assert!(
        result.contains("{dir}"),
        "missing placeholder should remain literal: {result}"
    );
}

#[test]
fn translate_error_uses_app_error_args() {
    let t = Translator::for_locale(Locale::En);
    let err = AppError::io("/tmp/foo.png", "permission denied");
    let translated = t.translate_error(&err);
    assert!(translated.contains("/tmp/foo.png"));
    assert!(translated.contains("permission denied"));
    assert!(!translated.contains("{path}"));
    assert!(!translated.contains("{cause}"));
}

#[test]
fn translate_error_handles_all_variants() {
    let t = Translator::for_locale(Locale::En);
    let cases = [
        AppError::unsupported_file("foo.bin"),
        AppError::io("/x", "denied"),
        AppError::decode("bad PNG"),
        AppError::rasterize("bad SVG"),
        AppError::resize("zero size"),
        AppError::export("staging fail"),
        AppError::not_implemented("future thing"),
    ];
    for err in &cases {
        let translated = t.translate_error(err);
        assert!(
            !translated.contains('{') || !translated.contains('}'),
            "Error variant {err:?} left a placeholder: {translated}"
        );
    }
}

#[test]
fn locale_from_bcp47_handles_common_forms() {
    assert_eq!(Locale::from_bcp47("en"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("en-US"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("en_US"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("EN"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("xx"), None);
    assert_eq!(Locale::from_bcp47(""), None);
}

#[test]
fn locale_round_trips_through_bcp47() {
    for loc in Locale::all() {
        let tag = loc.as_bcp47();
        assert_eq!(
            Locale::from_bcp47(tag),
            Some(loc),
            "round-trip failed for {loc:?}"
        );
    }
}

#[test]
fn translator_default_is_english() {
    let t = Translator::default();
    assert_eq!(t.locale(), Locale::En);
    assert_eq!(t.t(MessageKey::AppTitle), "Logolig");
}
