//! Integration tests for `logolig-i18n` (v1.5.0).
//!
//! Coverage:
//! - Bundled English dictionary parses without error (no missing keys)
//! - `Translator::t()` maps keys to the correct strings
//! - `Translator::t_args()` placeholder substitution works
//! - `Translator::translate_error()` is consistent with `AppError::args()`
//! - `Locale::from_bcp47` behaviour

use logolig::{AppError, MessageKey};
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
    assert!(
        result.contains("7"),
        "count placeholder should be substituted"
    );
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
    // Unspecified placeholders are left as-is (easier to debug).
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
    // English
    assert_eq!(Locale::from_bcp47("en"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("en-US"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("en_US"), Some(Locale::En));
    assert_eq!(Locale::from_bcp47("EN"), Some(Locale::En));
    // Japanese (v1.6.0)
    assert_eq!(Locale::from_bcp47("ja"), Some(Locale::Ja));
    assert_eq!(Locale::from_bcp47("ja-JP"), Some(Locale::Ja));
    assert_eq!(Locale::from_bcp47("ja_JP"), Some(Locale::Ja));
    // POSIX form (e.g. macOS / Linux `LANG=ja_JP.UTF-8`)
    assert_eq!(Locale::from_bcp47("ja_JP.UTF-8"), Some(Locale::Ja));
    // Unresolvable
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

// ---------------------------------------------------------------------------
// v1.6.0: Japanese dictionary tests
// ---------------------------------------------------------------------------

#[test]
fn japanese_dictionary_loads_without_panic() {
    // If ja.toml has a syntax error or missing field, Translator::for_locale
    // will panic at startup. Successful parse is the first assertion.
    let _t = Translator::for_locale(Locale::Ja);
}

#[test]
fn japanese_translator_locale_is_ja() {
    let t = Translator::for_locale(Locale::Ja);
    assert_eq!(t.locale(), Locale::Ja);
}

#[test]
fn japanese_app_title_is_logolig() {
    // Brand name is not localised; verify "Logolig" is returned in Japanese too
    let t = Translator::for_locale(Locale::Ja);
    assert_eq!(t.t(MessageKey::AppTitle), "Logolig");
}

#[test]
fn japanese_translation_differs_from_english_for_ui_text() {
    // Non-brand keys (descriptions etc.) should differ from their English equivalents.
    // Catching this property would reveal a regression like "ja.toml is just a copy
    // of en.toml".
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    let differing_keys = [
        MessageKey::DropZoneInstruction,
        MessageKey::AdvancedTitle,
        MessageKey::ResetButton,
        MessageKey::CloseButton,
        MessageKey::ExportButton,
        MessageKey::SectionLanguage,
        MessageKey::ToastNonSquareInputTitle,
        MessageKey::ToastNonSquareInputBody,
        MessageKey::ToastSvgPartsTitle,
        MessageKey::ToastSvgImagesBody,
        MessageKey::ToastSvgTextBody,
        MessageKey::ToastSvgLargeTitle,
        MessageKey::ToastSvgLargeBody,
    ];
    for key in differing_keys {
        let en_text = en.t(key);
        let ja_text = ja.t(key);
        assert_ne!(
            en_text, ja_text,
            "{:?} should differ between en/ja but both are {:?}",
            key, en_text
        );
    }
}

#[test]
fn japanese_every_key_has_non_empty_translation() {
    // Verify the same key set in Japanese. TOML-level missing fields are caught
    // by serde, but this test catches "filled with empty string"
    // as an additional guard.
    let t = Translator::for_locale(Locale::Ja);
    let keys = [
        MessageKey::AppTitle,
        MessageKey::DropZoneInstruction,
        MessageKey::DropZoneSecondary,
        MessageKey::ChooseFileButton,
        MessageKey::AdvancedTitle,
        MessageKey::SectionResize,
        MessageKey::SectionSvg,
        MessageKey::SectionFiles,
        MessageKey::SectionPngSizes,
        MessageKey::SectionIcoSizes,
        MessageKey::SectionLanguage,
        MessageKey::ResetButton,
        MessageKey::CloseButton,
        MessageKey::ExportButton,
        MessageKey::ToggleAdvancedButton,
        MessageKey::ToggleThemeButton,
        MessageKey::AlgorithmLanczos3,
        MessageKey::VtracerPresetSharp,
        MessageKey::LanguageEnglish,
        MessageKey::LanguageJapanese,
        MessageKey::LanguageSystemDefault,
        MessageKey::ToastExportTitle,
        MessageKey::ToastExportBody,
        MessageKey::ToastResetTitle,
        MessageKey::ToastInvalidSizeTitle,
        MessageKey::ToastNonSquareInputTitle,
        MessageKey::ToastNonSquareInputBody,
        MessageKey::ToastSvgPartsTitle,
        MessageKey::ToastSvgImagesBody,
        MessageKey::ToastSvgTextBody,
        MessageKey::ToastSvgLargeTitle,
        MessageKey::ToastSvgLargeBody,
        MessageKey::ErrorUnsupportedFile,
        MessageKey::ErrorIo,
        MessageKey::ErrorDecode,
        MessageKey::ErrorRasterize,
        MessageKey::ErrorResize,
        MessageKey::ErrorExport,
        MessageKey::ErrorNotImplemented,
    ];
    for key in keys {
        let s = t.t(key);
        assert!(!s.is_empty(), "ja key {:?} translated to empty string", key);
    }
}

#[test]
fn japanese_placeholder_substitution_works() {
    // ToastExportBody = "{count} ファイルを {dir} に書き出しました"
    let t = Translator::for_locale(Locale::Ja);
    let s = t.t_args(
        MessageKey::ToastExportBody,
        &[("count", "12"), ("dir", "/tmp/icons")],
    );
    assert_eq!(s, "12 ファイルを /tmp/icons に書き出しました");
}

#[test]
fn japanese_translate_error_substitutes_args() {
    // error_io = "{path} の入出力エラー: {cause}"
    let t = Translator::for_locale(Locale::Ja);
    let err = AppError::io("/tmp/foo", "permission denied");
    let s = t.translate_error(&err);
    assert!(s.contains("/tmp/foo"), "path missing: {s}");
    assert!(s.contains("permission denied"), "cause missing: {s}");
    assert!(s.contains("入出力"), "Japanese keyword missing: {s}");
}

// ---------------------------------------------------------------------------
// RFC 010: non-square advisory texts (fixed by the architect; do not reword)
// ---------------------------------------------------------------------------

#[test]
fn non_square_toast_texts_match_the_specification() {
    let en = Translator::for_locale(Locale::En);
    assert_eq!(
        en.t(MessageKey::ToastNonSquareInputTitle),
        "Your logo is not square"
    );
    assert_eq!(
        en.t(MessageKey::ToastNonSquareInputBody),
        "It is shown in full, with empty space around it, so the icon may look small. A square version of your logo gives a larger icon."
    );
    let ja = Translator::for_locale(Locale::Ja);
    assert_eq!(
        ja.t(MessageKey::ToastNonSquareInputTitle),
        "ロゴが正方形ではありません"
    );
    assert_eq!(
        ja.t(MessageKey::ToastNonSquareInputBody),
        "ロゴ全体を表示するため、アイコンの周りに余白が付きます。正方形のロゴ画像を用意すると、アイコンを大きく表示できます。"
    );
}

// ---------------------------------------------------------------------------
// RFC 015 phase 2a: SVG advisory texts (fixed by the architect; do not reword)
// ---------------------------------------------------------------------------

#[test]
fn svg_toast_texts_match_the_specification() {
    let en = Translator::for_locale(Locale::En);
    assert_eq!(
        en.t(MessageKey::ToastSvgPartsTitle),
        "Some parts of your SVG are not shown"
    );
    assert_eq!(
        en.t(MessageKey::ToastSvgImagesBody),
        "It refers to image files on your computer. For your safety Logolig does not read them. Embed the images in the SVG to include them."
    );
    assert_eq!(
        en.t(MessageKey::ToastSvgTextBody),
        "Logolig does not draw text in SVG files yet. Convert the text to outlines (paths) in your design tool and load the SVG again."
    );
    let ja = Translator::for_locale(Locale::Ja);
    assert_eq!(
        ja.t(MessageKey::ToastSvgPartsTitle),
        "SVG の一部が表示されません"
    );
    assert_eq!(
        ja.t(MessageKey::ToastSvgImagesBody),
        "SVG がこのコンピューター上の画像ファイルを参照しています。安全のため、Logolig はそれらを読み込みません。画像を SVG に埋め込むと反映されます。"
    );
    assert_eq!(
        ja.t(MessageKey::ToastSvgTextBody),
        "Logolig は SVG 内の文字をまだ描画できません。デザインツールで文字をアウトライン化（パス化）してから、SVG を読み込み直してください。"
    );
}

// ---------------------------------------------------------------------------
// RFC 019 phase 2: the keyboard shortcut list on the Settings page
// ---------------------------------------------------------------------------

const SHORTCUT_KEYS: [MessageKey; 7] = [
    MessageKey::ShortcutsSectionTitle,
    MessageKey::ShortcutChooseFile,
    MessageKey::ShortcutDownloadAll,
    MessageKey::ShortcutEscape,
    MessageKey::ShortcutPages,
    MessageKey::ShortcutHelp,
    MessageKey::ShortcutsLimitNote,
];

#[test]
fn shortcut_texts_exist_in_both_languages_and_differ() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    for key in SHORTCUT_KEYS {
        assert!(!en.t(key).is_empty(), "en {key:?} is empty");
        assert!(!ja.t(key).is_empty(), "ja {key:?} is empty");
        assert_ne!(en.t(key), ja.t(key), "{key:?} should differ between en/ja");
    }
}

#[test]
fn shortcut_texts_match_the_specification() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    let expected = [
        (
            MessageKey::ShortcutsSectionTitle,
            "Keyboard shortcuts",
            "キーボードショートカット",
        ),
        (
            MessageKey::ShortcutChooseFile,
            "Choose a file",
            "ファイルを選ぶ",
        ),
        (
            MessageKey::ShortcutDownloadAll,
            "Download all (ZIP)",
            "すべてダウンロード（ZIP）",
        ),
        (
            MessageKey::ShortcutEscape,
            "Close the message, or go back to the start",
            "メッセージを閉じる、または最初の画面に戻る",
        ),
        (
            MessageKey::ShortcutPages,
            "Go to Home, Customize or Settings",
            "ホーム・カスタマイズ・設定を開く",
        ),
        (MessageKey::ShortcutHelp, "Show this list", "この一覧を表示"),
        (
            MessageKey::ShortcutsLimitNote,
            "Moving between buttons and fields with the Tab key is not available yet.",
            "Tab キーでボタンや入力欄の間を移動する機能は、まだありません。",
        ),
    ];
    for (key, en_text, ja_text) in expected {
        assert_eq!(en.t(key), en_text, "en {key:?}");
        assert_eq!(ja.t(key), ja_text, "ja {key:?}");
    }
}

