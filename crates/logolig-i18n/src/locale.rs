//! ロケール (v1.5.0)。
//!
//! v1.5.0 は英語のみサポート。 将来 (v1.6+) ja を追加する際は `Locale::Ja`
//! バリアントを足し、 `dictionary::load` の match に `ja.toml` を追加する。

use serde::{Deserialize, Serialize};

/// サポートしているロケール。
///
/// 将来 (v1.6+) で日本語追加時には `Ja` バリアントを足す。 そのとき
/// `Translator` の網羅性 match と `dictionary::load` がコンパイルエラーになる
/// ので、 翻訳辞書追加を漏らさず実装できる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    // 将来追加予定:
    // Ja,
}

impl Locale {
    /// pick_list 用の全列挙。
    pub fn all() -> [Self; 1] {
        [Self::En]
    }

    /// IETF BCP-47 風のタグ。 `PersistedSettings.locale` の値と対応する。
    pub fn as_bcp47(self) -> &'static str {
        match self {
            Self::En => "en",
        }
    }

    /// BCP-47 タグ (もしくはそれに似た文字列) から Locale を解決する。
    /// 未知のタグや言語コード前半部分のみ ("en-US" → "en") で照合する。
    /// 解決できない場合は `None` を返す(呼び出し側でフォールバックする)。
    pub fn from_bcp47(tag: &str) -> Option<Self> {
        // "en-US" / "en_US" / "en" すべてマッチさせるため、 先頭の言語タグだけ見る
        let primary = tag
            .split(|c: char| c == '-' || c == '_')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            // "ja" => Some(Self::Ja),  // v1.6 で追加
            _ => None,
        }
    }
}

/// OS のロケールを検出して `Locale` に解決する。
///
/// `sys-locale::get_locale()` は LANG / macOS API / Windows API を吸収する。
/// 検出失敗または未サポートロケールの場合は `Locale::default()` (英語) を返す。
///
/// v2 ブラウザ環境では `sys-locale` が `navigator.language` を返すため、
/// 同じコードでブラウザでも動く。
pub fn detect_system_locale() -> Locale {
    sys_locale::get_locale()
        .as_deref()
        .and_then(Locale::from_bcp47)
        .unwrap_or_default()
}
