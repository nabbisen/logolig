//! `Translator` (v1.5.0).
//!
//! Primary API for the UI layer. Accepts a `MessageKey` and optional
//! placeholder arguments; returns a translated `String`.

use logolig_core::{AppError, MessageKey};

use crate::dictionary::Dictionary;
use crate::locale::Locale;

/// Central translation object.
///
/// Held as `AppState::translator`. On locale change,
/// replace with `Translator::for_locale(new_locale)`.
#[derive(Debug, Clone)]
pub struct Translator {
    locale: Locale,
    dict: Dictionary,
}

impl Translator {
    /// Construct a `Translator` for the given locale.
    pub fn for_locale(locale: Locale) -> Self {
        Self {
            locale,
            dict: Dictionary::load(locale),
        }
    }

    /// Return the current `Locale` (used by the UI to show the active language).
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Simple translation with no placeholder substitution.
    ///
    /// Returns the raw template string even if it contains `{name}` placeholders
    /// (substitution is done by `t_args`). Intended for keys with no arguments.
    pub fn t(&self, key: MessageKey) -> String {
        self.dict.lookup(key).to_string()
    }

    /// Translation with argument substitution. Replaces `{name}` in the template.
    ///
    /// All occurrences of each placeholder are replaced.
    /// Unspecified placeholders are left as-is (easier to debug).
    pub fn t_args(&self, key: MessageKey, args: &[(&str, &str)]) -> String {
        let mut s = self.dict.lookup(key).to_string();
        for (name, value) in args {
            let needle = format!("{{{name}}}");
            s = s.replace(&needle, value);
        }
        s
    }

    /// Convenience wrapper: translate an `AppError`. Calls `key()` and `args()`
    /// automatically so call sites only need `translator.translate_error(&err)`.
    pub fn translate_error(&self, err: &AppError) -> String {
        let owned = err.args();
        let refs: Vec<(&str, &str)> = owned.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.t_args(err.key(), &refs)
    }
}

impl Default for Translator {
    /// Default is English, consistent with `Locale::default()`.
    fn default() -> Self {
        Self::for_locale(Locale::default())
    }
}