// ---------------------------------------------------------------------------
// RFC 030: the large-favicon.svg advisory
// ---------------------------------------------------------------------------

#[test]
fn svg_large_texts_exist_in_both_languages_and_differ() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    for key in [
        MessageKey::ToastSvgLargeTitle,
        MessageKey::ToastSvgLargeBody,
    ] {
        assert!(!en.t(key).is_empty(), "en {key:?} is empty");
        assert!(!ja.t(key).is_empty(), "ja {key:?} is empty");
        assert_ne!(en.t(key), ja.t(key), "{key:?} should differ between en/ja");
    }
}

#[test]
fn svg_large_texts_match_the_specification() {
    let en = Translator::for_locale(Locale::En);
    assert_eq!(
        en.t(MessageKey::ToastSvgLargeTitle),
        "favicon.svg is large and only approximate"
    );
    assert_eq!(
        en.t(MessageKey::ToastSvgLargeBody),
        "This image looks like a photograph, so tracing it to a vector gives a large file and only a rough likeness. Turn off SVG output in Customize if you do not need favicon.svg."
    );
    let ja = Translator::for_locale(Locale::Ja);
    assert_eq!(
        ja.t(MessageKey::ToastSvgLargeTitle),
        "favicon.svg が大きく、近似になります"
    );
    assert_eq!(
        ja.t(MessageKey::ToastSvgLargeBody),
        "この画像は写真のようなので、ベクターに変換すると大きなファイルになり、似せるだけの近似になります。favicon.svg が不要な場合は、「カスタマイズ」で SVG の出力をオフにしてください。"
    );
}
