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
pub fn render(plan: &ExportPlan, base: &str) -> String {
    let base = normalize_base(base);
    let mut out = String::new();

    // ICO は最も古い後方互換のため筆頭に置く。
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
