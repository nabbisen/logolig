//! HTML スニペット生成テスト (§7.2)。
//!
//! - デフォルトプランから期待される `<link>` 群が生成される
//! - 計画から外された成果物への参照は出ない
//! - base パスの正規化 (末尾スラッシュ補完)
//! - 出力にレガシーな `msapplication-*` や `browserconfig.xml` が**含まれない**

use logolig_core::services::html_snippet::{render, DEFAULT_BASE};
use logolig_core::ExportPlan;

#[test]
fn default_plan_renders_modern_minimal_set() {
    let html = render(&ExportPlan::default(), DEFAULT_BASE);
    // ICO は最初に来る (古い後方互換性のため筆頭)
    assert!(html.contains(r#"<link rel="icon" href="/favicon.ico" sizes="any">"#));
    // PNG サイズ (デフォルト 32 / 192 / 512) がそれぞれ出る
    assert!(html.contains(r#"sizes="32x32" href="/favicon-32.png""#));
    assert!(html.contains(r#"sizes="192x192" href="/favicon-192.png""#));
    assert!(html.contains(r#"sizes="512x512" href="/favicon-512.png""#));
    // Apple touch icon
    assert!(html.contains(r#"rel="apple-touch-icon" sizes="180x180""#));
}

#[test]
fn excluded_artifacts_do_not_appear_in_html() {
    let mut plan = ExportPlan::default();
    plan.include_apple_touch = false;
    plan.include_ico = false;

    let html = render(&plan, DEFAULT_BASE);
    assert!(!html.contains("apple-touch-icon"));
    assert!(!html.contains("favicon.ico"));
    // PNG 参照は残る
    assert!(html.contains("favicon-32.png"));
}

#[test]
fn legacy_microsoft_tags_are_never_emitted() {
    // §7.2 「現代的な favicon 参照構成を反映する」
    let html = render(&ExportPlan::default(), DEFAULT_BASE);
    assert!(!html.contains("msapplication"));
    assert!(!html.contains("browserconfig"));
    assert!(!html.contains("mstile"));
    assert!(!html.contains("apple-touch-icon-precomposed"));
}

#[test]
fn base_path_normalization_appends_slash() {
    let plan = ExportPlan::default();
    let html_no_slash = render(&plan, "/static/icons");
    let html_slash = render(&plan, "/static/icons/");
    // 末尾スラッシュの有無で結果が変わってはいけない
    assert_eq!(html_no_slash, html_slash);
    assert!(html_no_slash.contains("/static/icons/favicon.ico"));
}

#[test]
fn empty_base_falls_back_to_root() {
    let plan = ExportPlan::default();
    let html = render(&plan, "");
    assert!(html.contains(r#"href="/favicon.ico""#));
}

#[test]
fn png_sizes_are_sorted_and_deduped() {
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![512, 32, 32, 192];
    let html = render(&plan, "/");
    // 出力中の出現順序は昇順
    let pos_32 = html.find("favicon-32.png").unwrap();
    let pos_192 = html.find("favicon-192.png").unwrap();
    let pos_512 = html.find("favicon-512.png").unwrap();
    assert!(pos_32 < pos_192 && pos_192 < pos_512);
    // 32 が 1 度しか登場しないことの確認 (重複除去)
    assert_eq!(html.matches("favicon-32.png").count(), 1);
}
