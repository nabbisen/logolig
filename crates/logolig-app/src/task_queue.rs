//! 非同期タスクのヘルパ。
//!
//! `iced::Task::perform` をラップして、 UI 層から重い処理を呼ぶときの
//! クロージャを散らかさないようにする。
//!
//! このモジュールは `iced::Task` と `crate::app::Message` の両方に依存する
//! ため、 logolig-core ではなく logolig-app 側に置く。

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use logolig_core::{AppError, ExportPlan, InMemoryArtifact, ResizeAlgorithm, Rgba8, SourceAsset};

use crate::app::Message;
use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

/// ファイル読み込みタスクを起動する。
///
/// 完了は `Message::IngestCompleted(Result<_,_>)` で UI に戻る。
pub fn ingest_task(path: PathBuf) -> Task<Message> {
    Task::perform(
        logolig_core::services::ingest::ingest(path),
        Message::IngestCompleted,
    )
}

/// rfd のネイティブファイルピッカーを開き、 選ばれたパスを `Message::FilePicked`
/// として返す。 キャンセル時は `FilePicked(None)` を返す (§5.1, §12 代替経路)。
///
/// `AsyncFileDialog::pick_file()` が返す `FileHandle` は `path()` で
/// `&Path` を取れる。 `PathBuf` に複製してから iced::Task のメッセージに乗せる。
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

/// プレビュー画像 (16×16 と 120×120) を生成するタスク。 CPU バウンドな画像
/// 処理なので、 `iced::Task::perform` 経由で UI スレッドから逃がす。
///
/// `SourceAsset` を `Arc` に包むのは、 タスクへ move する際に `raw: Arc<[u8]>`
/// 周りのコピーをさらに減らすため。
pub fn build_preview_task(asset: Arc<SourceAsset>, algorithm: ResizeAlgorithm) -> Task<Message> {
    Task::perform(
        async move { logolig_core::services::preview::build_preview(&asset, algorithm) },
        Message::PreviewBuilt,
    )
}

// ---------------------------------------------------------------------------
// v1.16.0 / v1.19.0: メモリ完結変換タスク
// ---------------------------------------------------------------------------

/// ファイル投入後の自動変換タスク (v1.16.0 で導入、 v1.19.0 で簡素化)。
///
/// 変換結果はメモリに保持し (`ResultAssets`)、 ユーザが個別 DL or ZIP DL を
/// 押したときに初めて書き出す。 UI 層は `Message::ConvertCompleted` で
/// 結果を受け取って Result 画面に遷移する。
///
/// ## v1.19.0 変更
///
/// 旧実装 (`run_convert_in_memory` v1.16) は OS の一時ディレクトリに
/// `exporter::run` を走らせて結果を読み戻すラッパだったが、 v1.19.0 で
/// `exporter::run_in_memory` 直接 API が logolig-core に追加されたため、
/// 一時ディレクトリ経由のステップを撤去し、 直接呼び出しに簡素化。
/// 副次効果:
/// - ディスク I/O 一切なし → ブラウザ移行 (= file system API なしで動く)
///   が視野に入る
/// - 一時ディレクトリの後始末漏れ (途中で panic 等) リスクゼロ
/// - パフォーマンス: ディスク I/O 削減 (favicon 一式は 1 MB 未満なので
///   メモリ完結が自然)
///
/// 関数名も旧 `convert_in_memory_task` から **`convert_task`** に rename
/// (v1.19 では「変換 = メモリ完結」 が前提なので "in_memory" 修飾子は冗長)。
pub fn convert_task(asset: Arc<SourceAsset>, plan: ExportPlan) -> Task<Message> {
    Task::perform(
        async move { run_convert(&asset, &plan) },
        Message::ConvertCompleted,
    )
}

fn run_convert(asset: &SourceAsset, plan: &ExportPlan) -> Result<ResultAssets, AppError> {
    // 全成果物をメモリ上で組み立てる。
    let in_memory = logolig_core::services::exporter::run_in_memory(asset, plan)?;
    // ResultAssets (UI カードレンダリング用) に変換。
    Ok(collect_assets(in_memory))
}

/// `Vec<InMemoryArtifact>` を `ResultAssets` (UI カード表示用) に変換する。
///
/// 各 artifact について:
/// 1. 種別判定 (PngMono / Png / Ico / Svg / HtmlSnippet / WebManifest)
/// 2. 寸法取得 (PNG IHDR / ICO ヘッダから)
/// 3. サムネ生成 (PNG / ICO のみ、 image crate でデコード)
fn collect_assets(in_memory: Vec<InMemoryArtifact>) -> ResultAssets {
    let mut items = Vec::with_capacity(in_memory.len());
    for art in in_memory {
        let file_name = art
            .relative_path
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| "(unknown)".to_string());
        let kind = classify_asset(&file_name, &art.relative_path);
        let dimensions = derive_dimensions(kind, &art.bytes);
        let thumbnail = build_thumbnail(kind, &art.bytes);
        items.push(ResultAssetItem {
            file_name,
            bytes: art.bytes,
            kind,
            dimensions,
            thumbnail,
        });
    }
    ResultAssets { items }
}

