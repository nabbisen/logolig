//! ファイル読み込み (ingest)。
//!
//! 受け入れる形式は PNG / SVG のみ (§6.1)。
//!
//! Step 2 で次を実装する:
//! - 拡張子・先頭バイトによる種別判定
//! - 非同期な `tokio::fs` 経由のバイト読込
//! - PNG はデコードヘッダで論理サイズを推定
//! - SVG は viewBox から論理サイズを推定
//! - 失敗時は `AppError` に正規化
//!
//! Step 1 段階では型シグネチャのみを公開し、未実装エラーを返す。
//! UI スレッドをブロックしないため `iced::Task::perform` から呼ばれる
//! 想定で `async fn` にしてある (§2.4)。

use std::path::PathBuf;

use crate::domain::SourceAsset;
use crate::error::AppError;

pub async fn ingest(_path: PathBuf) -> Result<SourceAsset, AppError> {
    Err(AppError::NotImplemented("logolig_core::services::ingest"))
}
