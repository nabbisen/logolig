//! Tests for the toast lifetime rule (RFC 029): `seconds = clamp(ceil(2 + 0.3 * units), 4, 20)`.
//!
//! The reference values for the real advisories were computed by the architect with a separate
//! reference implementation of the rule; they must be reproduced exactly.

use super::*;

use logolig::MessageKey;
use logolig_i18n::{Locale, Translator};

fn words(n: usize) -> String {
    vec!["word"; n].join(" ")
}

/// Lifetime of each shipped advisory text in one locale, in the order
/// non-square, JPEG, SVG combined, SVG images only, fully opaque.
fn advisories(locale: Locale) -> [u64; 5] {
    let t = Translator::for_locale(locale);
    let svg_images = t.t(MessageKey::ToastSvgImagesBody);
    let svg_text = t.t(MessageKey::ToastSvgTextBody);
    [
        seconds_for(
            &t.t(MessageKey::ToastNonSquareInputTitle),
            &t.t(MessageKey::ToastNonSquareInputBody),
        ),
        seconds_for(
            &t.t(MessageKey::ToastJpegInputTitle),
            &t.t(MessageKey::ToastJpegInputBody),
        ),
        // The app joins the applicable sentences with a newline, images first.
        seconds_for(
            &t.t(MessageKey::ToastSvgPartsTitle),
            &format!("{svg_images}\n{svg_text}"),
        ),
        seconds_for(&t.t(MessageKey::ToastSvgPartsTitle), &svg_images),
        seconds_for(
            &t.t(MessageKey::ToastFullyOpaqueTitle),
            &t.t(MessageKey::ToastFullyOpaqueBody),
        ),
    ]
}

/// 1: empty and tiny toasts get the 4 s floor.
#[test]
fn t1_empty_and_one_word_toasts_get_the_floor() {
    assert_eq!(seconds_for("", ""), 4);
    assert_eq!(seconds_for("Saved", ""), 4);
    assert_eq!(seconds_for("", "Saved"), 4);
}

/// 1b: exact boundaries of the integer arithmetic (no floating-point rounding may move them).
#[test]
fn t1b_boundaries_are_exact() {
    assert_eq!(
        seconds_for("", &words(6)),
        4,
        "6 units: 2 + 1.8 = 3.8, floor 4"
    );
    assert_eq!(
        seconds_for("", &words(7)),
        5,
        "7 units: 2 + 2.1 = 4.1, rounded up"
    );
    assert_eq!(
        seconds_for("", &words(30)),
        11,
        "30 units: 2 + 9.0 = 11 exactly"
    );
    assert_eq!(
        seconds_for("", &words(31)),
        12,
        "31 units: 2 + 9.3 = 11.3, rounded up"
    );
}

/// 2: the real English advisories (architect's reference values).
#[test]
fn t2_english_advisories_match_the_reference_values() {
    assert_eq!(advisories(Locale::En), [12, 17, 20, 13, 12]);
}

/// 3: the real Japanese advisories (architect's reference values).
#[test]
fn t3_japanese_advisories_match_the_reference_values() {
    assert_eq!(advisories(Locale::Ja), [11, 15, 20, 12, 11]);
}

/// 4: a long path is not one word: 200 characters count as ceil(200 / 6) = 34 units.
#[test]
fn t4_a_long_single_token_counts_as_several_units() {
    let path = "a".repeat(200);
    assert_eq!(
        seconds_for("", &path),
        13,
        "ceil(2 + 0.3 * 34) = 13, not the floor"
    );
}

/// 5: far beyond the cap stays at 20.
#[test]
fn t5_the_cap_is_twenty_seconds() {
    assert_eq!(seconds_for("", &words(1000)), 20);
    assert_eq!(seconds_for(&words(500), &words(500)), 20);
}

/// 6: adding words never lowers the lifetime.
#[test]
fn t6_lifetime_is_monotonic_in_the_text_length() {
    let mut previous = 0;
    for n in 0..120 {
        let now = seconds_for("Title", &words(n));
        assert!(
            now >= previous,
            "{n} words gave {now}, fewer words gave {previous}"
        );
        previous = now;
    }
}

/// 7: each CJK character weighs 0.4 unit: 25 characters are 10 units, so ceil(2 + 0.3 * 10) = 5.
#[test]
fn t7_cjk_characters_weigh_two_fifths_of_a_unit() {
    assert_eq!(seconds_for("", &"あ".repeat(25)), 5);
    // Han, Hiragana, Katakana and Hangul all count.
    assert_eq!(seconds_for("", &"漢".repeat(25)), 5);
    assert_eq!(seconds_for("", &"カ".repeat(25)), 5);
    assert_eq!(seconds_for("", &"한".repeat(25)), 5);
}

/// 8: whitespace and punctuation alone count nothing.
#[test]
fn t8_whitespace_and_punctuation_only_get_the_floor() {
    assert_eq!(seconds_for(" \n\t ", "... — 、。 !? （） \u{3000}"), 4);
}

/// Latin words inside Japanese text and the title/body split both count.
#[test]
fn mixed_scripts_and_title_plus_body_are_both_counted() {
    // 25 CJK characters (10 units) next to 20 Latin words: 30 units in total -> 11 s.
    let text = format!("{} {}", "あ".repeat(25), words(20));
    assert_eq!(seconds_for("", &text), 11);
    // The same 30 units split over title and body give the same result.
    assert_eq!(seconds_for(&words(10), &words(20)), 11);
}

/// `lifetime_for` wraps the seconds as a transient lifetime.
#[test]
fn lifetime_for_is_a_transient_lifetime_of_those_seconds() {
    assert_eq!(lifetime_for("", &words(31)), ToastLifetime::seconds(12));
}