/// ファイル名 (+ 相対パス) からアセット種別を判定。
fn classify_asset(file_name: &str, relative_path: &std::path::Path) -> ResultAssetKind {
    let name = file_name.to_ascii_lowercase();
    // 相対パスの parent が `mono` なら mono ファイルとして扱う。
    let is_mono = relative_path
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n == "mono")
        .unwrap_or(false);
    if name.ends_with(".ico") {
        ResultAssetKind::Ico
    } else if name.ends_with(".svg") {
        ResultAssetKind::Svg
    } else if name.ends_with(".png") {
        if is_mono {
            ResultAssetKind::PngMono
        } else {
            ResultAssetKind::Png
        }
    } else if name.ends_with(".html") {
        ResultAssetKind::HtmlSnippet
    } else if name.ends_with(".webmanifest") || name.ends_with(".json") {
        ResultAssetKind::WebManifest
    } else {
        // 想定外形式。 型としては HTML スニペット相当の「テキスト系」 に倒して
        // おく (DL は問題なくできる)。
        ResultAssetKind::HtmlSnippet
    }
}

/// PNG / ICO なら寸法をパース。 失敗したら None。
fn derive_dimensions(kind: ResultAssetKind, bytes: &[u8]) -> Option<(u32, u32)> {
    match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => parse_png_size(bytes),
        ResultAssetKind::Ico => {
            // ICO ヘッダ: 6 byte 署名 + 各 entry 16 byte。 1 個目の entry の
            // 幅/高さは 4 byte 目と 5 byte 目に格納 (0 は 256 を意味)。
            if bytes.len() >= 8 {
                let w = match bytes[6] {
                    0 => 256,
                    n => n as u32,
                };
                let h = match bytes[7] {
                    0 => 256,
                    n => n as u32,
                };
                Some((w, h))
            } else {
                None
            }
        }
        // SVG は viewBox で表現されており事実上「サイズ可変」 なので None。
        ResultAssetKind::Svg => None,
        _ => None,
    }
}

/// PNG IHDR から幅高さを取得 (8 byte signature + IHDR 13 byte payload)。
fn parse_png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 24 {
        return None;
    }
    if &bytes[0..8] != [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
        return None;
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    Some((w, h))
}

/// 画像系アセットならカード表示用に decode しておく。 失敗したら None
/// (テキスト系扱いになり、 アイコンプレースホルダで表示される)。
fn build_thumbnail(kind: ResultAssetKind, bytes: &[u8]) -> Option<Rgba8> {
    if !kind.has_visual_thumbnail() {
        return None;
    }
    let format = match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => image::ImageFormat::Png,
        ResultAssetKind::Ico => image::ImageFormat::Ico,
        // SVG はサムネ生成を行わず、 バッジ表示のみ。
        _ => return None,
    };
    let img = image::load_from_memory_with_format(bytes, format).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: std::sync::Arc<[u8]> = rgba.into_raw().into();
    Rgba8::try_from_raw(w, h, pixels)
}

// ---------------------------------------------------------------------------
// v1.16.0: DL ファイル保存ダイアログ + 書出タスク
// ---------------------------------------------------------------------------

/// 個別 DL の保存ダイアログ。 デフォルトファイル名は `default_name`。
pub fn pick_save_one_task(idx: usize, default_name: String) -> Task<Message> {
    Task::perform(
        async move {
            let dialog = rfd::AsyncFileDialog::new().set_file_name(&default_name);
            let chosen = dialog.save_file().await;
            (idx, chosen.map(|h| h.path().to_path_buf()))
        },
        |(idx, path)| Message::DownloadOneTargetPicked(idx, path),
    )
}

/// ZIP 一括 DL の保存ダイアログ。 デフォルトファイル名は `favicon-bundle.zip`。
pub fn pick_save_all_task() -> Task<Message> {
    Task::perform(
        async move {
            let dialog = rfd::AsyncFileDialog::new()
                .set_file_name("favicon-bundle.zip")
                .add_filter("ZIP", &["zip"]);
            let chosen = dialog.save_file().await;
            chosen.map(|h| h.path().to_path_buf())
        },
        Message::DownloadAllTargetPicked,
    )
}

/// 単一ファイルを `path` に書き出す。
pub fn write_one_task(path: PathBuf, bytes: Vec<u8>) -> Task<Message> {
    Task::perform(
        async move {
            std::fs::write(&path, &bytes)
                .map(|_| path.clone())
                .map_err(|e| AppError::export(format!("failed to write {}: {}", path.display(), e)))
        },
        Message::DownloadOneCompleted,
    )
}

/// 全アセットを ZIP に固めて `path` に書き出す。
pub fn write_zip_task(path: PathBuf, items: Vec<ResultAssetItem>) -> Task<Message> {
    Task::perform(
        async move { write_zip_blocking(&path, &items).map(|_| path.clone()) },
        Message::DownloadAllCompleted,
    )
}

fn write_zip_blocking(path: &std::path::Path, items: &[ResultAssetItem]) -> Result<(), AppError> {
    use std::io::Write;
    let file = std::fs::File::create(path)
        .map_err(|e| AppError::export(format!("failed to create zip file: {}", e)))?;
    let mut zip = zip::ZipWriter::new(file);
    let opts: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    for item in items {
        zip.start_file(item.file_name.as_str(), opts)
            .map_err(|e| AppError::export(format!("zip start_file: {}", e)))?;
        zip.write_all(&item.bytes)
            .map_err(|e| AppError::export(format!("zip write_all: {}", e)))?;
    }
    zip.finish()
        .map_err(|e| AppError::export(format!("zip finish: {}", e)))?;
    Ok(())
}
