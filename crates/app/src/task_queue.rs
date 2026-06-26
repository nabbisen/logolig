//! Async task helpers.
//!
//! Wraps `iced::Task::perform` to keep the heavy-lifting closures out of
//! the UI layer.
//!
//! This module depends on both `iced::Task` and `crate::app::Message`,
//! so it lives in logolig-app rather than logolig (core).

use std::path::PathBuf;
use std::sync::Arc;

use iced::Task;

use logolig::{AppError, ExportPlan, InMemoryArtifact, ResizeAlgorithm, Rgba8, SourceAsset};

use crate::app::Message;
use crate::result::{ResultAssetItem, ResultAssetKind, ResultAssets};

/// Spawn a file ingestion task.
///
/// Completion is delivered as `Message::IngestCompleted(Result<_,_>)`.
pub fn ingest_task(path: PathBuf) -> Task<Message> {
    Task::perform(
        logolig::services::ingest::ingest(path),
        Message::IngestCompleted,
    )
}

/// Opens an rfd native file picker; the chosen path is sent as
/// Cancellation sends `FilePicked(None)` (§5.1, §12 alternative path).
///
/// `AsyncFileDialog::pick_file()` returns a `FileHandle`; call `.path()`
/// to get a `&Path`, then clone to a `PathBuf` before sending as an iced Task message.
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

/// Spawn a task that generates preview images (16×16 and 120×120). CPU-bound,
/// dispatched via `iced::Task::perform` off the UI thread.
///
/// `SourceAsset` is wrapped in `Arc` so that `raw: Arc<[u8]>`
/// to avoid copying the raw byte buffer.
pub fn build_preview_task(asset: Arc<SourceAsset>, algorithm: ResizeAlgorithm) -> Task<Message> {
    Task::perform(
        async move { logolig::services::preview::build_preview(&asset, algorithm) },
        Message::PreviewBuilt,
    )
}

// ---------------------------------------------------------------------------
// v1.16.0 / v1.19.0: in-memory conversion task
// ---------------------------------------------------------------------------

/// Post-ingest conversion task (introduced v1.16.0, simplified v1.19.0).
///
/// Conversion results are held in memory (`ResultAssets`); only written
/// to disk when the user requests a download.
/// UI receives `Message::ConvertCompleted` and transitions to the Result screen.
///
/// ## v1.19.0 changes
///
/// The old implementation (v1.16) used a temp directory with `exporter::run`,
/// then read results back. v1.19.0 added `exporter::run_in_memory` to
/// logolig (core), eliminating the temp-directory round-trip.
/// Benefits:

/// - Zero disk I/O → compatible with a future browser port

/// - No risk of leaking temp directories on panic
/// - Better performance: favicon output is typically < 1 MB,
///   so in-memory is natural
///
/// Renamed from `convert_in_memory_task` to `convert_task` in v1.19:
/// "in_memory" is now implied (all conversions are in-memory).
pub fn convert_task(asset: Arc<SourceAsset>, plan: ExportPlan) -> Task<Message> {
    Task::perform(
        async move { run_convert(&asset, &plan) },
        Message::ConvertCompleted,
    )
}

fn run_convert(asset: &SourceAsset, plan: &ExportPlan) -> Result<ResultAssets, AppError> {
    // Assemble all artifacts in memory.
    let in_memory = logolig::services::exporter::run_in_memory(asset, plan)?;
    // Convert to ResultAssets (for UI card rendering).
    Ok(collect_assets(in_memory))
}

/// Convert `Vec<InMemoryArtifact>` to `ResultAssets` for UI card display.
///
/// For each artifact:
/// 1. Classify kind (PngMono / Png / Ico / Svg / HtmlSnippet / WebManifest)
/// 2. Extract dimensions (from PNG IHDR / ICO header)
/// 3. Build thumbnail (PNG / ICO only, decoded via image crate)
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

/// Classify asset kind from file name and relative path.
fn classify_asset(file_name: &str, relative_path: &std::path::Path) -> ResultAssetKind {
    let name = file_name.to_ascii_lowercase();
    // Parent dir "mono" → treat as monochrome variant.
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
        // Unexpected format. Fall back to text-like (HtmlSnippet) so it
        // can still be downloaded.
        ResultAssetKind::HtmlSnippet
    }
}

/// Parse image dimensions for PNG or ICO. Returns `None` on failure.
fn derive_dimensions(kind: ResultAssetKind, bytes: &[u8]) -> Option<(u32, u32)> {
    match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => parse_png_size(bytes),
        ResultAssetKind::Ico => {
            // ICO header: 6-byte signature + 16-byte entries.
            // Width/height are at bytes 4 and 5 of the first entry (0 = 256).
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
        // SVG uses viewBox (logically scalable) → return None.
        ResultAssetKind::Svg => None,
        _ => None,
    }
}

/// Extract width/height from PNG IHDR (8-byte signature + 13-byte IHDR payload).
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

/// Pre-decode image assets for card thumbnail display. Returns `None` on failure
/// (asset is treated as text-like and shown with a placeholder icon).
fn build_thumbnail(kind: ResultAssetKind, bytes: &[u8]) -> Option<Rgba8> {
    if !kind.has_visual_thumbnail() {
        return None;
    }
    let format = match kind {
        ResultAssetKind::Png | ResultAssetKind::PngMono => image::ImageFormat::Png,
        ResultAssetKind::Ico => image::ImageFormat::Ico,
        // SVG: no raster thumbnail — show badge only.
        _ => return None,
    };
    let img = image::load_from_memory_with_format(bytes, format).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: std::sync::Arc<[u8]> = rgba.into_raw().into();
    Rgba8::try_from_raw(w, h, pixels)
}

// ---------------------------------------------------------------------------
// v1.16.0: download dialog + write tasks
// ---------------------------------------------------------------------------

/// Show a save dialog for a single file. Default file name is `default_name`.
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

/// Show a save dialog for the ZIP bundle. Default file name is `favicon-bundle.zip`.
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

/// Write a single file to `path`.
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

/// Bundle all assets into a ZIP and write to `path`.
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
