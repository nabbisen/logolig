//! `Translator` (v1.5.0)。
//!
//! UI 層が呼び出す主要 API。 `MessageKey` と必要なら placeholder 引数を
//! 受け取り、 翻訳済み `String` を返す。

use logolig_core::{AppError, MessageKey};

use crate::dictionary::Dictionary;
use crate::locale::Locale;

/// 翻訳機能の中心。
///
/// `AppState::translator` として保持される。 ロケール変更時は
/// `state.translator = Translator::for_locale(new_locale)` で入れ替える。
#[derive(Debug, Clone)]
pub struct Translator {
    locale: Locale,
    dict: Dictionary,
}

impl Translator {
    /// 指定ロケール用の Translator を構築。
    pub fn for_locale(locale: Locale) -> Self {
        Self {
            locale,
            dict: Dictionary::load(locale),
        }
    }

    /// 現在の Locale を返す (UI で「現在何語か」 を表示する用途)。
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// プレースホルダなしの単純翻訳。
    ///
    /// テンプレートに `{name}` プレースホルダが含まれていてもそのまま返す
    /// (置換は `t_args` で行う)。 引数なしのキーで呼ぶことを想定。
    pub fn t(&self, key: MessageKey) -> String {
        self.dict.lookup(key).to_string()
    }

    /// 引数付きの翻訳。 テンプレート内の `{name}` を args に従って置換する。
    ///
    /// args は順序を持つので、 同じプレースホルダが複数回現れても全て置換
    /// される。 未指定のプレースホルダは原文のまま残る (デバッグしやすい挙動)。
    pub fn t_args(&self, key: MessageKey, args: &[(&str, &str)]) -> String {
        let mut s = self.dict.lookup(key).to_string();
        for (name, value) in args {
            let needle = format!("{{{name}}}");
            s = s.replace(&needle, value);
        }
        s
    }

    /// `AppError` を翻訳済みエラー文言にする糖衣。 `key()` と `args()` を
    /// 自動で展開するので、 呼び出し側は `translator.translate_error(&err)` で済む。
    pub fn translate_error(&self, err: &AppError) -> String {
        let owned = err.args();
        let refs: Vec<(&str, &str)> = owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.t_args(err.key(), &refs)
    }
}

impl Default for Translator {
    /// デフォルトは英語。 `Locale::default()` と整合する。
    fn default() -> Self {
        Self::for_locale(Locale::default())
    }
}
