//! 永続化される設定 (v1.4.0)。
//!
//! 起動時に `SettingsStore::load_or_default()` でロードし、 ユーザ操作で
//! 値が変わるたびに `SettingsStore::update()` で即時保存する。
//!
//! ## 含めるもの
//!
//! - `export_plan`: 出力プラン全体 (PNG/ICO サイズ群、 アルゴリズム、 各種チェックボックス)
//! - `theme`:       テーマモード (System / Light / Dark の選択)
//! - `locale`:      v1.5 の i18n 基盤への伏線。 `Some("ja")` のように OS ロケール
//!                  を上書きできる。 v1.4 時点では未使用フィールドだが、 後続
//!                  バージョンで読み出される
//!
//! ## 含めないもの
//!
//! - `source_path` / `source_asset` — セッション固有 (前回開いたファイルを次回も
//!   開きたいかは別問題)
//! - `screen` / `busy` — UI 状態であって設定ではない
//! - `preview_cache` — 計算結果なので保存する意味がない
//! - `advanced_open` — 開閉状態は保存対象として揺れがある (一部ユーザは
//!   常時開きたい、 一部は常時閉じたい)。 v1.4 では含めず、 必要が見えてから
//!   追加する
//!
//! ## 前方互換性 (`#[serde(default)]`)
//!
//! 全フィールドに serde の default を適用してある。 古い JSON が新しいフィールド
//! を持たない場合に `T::default()` で埋まる。 これにより、 後でフィールドを増やす
//! 変更が破壊的でなくなる。
//!
//! ## なぜ単一構造体か
//!
//! セクションごと (export_plan / theme / locale) に分割した複数キーで保存する
//! 案もあったが、 単一構造体にして単一ファイル / 単一 LocalStorage キーに
//! 入れる方針を採った:
//!
//! - `app-json-settings` v2 の `ConfigManager<T>` API がそもそも単一型前提
//! - `update()` の atomicity が自然 (ファイル全体 / キー全体を一度に更新)
//! - データ量が小さい (実測 1KB 未満)
//! - 将来分割が必要になったら、 `serde(flatten)` で内部を分割するなど後付け可能

use serde::{Deserialize, Serialize};

use crate::domain::{ExportPlan, ThemeMode};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    /// 出力プラン (§7)。 PNG/ICO サイズ群やアルゴリズム選択を含む。
    pub export_plan: ExportPlan,
    /// テーマモード。
    pub theme: ThemeMode,
    /// ロケール上書き (v1.5 の i18n で使う伏線)。
    /// `None` のとき OS 検出値を使う。
    /// `Some("en")` / `Some("ja")` のように IETF BCP-47 風の文字列。
    pub locale: Option<String>,
}
