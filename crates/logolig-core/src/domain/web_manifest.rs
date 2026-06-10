//! Web manifest 設定 (v1.8.0)。
//!
//! `manifest.webmanifest` (W3C Web App Manifest) を出力するための入力データ。
//! 仕様: https://www.w3.org/TR/appmanifest/
//!
//! ## v1.8 のスコープ判断
//!
//! W3C 仕様は 30 以上のフィールドを定義しているが、 logolig は favicon
//! ジェネレータの延長として「PWA としてホーム画面に追加された時の見た目」
//! を最低限カバーする。 v1.8 で扱うのは:
//!
//! - `name` — 正式名 (ホーム画面のラベル)
//! - `short_name` — 短縮名
//! - `theme_color` — ブラウザ UI のアクセント色 (`#RRGGBB`)
//! - `background_color` — スプラッシュスクリーンの背景色 (`#RRGGBB`)
//! - `icons` — `ExportPlan.png_sizes` から自動生成
//!
//! デフォルト固定:
//!
//! - `start_url = "/"` — favicon 用途では変更が稀
//! - `display = "standalone"` — PWA インストール時の典型値
//!
//! v1.8 では UI で変更不可。 必要なら v1.8.x で追加する。
//!
//! 追加で扱わないもの:
//!
//! - `description` / `lang` / `dir` / `orientation` / `scope` — favicon
//!   ジェネレータの責務外
//! - `purpose: "maskable"` — Android のマスク対応。 別途トリミング考慮が
//!   必要なため v1.8 では `purpose: "any"` 固定
//! - `screenshots` / `shortcuts` / `related_applications` — PWA 専門ツール領域
//!
//! ## デフォルト値
//!
//! `Default` は「最小の有効な manifest」 ではなく「触らずに出力すれば
//! プレースホルダが書かれている」 状態。 ユーザがフィールドを空のまま出力
//! しないように、 各フィールドに「`My App`」 「`#FFFFFF`」 のような分かりやすい
//! プレースホルダを入れる。 永続化される段階でユーザの入力に置き換わる。

use serde::{Deserialize, Serialize};

/// Web manifest 設定。
///
/// `serde(default)` を全フィールドに付け、 v1.8 以降で新フィールドを追加した
/// 時に旧 settings.json から問題なく読めるようにする。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebManifestSettings {
    /// 正式名。 PWA インストール時のホーム画面ラベル。
    /// 空文字列だと一部のブラウザがインストール拒否するため、 空にしないこと。
    #[serde(default = "default_name")]
    pub name: String,

    /// 短縮名。 ホーム画面でスペースが狭い時のフォールバック。
    /// `name` より短く、 12 文字以下が望ましい (W3C 推奨)。
    #[serde(default = "default_short_name")]
    pub short_name: String,

    /// テーマカラー (#RRGGBB)。 ブラウザ UI のアクセント色として使われる。
    /// アドレスバー / 通知バーの背景色など。
    #[serde(default = "default_theme_color")]
    pub theme_color: String,

    /// 背景色 (#RRGGBB)。 PWA 起動時のスプラッシュスクリーン背景。
    /// アイコンが背景に溶け込まない色を選ぶ。
    #[serde(default = "default_background_color")]
    pub background_color: String,
}

fn default_name() -> String {
    "My App".to_string()
}

fn default_short_name() -> String {
    "App".to_string()
}

fn default_theme_color() -> String {
    "#FFFFFF".to_string()
}

fn default_background_color() -> String {
    "#FFFFFF".to_string()
}

impl Default for WebManifestSettings {
    fn default() -> Self {
        Self {
            name: default_name(),
            short_name: default_short_name(),
            theme_color: default_theme_color(),
            background_color: default_background_color(),
        }
    }
}

impl WebManifestSettings {
    /// 色文字列が `#RRGGBB` の 7 文字 16 進形式かを検証する。
    /// W3C 仕様は `#RGB` や名前付き色 (`red`) も受け付けるが、 logolig は
    /// 「迷わない UI」 (§5) のため `#RRGGBB` 形式に限定。
    ///
    /// 検証は UI 層 (logolig-app) でユーザ入力時に走らせる。 ここでは
    /// 純関数として提供するだけ。
    pub fn is_valid_color(s: &str) -> bool {
        if s.len() != 7 {
            return false;
        }
        let bytes = s.as_bytes();
        if bytes[0] != b'#' {
            return false;
        }
        bytes[1..].iter().all(|c| c.is_ascii_hexdigit())
    }

    /// `name` と `short_name` がそれぞれ非空かを検証する。 空だと PWA
    /// インストール時に問題があるため、 UI 層で出力前にチェックする。
    pub fn has_required_text(&self) -> bool {
        !self.name.trim().is_empty() && !self.short_name.trim().is_empty()
    }
}
