//! 非同期タスクのヘルパ。
//!
//! `iced::Task::perform` をラップして、UI 層から重い処理を呼ぶときの
//! クロージャを散らかさないようにする。
//!
//! このモジュールは `iced::Task` と `crate::app::Message` の両方に依存する
//! ため、logolig-core ではなく logolig-app 側に置く。

use std::path::PathBuf;

use iced::Task;

use crate::app::Message;

/// ファイル読み込みタスクを起動する。
///
/// 完了は `Message::IngestCompleted(Result<_,_>)` で UI に戻る。
pub fn ingest_task(path: PathBuf) -> Task<Message> {
    Task::perform(
        logolig_core::services::ingest::ingest(path),
        Message::IngestCompleted,
    )
}
