//! `AppState::default()` が仕様通りであることを確認する。
//!
//! `update` 関数の振る舞い検証は Step 2 以降で実装する。

// logolig-app は bin クレートのため、tests/ から直接 src/app.rs の
// 内部型 (AppState, Screen) を見ることはできない。
//
// したがって Step 1 では logolig-core 側で再現できる初期値の確認に留め、
// app 側の状態テストは Step 2 で必要な型を `pub` にしてから書く。

use logolig_core::{ExportPlan, ResizeAlgorithm, ThemeMode};

#[test]
fn defaults_match_spec() {
    // AppState::default() が依存する各コンポーネントの既定値が、
    // 仕様 §4.2 / §5.3 / §6.2 / §7.1 に整合していること。
    assert_eq!(ThemeMode::default(), ThemeMode::System);
    assert_eq!(ResizeAlgorithm::default(), ResizeAlgorithm::Lanczos3);

    let plan = ExportPlan::default();
    assert!(plan.include_ico);
    assert!(plan.include_apple_touch);
    assert!(plan.include_html_snippet);
    assert!(plan.overrides.is_empty());
}
