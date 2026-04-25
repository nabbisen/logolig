//! UI 層。
//!
//! 画面構成は状態 (`AppState`) の関数として定義する (§11.2)。
//! 副作用は `logolig_core::services` または `crate::task_queue` に逃がす。

pub mod accessibility;
pub mod advanced_drawer;
pub mod drop_zone;
pub mod preview_panel;
