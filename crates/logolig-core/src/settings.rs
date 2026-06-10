//! 設定の保存抽象 (v1.4.0)。
//!
//! `SettingsStore<T>` は「型 T の値を 1 つ永続化する」 ための trait。
//! API シェイプは `app-json-settings` v2 の `ConfigManager<T>` に意図的に
//! 揃えてある:
//!
//! - `load_or_default()`: 起動時の標準ロード。 ファイル/ストレージが空なら
//!   `T::default()` を保存して返す
//! - `save()`:            全体置換書き込み
//! - `update()`:          安全な read-modify-write
//!
//! ## なぜ trait なのか
//!
//! v1 (ネイティブ iced) は OS の標準 config dir に JSON ファイルとして
//! 保存する。 v2 (将来の WASM ブラウザ) は LocalStorage に JSON 文字列
//! として保存する。 保存メディアは違うが API シェイプは完全に同じにできる。
//!
//! `logolig-core` がこの trait を所有することで、 永続化方式を 1 か所に
//! 抽象化できる。 v2 への移行時に「実装差し替えだけ」 で済む。
//!
//! ## 依存方向
//!
//! ```text
//!         logolig-app          (v1)
//!          ↓
//!   ┌────────────┐
//!   │ logolig-   │  ←── ここで trait 定義
//!   │  core      │      (serde には依存するが ファイル I/O には依存しない)
//!   └────────────┘
//!          ↑
//!         logolig-web          (v2、将来)
//! ```
//!
//! `logolig-core` は **ファイル I/O も localStorage 抽象も持たない**。
//! 実装は呼び出し側 (logolig-app または logolig-web) が提供する。

use std::error::Error as StdError;

use serde::{de::DeserializeOwned, Serialize};

/// 型 T の値を 1 つ永続化するためのストア。
///
/// `T` は `Serialize + DeserializeOwned + Default` を要求する。 default 制約は
/// `load_or_default()` が「初回起動でファイルが無い」 ケースをきれいに扱う
/// ためのもの。 logolig での T は `PersistedSettings`。
///
/// # 実装上のメモ
///
/// - `Error` は実装ごとに異なる(ファイル I/O エラー / serde エラー /
///   localStorage アクセス権 など)。 trait レベルでは `StdError + Send + Sync`
///   を要求するに留め、 具体的な型は実装側に委ねる
/// - `update()` は `load_or_default()` → クロージャ適用 → `save()` の
///   合成。 並行更新の atomicity は保証しない (logolig は単一プロセスからの
///   逐次更新しかしないため不要)
pub trait SettingsStore<T>
where
    T: Serialize + DeserializeOwned + Default,
{
    /// 実装固有のエラー型。
    type Error: StdError + Send + Sync + 'static;

    /// 既存値をロード。 存在しなければ default を保存して返す。
    fn load_or_default(&self) -> Result<T, Self::Error>;

    /// 全体置換書き込み。
    fn save(&self, config: &T) -> Result<(), Self::Error>;

    /// 読み込み → クロージャでミューテーション → 書き込み。
    /// 戻り値は更新後の T。 UI 側で「保存に成功した最新値」 を再取得する用途。
    fn update<F>(&self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut T);
}
