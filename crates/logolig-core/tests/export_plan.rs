//! ドメイン型の既定値検証。
//!
//! 仕様書 §7.1, §6.2 で「既定値」が明示されている部分の回帰防止。

use logolig_core::{ExportPlan, ResizeAlgorithm, ThemeMode};

#[test]
fn export_plan_default_is_minimal_modern_set() {
    // §7.1: favicon.ico, apple-touch-icon.png, 高解像度 PNG, HTML スニペット
    // v1.2.0: favicon.svg を default に追加 (SVG ソースは raw コピー、 ラスタは vectorize)
    // v1.4.1: vtracer_preset = Default を追加 (v1.2.0 互換)
    let plan = ExportPlan::default();
    assert!(plan.include_ico);
    assert!(plan.include_apple_touch);
    assert!(plan.include_html_snippet);
    assert!(plan.include_svg);
    assert!(plan.vectorize_on_raster);
    assert_eq!(plan.vtracer_preset, logolig_core::VtracerPreset::Default);
    // モダン構成の最小: 32 / 192 / 512 (browser, PWA, hi-res)
    assert_eq!(plan.png_sizes, vec![32, 192, 512]);
    // ICO は古い環境向けに 16/32/48 を内包
    assert_eq!(plan.ico_sizes, vec![16, 32, 48]);
}

#[test]
fn resize_default_is_lanczos3() {
    // §6.2「既定値は品質重視にする」
    assert_eq!(ResizeAlgorithm::default(), ResizeAlgorithm::Lanczos3);
}

#[test]
fn theme_mode_cycles() {
    // テーマトグル: System -> Light -> Dark -> System
    assert_eq!(ThemeMode::System.next(), ThemeMode::Light);
    assert_eq!(ThemeMode::Light.next(), ThemeMode::Dark);
    assert_eq!(ThemeMode::Dark.next(), ThemeMode::System);
}

#[test]
fn artifact_count_matches_default_plan() {
    // v1.2.0: svg(1) + ico(1) + apple(1) + html(1) + png_sizes(3) = 7
    assert_eq!(ExportPlan::default().artifact_count(), 7);
}

// ---------------------------------------------------------------------------
// v1.3.0: サイズ集合の編集 API
// ---------------------------------------------------------------------------

#[test]
fn add_png_size_keeps_set_sorted_and_unique() {
    let mut plan = ExportPlan::default();
    // 初期: [32, 192, 512]
    assert!(plan.add_png_size(64));
    assert_eq!(plan.png_sizes, vec![32, 64, 192, 512]);
    // 重複は加わらない
    assert!(!plan.add_png_size(64));
    assert_eq!(plan.png_sizes, vec![32, 64, 192, 512]);
    // 先頭・末尾の挿入位置が正しいか
    assert!(plan.add_png_size(16));
    assert!(plan.add_png_size(1024));
    assert_eq!(plan.png_sizes, vec![16, 32, 64, 192, 512, 1024]);
}

#[test]
fn add_png_size_rejects_out_of_range() {
    let mut plan = ExportPlan::default();
    let baseline = plan.png_sizes.clone();
    // 下限未満
    assert!(!plan.add_png_size(15));
    assert!(!plan.add_png_size(0));
    // 上限超え
    assert!(!plan.add_png_size(1025));
    assert!(!plan.add_png_size(99999));
    assert_eq!(plan.png_sizes, baseline);
}

#[test]
fn remove_png_size_returns_false_for_missing() {
    let mut plan = ExportPlan::default();
    assert!(plan.remove_png_size(192));
    assert_eq!(plan.png_sizes, vec![32, 512]);
    // 存在しないサイズの削除は false
    assert!(!plan.remove_png_size(192));
    assert!(!plan.remove_png_size(7));
}

#[test]
fn add_ico_size_respects_256_upper_limit() {
    let mut plan = ExportPlan::default();
    // ICO 仕様上 256 まで
    assert!(plan.add_ico_size(256));
    assert_eq!(plan.ico_sizes, vec![16, 32, 48, 256]);
    // 257 以上は弾く
    assert!(!plan.add_ico_size(257));
    assert!(!plan.add_ico_size(512));
}

#[test]
fn ico_size_set_can_become_empty() {
    // ico_sizes が空でも構造的には許容する (ユーザは include_ico=false で運用)。
    let mut plan = ExportPlan::default();
    for size in [16, 32, 48] {
        plan.remove_ico_size(size);
    }
    assert!(plan.ico_sizes.is_empty());
}
