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

/// rfd のネイティブファイルピッカーを開き、選ばれたパスを `Message::FilePicked`
/// として返す。キャンセル時は `FilePicked(None)` を返す (§5.1, §12 代替経路)。
///
/// `AsyncFileDialog::pick_file()` が返す `FileHandle` は `path()` で
/// `&Path` を取れる。`PathBuf` に複製してから iced::Task のメッセージに乗せる。
pub fn pick_file_task() -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .add_filter("Images", &["png", "svg"])
                .set_title("Choose a PNG or SVG to forge favicons from")
                .pick_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        },
        Message::FilePicked,
    )
}
