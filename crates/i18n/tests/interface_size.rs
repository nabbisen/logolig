//! The texts of the Interface size setting (RFC 034, job J24), pinned by key in both languages.

use logolig::MessageKey;
use logolig_i18n::{Locale, Translator};

const TEXTS: &[(MessageKey, &str, &str)] = &[
    (
        MessageKey::SectionInterfaceSize,
        "Interface size",
        "表示の大きさ",
    ),
    (
        MessageKey::SectionInterfaceSizeBlurb,
        "Makes text and buttons larger.",
        "文字とボタンをまとめて大きくします。",
    ),
    (MessageKey::InterfaceSizeNormal, "Normal", "標準"),
    (MessageKey::InterfaceSizeLarge, "Large", "大きい"),
    (
        MessageKey::InterfaceSizeExtraLarge,
        "Extra large",
        "とても大きい",
    ),
];

#[test]
fn every_text_of_the_setting_is_the_confirmed_one_in_both_languages() {
    let en = Translator::for_locale(Locale::En);
    let ja = Translator::for_locale(Locale::Ja);
    for (key, english, japanese) in TEXTS {
        assert_eq!(en.t(*key), *english, "en {key:?}");
        assert_eq!(ja.t(*key), *japanese, "ja {key:?}");
    }
}

/// The three names are words, not percentages (RFC 034 D2), and the blurb makes no claim about the
/// system's own scaling: under X11 the application does not follow it (README, checked), so the
/// sentence would be untrue there.
#[test]
fn the_names_are_words_and_the_blurb_promises_nothing_about_the_system_scale() {
    for locale in [Locale::En, Locale::Ja] {
        let t = Translator::for_locale(locale);
        for key in [
            MessageKey::InterfaceSizeNormal,
            MessageKey::InterfaceSizeLarge,
            MessageKey::InterfaceSizeExtraLarge,
        ] {
            let name = t.t(key);
            assert!(
                !name.chars().any(|c| c.is_ascii_digit() || c == '%'),
                "{locale:?} {key:?}: {name}"
            );
        }
    }
    let en = Translator::for_locale(Locale::En).t(MessageKey::SectionInterfaceSizeBlurb);
    assert!(!en.to_lowercase().contains("system"), "{en}");
    let ja = Translator::for_locale(Locale::Ja).t(MessageKey::SectionInterfaceSizeBlurb);
    assert!(
        !ja.contains("システム") && !ja.contains("画面の拡大率"),
        "{ja}"
    );
}
