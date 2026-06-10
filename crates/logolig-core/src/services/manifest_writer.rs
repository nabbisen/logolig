//! Web manifest 生成 (v1.8.0)。
//!
//! `WebManifestSettings` + `ExportPlan.png_sizes` から
//! `manifest.webmanifest` の JSON 文字列を組み立てる。
//!
//! ## なぜサービス層か
//!
//! 文字列生成自体は純関数で済むので domain 寄りに見えるが、 出力ファイル名
//! (`manifest.webmanifest`) と icon 配列に並べる PNG ファイル名のルールが
//! `exporter` のファイル命名規約と結合している。 「icon 配列に書くファイル名」
//! と「実際に書き出される PNG ファイル名」 は同じ規約に従う必要があるため、
//! services 層に置いて将来 exporter と命名ロジックを共有しやすくする。
//!
//! ## 出力 JSON のフィールド
//!
//! v1.8 で出力するのは以下の固定構造:
//!
//! ```json
//! {
//!   "name": "...",
//!   "short_name": "...",
//!   "icons": [
//!     {"src": "favicon-32.png", "sizes": "32x32", "type": "image/png", "purpose": "any"},
//!     ...
//!   ],
//!   "start_url": "/",
//!   "display": "standalone",
//!   "theme_color": "#...",
//!   "background_color": "#..."
//! }
//! ```
//!
//! `start_url` と `display` は v1.8 では UI から変更できない固定値 (前述)。
//!
//! ## ファイル名命名
//!
//! PNG サイズは `favicon-{size}.png` 形式 (例: `favicon-32.png`)。 これは
//! `exporter` の命名規約と一致させること — UI の HTML スニペットも同じ規約
//! を前提にしている。 v1.8 で命名を変えるならここと exporter / html_snippet
//! の 3 箇所同時に修正が必要。

use serde_json::json;

use crate::domain::WebManifestSettings;

/// 出力するファイル名 (拡張子含む)。 W3C 推奨は `.webmanifest` 拡張子だが、
/// `.json` でも各ブラウザは受け付ける。 logolig は推奨側を採用。
pub const MANIFEST_FILENAME: &str = "manifest.webmanifest";

/// `WebManifestSettings` と PNG サイズ集合から `manifest.webmanifest` の
/// JSON 文字列を生成する。 改行・インデント込みの読める形 (pretty-print)。
///
/// `png_sizes` が空の場合、 `icons` 配列は空のまま出力される (PWA としては
/// 推奨されない構成だが、 v1.8 ではユーザの選択を尊重する)。
pub fn build_manifest_json(settings: &WebManifestSettings, png_sizes: &[u32]) -> String {
    // icons 配列を組み立てる。 manifest 内のファイル名は exporter の出力と
    // 一致する必要があるため、 命名規約 `favicon-{size}.png` をここに直書き。
    let icons: Vec<serde_json::Value> = png_sizes
        .iter()
        .map(|size| {
            json!({
                "src": format!("favicon-{size}.png"),
                "sizes": format!("{size}x{size}"),
                "type": "image/png",
                "purpose": "any"
            })
        })
        .collect();

    let manifest = json!({
        "name": settings.name,
        "short_name": settings.short_name,
        "icons": icons,
        "start_url": "/",
        "display": "standalone",
        "theme_color": settings.theme_color,
        "background_color": settings.background_color
    });

    // pretty-print。 末尾改行も足す (POSIX text file の慣習)。
    let mut out = serde_json::to_string_pretty(&manifest)
        .expect("serde_json: serializing well-formed JSON should not fail");
    out.push('\n');
    out
}
