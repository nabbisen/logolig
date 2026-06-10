//! # logolig-i18n
//!
//! `logolig` の翻訳辞書とロケール解決 (v1.5.0)。
//!
//! ## 設計方針
//!
//! - **キーは `logolig_core::MessageKey` enum** — 文字列キーではなく型安全
//! - **辞書は TOML** で `include_str!` によりバイナリにバンドル
//! - **net 出力は `String`** — UI 層が iced/snora の `text()` に直接渡せる
//! - **logolig-i18n は logolig-core に片方向依存**。 これは扇形を崩すが、
//!   翻訳キーが core の語彙であるという関係を型で表現するための健全な依存
//! - **ファイル I/O も localStorage 抽象も持たない** — 辞書はビルド時に
//!   `include_str!` で取り込まれるため、 v1 ネイティブも v2 WASM も同じコード
//!   で動く
//!
//! ## API
//!
//! ```ignore
//! use logolig_core::MessageKey;
//! use logolig_i18n::{Locale, Translator};
//!
//! let t = Translator::for_locale(Locale::En);
//! assert_eq!(t.t(MessageKey::AppTitle), "Logolig");
//!
//! let path = "/tmp/foo".to_string();
//! let body = t.t_args(MessageKey::ErrorIo, &[("path", &path), ("source", "permission denied")]);
//! ```
//!
//! ## Translator placement
//!
//! v1.5 では `AppState::translator: Translator` のように **AppState 内に保持** する
//! 設計を採用した。 グローバル static を避ける理由:
//!
//! - iced の関数型 view と整合
//! - Locale 切替が `state.translator = Translator::for_locale(new_locale)` の
//!   1 行で済み、 再描画 1 回で UI 全体が新言語になる
//! - `RwLock` などの同期コストが不要
//!
//! v2 (leptos) でも `Signal<Translator>` のような形で同じ流儀を踏襲できる。

mod dictionary;
mod locale;
mod translator;

pub use locale::{detect_system_locale, Locale};
pub use translator::Translator;
