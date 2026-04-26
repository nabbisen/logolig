//! アクセシビリティ補助 (§12, ABDD)。
//!
//! - ラベルは集約して使い回す（スクリーンリーダー一貫性）
//! - 略語ではなく意味の通る短文にする
//! - 状態を色だけに依存させない（マーカー文字を併用）
//!
//! Step 1 では一部のラベル/マーカーは未使用 (`CHOOSE_FILE_BTN` は file picker、
//! `TOGGLE_THEME_BTN` は将来のテーマ pick_list、`ERROR`/`READY` は失敗・成功の
//! バッジ)。完成時の語彙を一望できるよう、最初から並べてある。

#[allow(dead_code)]
pub mod label {
    pub const APP_TITLE: &str = "Logolig";
    pub const DROP_ZONE: &str = "Drop a PNG or SVG image here, or activate to choose a file";
    pub const CHOOSE_FILE_BTN: &str = "Choose source image file";
    pub const TOGGLE_THEME_BTN: &str = "Toggle theme (System / Light / Dark)";
    pub const TOGGLE_ADVANCED_BTN: &str = "Show or hide advanced settings";
    pub const EXPORT_BTN: &str = "Export favicons to disk";
}

#[allow(dead_code)]
pub mod marker {
    pub const BUSY: &str = "⏳";
    pub const ERROR: &str = "⚠";
    pub const READY: &str = "✓";
}
