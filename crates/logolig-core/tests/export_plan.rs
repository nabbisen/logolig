//! ドメイン型の既定値検証。
//!
//! 仕様書 §7.1, §6.2 で「既定値」が明示されている部分の回帰防止。

use logolig_core::{ExportPlan, ResizeAlgorithm, ThemeMode};

#[test]
fn export_plan_default_is_minimal_modern_set() {
    // §7.1: favicon.ico, apple-touch-icon.png, 高解像度 PNG, HTML スニペット
    let plan = ExportPlan::default();
    assert!(plan.include_ico);
    assert!(plan.include_apple_touch);
    assert!(plan.include_html_snippet);
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
    // ico(1) + apple(1) + html(1) + png_sizes(3) = 6
    assert_eq!(ExportPlan::default().artifact_count(), 6);
}
