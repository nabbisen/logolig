//! Toast lifetime proportional to the toast's content (RFC 029).
//!
//! A toast that disappears before it can be read is a message that did not arrive. Every
//! transient toast therefore stays for
//!
//! ```text
//! seconds = clamp(ceil(2 + 0.3 * units), 4, 20)
//! ```
//!
//! where `units` is the reading size of the displayed title plus body:
//!
//! * each whitespace-separated token of non-CJK text counts `max(1, ceil(chars / 6))`, so a long
//!   file path counts as several units, not one;
//! * each CJK character counts `0.4` (Japanese is read at about 500 characters per minute
//!   against about 200 words per minute for English);
//! * punctuation and whitespace count nothing; a token without any letter or digit counts nothing.
//!
//! `0.3 s` per unit is 200 words per minute, `2 s` is the time to notice the toast and start
//! reading, `4 s` is snora's default (the floor) and `20 s` is the cap.
//!
//! The arithmetic is done in **tenths of a unit** with integers, so no floating-point rounding
//! can move a result by a second at an exact boundary (30 units are exactly 11 s).

use snora::ToastLifetime;

/// Shortest lifetime: snora's own default.
const MIN_SECONDS: u64 = 4;
/// Longest lifetime.
const MAX_SECONDS: u64 = 20;
/// Time to notice a toast and start reading, in seconds.
const NOTICE_SECONDS: u64 = 2;
/// One unit of a non-CJK token is six characters; counted in tenths of a unit.
const TOKEN_CHARS_PER_UNIT: usize = 6;
const TENTHS_PER_UNIT: u64 = 10;
/// A CJK character is 0.4 unit, i.e. four tenths.
const CJK_CHAR_TENTHS: u64 = 4;

/// Whole seconds a transient toast with this title and body stays on screen.
pub(crate) fn seconds_for(title: &str, body: &str) -> u64 {
    let tenths = reading_tenths(title) + reading_tenths(body);
    // seconds = ceil(2 + 0.3 * units) with units = tenths / 10, i.e. 2 + ceil(3 * tenths / 100).
    let reading = (3 * tenths).div_ceil(100);
    (NOTICE_SECONDS + reading).clamp(MIN_SECONDS, MAX_SECONDS)
}

/// The lifetime to give a transient toast with this title and body.
pub(crate) fn lifetime_for(title: &str, body: &str) -> ToastLifetime {
    ToastLifetime::seconds(seconds_for(title, body))
}

/// Reading size of one text, in tenths of a unit.
fn reading_tenths(text: &str) -> u64 {
    let mut tenths: u64 = 0;
    // The token being read: its length in characters and whether it holds a letter or digit.
    let mut len: usize = 0;
    let mut has_alphanumeric = false;

    for c in text.chars() {
        if is_cjk(c) {
            tenths += CJK_CHAR_TENTHS;
            tenths += flush_token(&mut len, &mut has_alphanumeric);
        } else if c.is_whitespace() {
            tenths += flush_token(&mut len, &mut has_alphanumeric);
        } else {
            len += 1;
            has_alphanumeric |= c.is_alphanumeric();
        }
    }
    tenths + flush_token(&mut len, &mut has_alphanumeric)
}

/// Close the current token and return its weight in tenths (0 for a token without letters or digits).
fn flush_token(len: &mut usize, has_alphanumeric: &mut bool) -> u64 {
    let weight = if *has_alphanumeric {
        let units = len.div_ceil(TOKEN_CHARS_PER_UNIT).max(1);
        u64::try_from(units).unwrap_or(u64::MAX / TENTHS_PER_UNIT) * TENTHS_PER_UNIT
    } else {
        0
    };
    *len = 0;
    *has_alphanumeric = false;
    weight
}

/// A letter of a script that is read per character: Han, Hiragana, Katakana or Hangul.
///
/// `char::is_alphabetic` excludes punctuation (for example `、。` and full-width brackets), which
/// therefore counts nothing; the Unicode blocks below pick the scripts:
///
/// * `U+3400..=U+4DBF`, `U+4E00..=U+9FFF`, `U+F900..=U+FAFF`, `U+20000..=U+2FA1F`: Han;
/// * `U+3040..=U+309F`: Hiragana;
/// * `U+30A0..=U+30FF`, `U+31F0..=U+31FF`, `U+FF66..=U+FF9F`: Katakana (including the long-vowel
///   mark and half-width forms);
/// * `U+1100..=U+11FF`, `U+3130..=U+318F`, `U+AC00..=U+D7AF`: Hangul.
fn is_cjk(c: char) -> bool {
    c.is_alphabetic()
        && matches!(
            u32::from(c),
            0x3400..=0x4DBF
                | 0x4E00..=0x9FFF
                | 0xF900..=0xFAFF
                | 0x2_0000..=0x2_FA1F
                | 0x3040..=0x309F
                | 0x30A0..=0x30FF
                | 0x31F0..=0x31FF
                | 0xFF66..=0xFF9F
                | 0x1100..=0x11FF
                | 0x3130..=0x318F
                | 0xAC00..=0xD7AF
        )
}

#[cfg(test)]
mod tests;
