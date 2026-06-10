//! 非同期タスクのヘルパ。
//!
//! `iced::Task::perform` をラップして、UI 層から重い処理を呼ぶときの
//! クロージャを散らかさないようにする。
//!
//! このモジュールは `iced::Task` と `crate::app::Message` の両方に依存する
//! ため、logolig-core ではなく logolig-app 側に置く。

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use logolig_core::{ExportPlan, ResizeAlgorithm, SourceAsset};

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
                .add_filter("Images", &["png", "svg", "webp"])
                .set_title("Choose a PNG, SVG, or WebP to forge favicons from")
                .pick_file()
                .await
                .map(|handle| handle.path().to_path_buf())
        },
        Message::FilePicked,
    )
}

/// プレビュー画像 (16×16 と 120×120) を生成するタスク。
/// CPU バウンドな画像処理なので、 `iced::Task::perform` 経由で UI スレッドから逃がす。
///
/// `SourceAsset` を `Arc` に包むのは、 タスクへ move する際に `raw: Arc<[u8]>`
/// 周りのコピーをさらに減らすため。
pub fn build_preview_task(asset: Arc<SourceAsset>, algorithm: ResizeAlgorithm) -> Task<Message> {
    Task::perform(
        async move {
            // build_preview は同期関数なので spawn_blocking で別スレッドへ。
            // tokio の rt-multi-thread は引いていないので current_thread::spawn_blocking
            // ではなく素直にこのタスク内で計算する。プレビューは 16×16 と 120×120 のみ
            // でミリ秒オーダーなので UI スレッド的にも許容範囲。
            logolig_core::services::preview::build_preview(&asset, algorithm)
        },
        Message::PreviewBuilt,
    )
}

/// 書き出し先ディレクトリを選ぶダイアログを開く (§7)。
/// 結果は `Message::ExportDirPicked(Option<PathBuf>)` として返る。
pub fn pick_export_dir_task() -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Choose where to write the favicons")
                .pick_folder()
                .await
                .map(|handle| handle.path().to_path_buf())
        },
        Message::ExportDirPicked,
    )
}

/// 実際のエクスポートを走らせるタスク。
/// `exporter::run` は同期 + 数十ミリ秒〜数百ミリ秒程度の CPU/IO 仕事なので、
/// async ブロック内で実行して UI スレッドから逃がす (§2.4)。
pub fn export_task(
    asset: Arc<SourceAsset>,
    plan: ExportPlan,
    output_dir: PathBuf,
) -> Task<Message> {
    Task::perform(
        async move { logolig_core::services::exporter::run(&asset, &plan, &output_dir) },
        Message::ExportCompleted,
    )
}
