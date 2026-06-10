//! `<head>` 用 HTML スニペット生成 (§7.2)。
//!
//! 設計方針 (docs/export-spec.md):
//! - **セマンティック** — `rel="icon"` / `rel="apple-touch-icon"` / `type="image/png"` を適切に使い分け
//! - **モダン** — 旧 IE 用 `msapplication-*` や `browserconfig.xml` は出さない
//! - **簡潔** — 計画に含まれない成果物への参照は出さない (HTML が現実と乖離しないため)
//! - **a11y を損なわない** — `<link>` のみで他のセマンティック要素は弄らない
//! - **生成結果をそのまま貼れる** — 余計な囲みタグや HTML 全体は出さない

use crate::domain::ExportPlan;

/// 出力先 URL のルートプレフィックス。 デフォルトはサイトルート (`/`)。
pub const DEFAULT_BASE: &str = "/";

/// `ExportPlan` を反映した `<head>` 用スニペットを返す。
///
/// `base` は通常 `"/"`。 サブパスに置く場合は `"/static/favicons/"` のように指定。
/// 末尾スラッシュは内部で正規化する。
///
/// 出力順序 (v1.2.0):
/// 1. `<link rel="icon" type="image/svg+xml">` — モダンブラウザが優先選択
/// 2. `<link rel="icon" href="/favicon.ico" sizes="any">` — レガシー互換
/// 3. PNG 各サイズ (昇順)
/// 4. `<link rel="apple-touch-icon">` — iOS/Safari 用
///
/// なぜ SVG が先か: ブラウザは複数の `<link rel="icon">` から「最も適したもの」
/// を選ぶ。 SVG をサポートする現代ブラウザは SVG を選び、 高 DPI で美しく表示。
/// レガシーブラウザは SVG を無視して ICO/PNG にフォールバックする。
pub fn render(plan: &ExportPlan, base: &str) -> String {
    let base = normalize_base(base);
    let mut out = String::new();

    // SVG はモダンブラウザ向けの最優先候補。 出力順は HTML 表現の優先順位と
    // して効くため、 ICO や PNG の前に置く (§7.2 モダン構成)。
    if plan.include_svg {
        out.push_str(&format!(
            "<link rel=\"icon\" type=\"image/svg+xml\" href=\"{base}favicon.svg\">\n"
        ));
    }

    // ICO は最も古い後方互換のため二番目。
    // `sizes="any"` は ICO がスケーラブルである旨を示す現代的な書き方。
    if plan.include_ico {
        out.push_str(&format!(
            "<link rel=\"icon\" href=\"{base}favicon.ico\" sizes=\"any\">\n"
        ));
    }

    // PNG サイズはモダンブラウザ向け。 各サイズ独立の `<link>` で出す。
    let mut png_sizes = plan.png_sizes.clone();
    png_sizes.sort_unstable();
    png_sizes.dedup();
    for size in &png_sizes {
        out.push_str(&format!(
            "<link rel=\"icon\" type=\"image/png\" sizes=\"{size}x{size}\" href=\"{base}favicon-{size}.png\">\n"
        ));
    }

    // Apple touch icon は専用の rel。 `sizes` は省略可だが書いた方が親切。
    if plan.include_apple_touch {
        out.push_str(&format!(
            "<link rel=\"apple-touch-icon\" sizes=\"180x180\" href=\"{base}apple-touch-icon.png\">\n"
        ));
    }

    // v1.8.0: Web manifest が出力対象なら link 行を追加。
    // `<link rel="manifest">` は PWA 仕様の標準。 サーバ側の MIME 設定
    // (`application/manifest+json`) はユーザの責任で、 logolig は HTML への
    // 紐付けだけを担う。
    if plan.web_manifest.is_some() {
        out.push_str(&format!(
            "<link rel=\"manifest\" href=\"{base}manifest.webmanifest\">\n"
        ));
    }

    out
}

/// `base` の末尾にスラッシュを保証する。 空文字列ならデフォルトに置き換え。
fn normalize_base(base: &str) -> String {
    let trimmed = base.trim();
    if trimmed.is_empty() {
        return DEFAULT_BASE.to_string();
    }
    if trimmed.ends_with('/') {
        trimmed.to_string()
    } else {
        format!("{trimmed}/")
    }
}
