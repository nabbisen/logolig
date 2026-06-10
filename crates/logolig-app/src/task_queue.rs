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

// ---------------------------------------------------------------------------
// v1.16.0: メモリ完結変換タスク
// ---------------------------------------------------------------------------

use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};
use logolig_core::{AppError, Rgba8};

/// v1.16.0: ファイル投入後の自動変換タスク。
///
/// 旧 `export_task` は「ユーザが選択したディレクトリに直接書き出す」 動作
/// だったが、 v1.16 ではモデルが変わって「変換結果はメモリに保持し、
/// ユーザが個別 DL or ZIP DL のときに初めて書き出す」 になる。
///
/// 実装戦略: 当面は exporter::run を **OS の一時ディレクトリ** に書かせて
/// 各ファイルを読み戻すラッパとして実装する。 これは exporter 全体を
/// in-memory API に書き換えずに済むので、 v1.16.0 のスコープを抑える。
/// 将来的に exporter::run_in_memory 直接 API を生やすのは v1.17 以降の課題。
pub fn convert_in_memory_task(
    asset: Arc<SourceAsset>,
    plan: ExportPlan,
) -> Task<Message> {
    Task::perform(
        async move { run_convert_in_memory(&asset, &plan) },
        Message::ConvertCompleted,
    )
}

fn run_convert_in_memory(
    asset: &SourceAsset,
    plan: &ExportPlan,
) -> Result<ResultAssets, AppError> {
    // 一時ディレクトリ: tempfile crate ではなく std::env::temp_dir を使い、
    // pid + nanosec で衝突を避けた sub-directory を作る。 後始末はこの関数の
    // 末尾で removel_dir_all。
    let tmp_root = std::env::temp_dir();
    let stage_name = format!(
        "logolig-convert-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let stage = tmp_root.join(stage_name);
    std::fs::create_dir_all(&stage).map_err(|e| {
        AppError::export(format!(
            "failed to create temp directory: {}",
            e
        ))
    })?;

    // 既存の exporter::run を再利用してファイルとして書き出す。
    let report = match logolig_core::services::exporter::run(asset, plan, &stage) {
        Ok(r) => r,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&stage);
            return Err(e);
        }
    };

    // 書き出されたファイルを ResultAssets として読み戻す。
    let bundle = match collect_assets(&report.artifacts) {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&stage);
            return Err(e);
        }
    };

    // 一時ディレクトリを掃除。 失敗しても処理結果は返せるので無視。
    let _ = std::fs::remove_dir_all(&stage);

    Ok(bundle)
}

fn collect_assets(artifacts: &[PathBuf]) -> Result<ResultAssets, AppError> {
    let mut items = Vec::with_capacity(artifacts.len());
    for path in artifacts {
        let bytes = std::fs::read(path).map_err(|e| {
            AppError::export(format!(
                "failed to read converted artifact {}: {}",
                path.display(),
                e
            ))
        })?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from)
            .unwrap_or_else(|| "(unknown)".to_string());
        let kind = classify_asset(&file_name, path);
        let dimensions = derive_dimensions(&file_name, kind, &bytes);
        let thumbnail = build_thumbnail(kind, &bytes);
        items.push(ResultAssetItem {
            file_name,
            bytes,
            kind,
            dimensions,
            thumbnail,
        });
    }
    Ok(ResultAssets { items })
}

/// ファイル名 (+ パス) からアセット種別を判定。
fn classify_asset(file_name: &str, path: &std::path::Path) -> ResultAssetKind {
    let name = file_name.to_ascii_lowercase();
    // `mono/` サブディレクトリ配下なら mono PNG として扱う。
    let is_mono = path
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
fn derive_dimensions(
    file_name: &str,
    kind: ResultAssetKind,
    bytes: &[u8],
) -> Option<(u32, u32)> {
    match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => parse_png_size(bytes),
        ResultAssetKind::Ico => {
            // ICO ヘッダ: 6 byte 署名 + 各 entry 16 byte。 1st entry の幅/高さは
            // 4 byte 目と 5 byte 目に格納 (0 は 256 を意味)。 単純に 1 個目だけ
            // 読み取って表示用とする (マルチサイズ ICO の場合は最大サイズが
            // 1 個目に来る運用ではないが、 表示用なのでこれで十分)。
            if bytes.len() >= 8 {
                let w = match bytes[6] { 0 => 256, n => n as u32 };
                let h = match bytes[7] { 0 => 256, n => n as u32 };
                Some((w, h))
            } else {
                None
            }
        }
        ResultAssetKind::Svg => {
            // SVG は寸法が viewBox 等で表現され、 事実上「サイズ可変」 なので
            // None でよい。 表示は「SVG」 のバッジのみ。
            let _ = file_name;
            None
        }
        _ => None,
    }
}

/// PNG IHDR から幅高さを取得。 標準形式 (8 byte signature + IHDR 13 byte payload)
/// を仮定するシンプルなパーサ。
fn parse_png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    // 8 byte signature + 4 byte length + 4 byte type + 4 byte width + 4 byte height
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
    // PNG / ICO / SVG いずれも image crate でデコードを試みる。 ICO は image
    // crate が対応している場合と未対応の場合があるので、 失敗しても致命的に
    // しない。
    let format = match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => image::ImageFormat::Png,
        ResultAssetKind::Ico => image::ImageFormat::Ico,
        ResultAssetKind::Svg => return None, // SVG はサムネ生成を行わず、 バッジ表示のみ
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
                .map_err(|e| AppError::export(format!(
                    "failed to write {}: {}",
                    path.display(),
                    e
                )))
        },
        Message::DownloadOneCompleted,
    )
}

/// 全アセットを ZIP に固めて `path` に書き出す。
pub fn write_zip_task(
    path: PathBuf,
    items: Vec<ResultAssetItem>,
) -> Task<Message> {
    Task::perform(
        async move { write_zip_blocking(&path, &items).map(|_| path.clone()) },
        Message::DownloadAllCompleted,
    )
}

fn write_zip_blocking(path: &std::path::Path, items: &[ResultAssetItem]) -> Result<(), AppError> {
    use std::io::Write;
    let file = std::fs::File::create(path).map_err(|e| {
        AppError::export(format!("failed to create zip file: {}", e))
    })?;
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
