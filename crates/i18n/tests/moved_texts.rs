//! The texts that were written in the code and are keys now (RFC 016, job K4), pinned in both
//! languages, with the one placeholder.

use logolig::MessageKey;
use logolig_i18n::{Locale, Translator};

const TEXTS: &[(MessageKey, &str, &str)] = &[
    (
        MessageKey::ToastErrorTitle,
        "Operation failed",
        "処理に失敗しました",
    ),
    (MessageKey::FileDialogImagesFilter, "Images", "画像"),
    (
        MessageKey::FileDialogChooseTitle,
        "Choose a PNG, SVG, WebP, or JPEG to forge favicons from",
        "favicon のもとにする PNG、SVG、WebP、JPEG の画像を選んでください",
    ),
    (
        MessageKey::PreviewBrowserTabCaption,
        "Browser tab — 16×16 actual size",
        "ブラウザタブ — 16×16 の実寸",
    ),
    (
        MessageKey::PreviewSmartphoneHomeCaption,
        "Smartphone home — 60pt @2x",
        "スマートフォンのホーム — 60pt @2x",
    ),
    (
        MessageKey::ResultBadgePngMono,
        "PNG mono",
        "PNG グレースケール",
    ),
];

#[test]
fn every_moved_text_is_the_confirmed_one_in_both_languages() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    for (key, english, japanese) in TEXTS {
        assert_eq!(en.t(*key), *english, "{key:?} in English");
        assert_eq!(ja.t(*key), *japanese, "{key:?} in Japanese");
    }
}

/// The history card's count: a label and a number, with no plural form to get wrong ("1 files").
#[test]
fn the_history_count_takes_the_number_and_has_no_plural() {
    for (locale, one, many) in [
        (Locale::En, "Files: 1", "Files: 7"),
        (Locale::Ja, "ファイル数: 1", "ファイル数: 7"),
    ] {
        let t = Translator::for_locale(locale);
        assert_eq!(
            t.t_args(MessageKey::HistoryFileCount, &[("count", "1")]),
            one
        );
        assert_eq!(
            t.t_args(MessageKey::HistoryFileCount, &[("count", "7")]),
            many
        );
    }
}
