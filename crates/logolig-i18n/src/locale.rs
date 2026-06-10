//! ロケール (v1.6.0)。
//!
//! v1.5.0 で英語のみで導入し、 v1.6.0 で日本語 (`Ja`) を追加。 v1.6.0 以降の
//! ロケール追加手順は:
//!
//! 1. `Locale` 列挙に新バリアントを追加
//! 2. `Locale::all()` の配列長と要素を更新
//! 3. `as_bcp47` と `from_bcp47` の match に追加
//! 4. `crates/logolig-i18n/locales/<tag>.toml` を新設
//! 5. `dictionary::load` の match で `include_str!` を分岐
//! 6. `advanced_drawer.rs` の `locale_message_key` で `MessageKey` をマップ
//!
//! 1〜3 と 5 のいずれかを忘れるとコンパイルが通らないため、 「実装漏れ」 は
//! 構造的に防げる設計になっている。 6 と 4 はテストで検出する。

use serde::{Deserialize, Serialize};

/// サポートしているロケール。
///
/// 列挙にバリアントを足すと `Translator::for_locale` の dictionary 解決と
/// `Locale::all()` / `as_bcp47` / `from_bcp47` の match が連鎖的にコンパイル
/// エラーになる。 「ロケールを足したら全箇所追従する」 が型レベルで強制される。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    /// 日本語 (v1.6.0)。
    Ja,
}

impl Locale {
    /// pick_list 用の全列挙。
    pub fn all() -> [Self; 2] {
        [Self::En, Self::Ja]
    }

    /// IETF BCP-47 風のタグ。 `PersistedSettings.locale` の値と対応する。
    pub fn as_bcp47(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ja => "ja",
        }
    }

    /// BCP-47 タグ (もしくはそれに似た文字列) から Locale を解決する。
    /// 未知のタグや言語コード前半部分のみ ("en-US" → "en") で照合する。
    /// 解決できない場合は `None` を返す(呼び出し側でフォールバックする)。
    ///
    /// macOS の `ja_JP.UTF-8`、 Linux の `ja_JP`、 ブラウザの `ja-JP` など、
    /// 主要な揺れをすべて吸収する。
    pub fn from_bcp47(tag: &str) -> Option<Self> {
        // "en-US" / "en_US" / "en" すべてマッチさせるため、 先頭の言語タグだけ見る。
        // また `ja_JP.UTF-8` のような POSIX ロケール形式に備え、 `.` でも分割。
        let primary = tag
            .split(|c: char| c == '-' || c == '_' || c == '.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            "ja" => Some(Self::Ja),
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
