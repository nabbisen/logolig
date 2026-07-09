//! Export orchestrator (v1.16.0 in-memory model).
//!
//! ## v1.16.0 model
//!
//! Previous versions (≤ v1.15) wrote files directly to a user-selected
//! directory. v1.16 switched to an **in-memory** model:
//!
//! 1. File dropped
//! 2. `run_in_memory()` converts everything and returns `Vec<InMemoryArtifact>`
//! 3. Result screen: user downloads individual files or a single ZIP
//!
//! No directory picker, no intermediate writes. The user sees results
//! immediately and chooses what to save.
//!
//! ## Primary API
//!
//! `run_in_memory(source, plan)` is the single entry point. It accepts a
//! `SourceAsset` and an `ExportPlan` and returns `Vec<InMemoryArtifact>`.
//! The caller (logolig-app `task_queue`) wraps this in `iced::Task::perform`.

//!
//! - [`run`] — writes to disk. Was the primary API before v1.16.0.
//!   Now a thin wrapper: calls `run_in_memory` then writes atomically
//!   via a staging directory (refactored in v1.19.0).
//!
//! ## Transactional behaviour (§export-spec.md, [`run`] only)
//!
//! Guarantees "all or nothing":
//! 1. Create a `.<rand>.tmp` **staging subdirectory** inside the output directory.
//! 2. Write all artifacts to staging.
//! 3. On full success, rename each staging file to its final name.
//! 4. On any failure, delete the entire staging directory (rollback).
//!
//! `run_in_memory` never touches the disk, so transactional semantics
//! are not needed (the caller decides whether to use the Vec at all).

use std::path::{Path, PathBuf};

use crate::domain::{ExportPlan, MICROSOFT_APP_LOGOS, Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;
use crate::services::{
    canvas, decode_jpeg, decode_png, decode_webp, encode_png, html_snippet, ico_writer,
    manifest_writer, monochrome, rasterize_svg, resize, vectorize,
};

/// Result of a disk-write operation; tells the UI what was created.
#[derive(Debug, Clone)]
pub struct ExportReport {
    pub output_dir: PathBuf,
    /// Full paths of individual output files (stable order: ico, apple-touch, png ascending, html).
    pub artifacts: Vec<PathBuf>,
}

/// A single in-memory asset.
///
/// - `relative_path`: path relative to the output directory (e.g. `favicon.ico`,
///   `favicon-16.png`, `mono/favicon-32.png`, `manifest.webmanifest`).
///   Uses `PathBuf` rather than `String` because entries may include
///   sub-directories (e.g. `mono/`).
/// - `bytes`: raw file contents.
///
/// Generation order within [`run_in_memory`] return value is stable
/// apple-touch → manifest → mono/ → html). This determines the card grid order
/// in the UI (favicon.ico appears first).
#[derive(Debug, Clone)]
pub struct InMemoryArtifact {
    pub relative_path: PathBuf,
    pub bytes: Vec<u8>,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generate all artifacts in memory (v1.19.0). No disk I/O.
///
/// CPU-bound and synchronous. Intended to be called from `iced::Task::perform`;
/// does not use `tokio::*` internally. Total favicon byte size is typically < 1 MB,
/// Memory overhead is negligible (favicon suite is typically < 1 MB).
///
/// Error handling: any single artifact failure returns `Err` immediately.
/// Partial results (only ICO, or some PNGs) are never returned,
/// avoiding UI complexity for "partial success" states.
pub fn run_in_memory(
    asset: &SourceAsset,
    plan: &ExportPlan,
) -> Result<Vec<InMemoryArtifact>, AppError> {
    let mut artifacts: Vec<InMemoryArtifact> = Vec::new();

    // 1. Decode raster sources (PNG / WebP / JPEG) once; reuse for all sizes.
    //    SVG is re-rendered per size (§6.2).
    let decoded_raster: Option<Rgba8> = match asset.kind {
        SourceKind::Png => Some(decode_png::decode(asset)?),
        SourceKind::Webp => Some(decode_webp::decode(asset)?),
        SourceKind::Jpeg => Some(decode_jpeg::decode(asset)?),
        SourceKind::Svg => None,
    };

    // SVG output (v1.2.0). Track whether SVG was actually emitted via
    // `svg_actually_emitted` and reflect it when generating the HTML snippet.
    let svg_actually_emitted = if plan.include_svg {
        match asset.kind {
            SourceKind::Svg => {
                // Copy SVG source as-is. No re-parse or re-serialise;
                // preserve the original bytes (§6.4 non-destructive).
                push_artifact(&mut artifacts, "favicon.svg", asset.raw.to_vec());
                true
            }
            SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
                if plan.vectorize_on_raster {
                    let src = decoded_raster
                        .as_ref()
                        .ok_or_else(|| AppError::export("internal: missing decoded raster"))?;
                    let svg_string = vectorize::vectorize(src, plan.vtracer_preset)?;
                    push_artifact(&mut artifacts, "favicon.svg", svg_string.into_bytes());
                    true
                } else {
                    false
                }
            }
        }
    } else {
        false
    };

    // ICO
    if plan.include_ico {
        let frames = build_ico_frames(asset, decoded_raster.as_ref(), plan)?;
        let frame_refs: Vec<(u32, &Rgba8)> = frames.iter().map(|(s, r)| (*s, r)).collect();
        let ico_bytes = ico_writer::build(&frame_refs)?;
        push_artifact(&mut artifacts, "favicon.ico", ico_bytes);
    }

    // High-resolution PNG sizes. Output name: `favicon-<size>.png`.
    let mut png_sizes = plan.png_sizes.clone();
    png_sizes.sort_unstable();
    png_sizes.dedup();
    for size in &png_sizes {
        let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        let name = format!("favicon-{size}.png");
        push_artifact(&mut artifacts, &name, png_bytes);
    }

    // Generate apple-touch-icon.png (180×180 fixed size)
    if plan.include_apple_touch {
        let rgba = render_at_size(asset, decoded_raster.as_ref(), 180, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        push_artifact(&mut artifacts, "apple-touch-icon.png", png_bytes);
    }

    // v1.8.0: Web manifest output.
    if let Some(manifest_settings) = plan.web_manifest.as_ref() {
        let manifest_json =
            manifest_writer::build_manifest_json(manifest_settings, &plan.png_sizes);
        push_artifact(
            &mut artifacts,
            manifest_writer::MANIFEST_FILENAME,
            manifest_json.into_bytes(),
        );
    }

    // v1.26.0: Minimal Microsoft app logo set.
    if plan.include_microsoft_app_logos {
        for spec in MICROSOFT_APP_LOGOS {
            let rgba = render_to_contain_canvas(
                asset,
                decoded_raster.as_ref(),
                spec.width,
                spec.height,
                plan,
            )?;
            let png_bytes = encode_png::encode(&rgba)?;
            push_artifact(&mut artifacts, spec.file_name, png_bytes);
        }
    }

    // v1.9.0: Monochrome output set (mono/ subdirectory).
    if plan.monochrome {
        // Monochrome PNG per size. Same order and naming as normal PNGs.
        for size in &png_sizes {
            let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
            let mono_rgba = monochrome::to_grayscale(&rgba);
            let png_bytes = encode_png::encode(&mono_rgba)?;
            let name = format!("mono/favicon-{size}.png");
            push_artifact(&mut artifacts, &name, png_bytes);
        }

        // Monochrome ICO
        if plan.include_ico {
            let frames = build_ico_frames(asset, decoded_raster.as_ref(), plan)?;
            let mono_frames: Vec<(u32, Rgba8)> = frames
                .into_iter()
                .map(|(size, rgba)| (size, monochrome::to_grayscale(&rgba)))
                .collect();
            let frame_refs: Vec<(u32, &Rgba8)> = mono_frames.iter().map(|(s, r)| (*s, r)).collect();
            let ico_bytes = ico_writer::build(&frame_refs)?;
            push_artifact(&mut artifacts, "mono/favicon.ico", ico_bytes);
        }

        // SVG monochrome is out of scope in v1.9.0 (see git log).
    }

    // HTML snippet. Temporarily patch the plan to reflect whether SVG was actually written.
    if plan.include_html_snippet {
        let mut effective_plan = plan.clone();
        effective_plan.include_svg = svg_actually_emitted;
        let html = html_snippet::render(&effective_plan, html_snippet::DEFAULT_BASE);
        push_artifact(&mut artifacts, "favicon-snippet.html", html.into_bytes());
    }

    Ok(artifacts)
}

/// Write to disk (legacy path from ≤ v1.15). Refactored in v1.19.0:
/// calls `run_in_memory` to assemble all artifacts, then writes atomically
/// via a staging directory.
///
/// Existing tests (`tests/exporter.rs`, 12 cases) call this API directly,
/// so its signature is kept compatible with v1.18.
pub fn run(
    asset: &SourceAsset,
    plan: &ExportPlan,
    output_dir: &Path,
) -> Result<ExportReport, AppError> {
    if !output_dir.is_dir() {
        return Err(AppError::export(format!(
            "output directory does not exist or is not a directory: {}",
            output_dir.display()
        )));
    }

    // 1. Assemble all artifacts in memory. On failure, nothing touches disk
    //    (equivalent to the old staging guard behaviour, naturally).
    let in_memory = run_in_memory(asset, plan)?;

    // 2. Create a staging dir (pid + nanosec in name to avoid conflicts).
    let stage = make_staging_dir(output_dir)?;
    let mut guard = StagingGuard::new(stage.clone());

    // 3. Write each artifact to staging. Create parent dirs (e.g. mono/)
    //    as needed.
    let mut artifacts: Vec<PathBuf> = Vec::with_capacity(in_memory.len());
    for art in &in_memory {
        let staged = stage.join(&art.relative_path);
        if let Some(parent) = staged.parent() {
            if parent != stage && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::export(format!("create stage subdir {}: {e}", parent.display()))
                })?;
            }
        }
        write_file(&staged, &art.bytes)?;
        artifacts.push(output_dir.join(&art.relative_path));
    }

    // 4. All files in staging. Rename to final paths.
    finalize(&stage, output_dir, &artifacts)?;

    // All succeeded: disarm the guard (staging is now empty after renames).
    guard.cancel();
    // Remove the now-empty staging dir (contents were moved by rename).
    let _ = std::fs::remove_dir_all(&stage);

    Ok(ExportReport {
        output_dir: output_dir.to_path_buf(),
        artifacts,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Helper to push an artifact into the `Vec<InMemoryArtifact>`.
fn push_artifact(artifacts: &mut Vec<InMemoryArtifact>, relative_path: &str, bytes: Vec<u8>) {
    artifacts.push(InMemoryArtifact {
        relative_path: PathBuf::from(relative_path),
        bytes,
    });
}

/// Renders the source to Rgba8 at the given target size.
///
/// - PNG / WebP / JPEG: resizes the pre-decoded full-size image.
/// - SVG: renders individually at the target size (§6.2).
///
/// v1.21.0: when `plan.keep_transparency == false`, applies
/// [`flatten::flatten_to_white`] at the end to composite alpha against white.
/// This covers PNG sizes, ICO frames, apple-touch, mono PNG, and mono ICO frames —
/// every raster output path goes through `render_at_size`,
/// so one change here covers all of them. SVG outputs (direct `asset.raw` passthrough
/// or the `vectorize::vectorize` path) do not go through this function
/// and are therefore unaffected (Q2-a policy).
fn render_at_size(
    asset: &SourceAsset,
    decoded_raster: Option<&Rgba8>,
    size: u32,
    plan: &ExportPlan,
) -> Result<Rgba8, AppError> {
    let rgba = match asset.kind {
        SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
            let src = decoded_raster
                .ok_or_else(|| AppError::export("internal: missing decoded raster"))?;
            resize::resize(src, size, size, plan.algorithm)?
        }
        SourceKind::Svg => rasterize_svg::rasterize(asset, size)?,
    };
    if plan.keep_transparency {
        Ok(rgba)
    } else {
        Ok(crate::services::flatten::flatten_to_white(&rgba))
    }
}

/// Render the source into a non-cropping canvas while preserving aspect ratio.
///
/// Used by Microsoft app logo outputs, especially the non-square
/// `Wide310x150Logo.png`. Existing favicon outputs still use `render_at_size`
/// to preserve their historical square-output behaviour.
fn render_to_contain_canvas(
    asset: &SourceAsset,
    decoded_raster: Option<&Rgba8>,
    width: u32,
    height: u32,
    plan: &ExportPlan,
) -> Result<Rgba8, AppError> {
    let rgba = match asset.kind {
        SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
            let src = decoded_raster
                .ok_or_else(|| AppError::export("internal: missing decoded raster"))?;
            canvas::contain_on_transparent_canvas(src, width, height, plan.algorithm)?
        }
        SourceKind::Svg => rasterize_svg::rasterize_to_canvas(asset, width, height)?,
    };
    if plan.keep_transparency {
        Ok(rgba)
    } else {
        Ok(crate::services::flatten::flatten_to_white(&rgba))
    }
}

/// Renders all frames to be embedded in the ICO file.
fn build_ico_frames(
    asset: &SourceAsset,
    decoded_raster: Option<&Rgba8>,
    plan: &ExportPlan,
) -> Result<Vec<(u32, Rgba8)>, AppError> {
    let mut sizes = plan.ico_sizes.clone();
    sizes.sort_unstable();
    sizes.dedup();
    if sizes.is_empty() {
        return Err(AppError::export("ico_sizes is empty"));
    }
    let mut frames = Vec::with_capacity(sizes.len());
    for size in sizes {
        let rgba = render_at_size(asset, decoded_raster, size, plan)?;
        frames.push((size, rgba));
    }
    Ok(frames)
}

/// Create a staging dir named `.logolig-<pid>-<nanos>.tmp`.
/// The dotfile prefix keeps it out of directory listings.
fn make_staging_dir(parent: &Path) -> Result<PathBuf, AppError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();
    let name = format!(".logolig-{pid}-{nanos}.tmp");
    let path = parent.join(name);
    std::fs::create_dir(&path)
        .map_err(|e| AppError::export(format!("create staging {}: {e}", path.display())))?;
    Ok(path)
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    std::fs::write(path, bytes)
        .map_err(|e| AppError::export(format!("write {}: {e}", path.display())))
}

/// Rename files from staging to their final output paths.
///
/// Existing files are overwritten — this is intentional:
/// re-exporting to update favicon.ico should not require manual deletion.
fn finalize(stage: &Path, output_dir: &Path, artifacts: &[PathBuf]) -> Result<(), AppError> {
    for final_path in artifacts {
        let rel = final_path.strip_prefix(output_dir).map_err(|_| {
            AppError::export(format!(
                "internal: artifact {} not under output_dir {}",
                final_path.display(),
                output_dir.display()
            ))
        })?;
        let staged = stage.join(rel);

        // Create output subdirectory (e.g. mono/) if missing.
        // It only exists in staging; mkdir on the output side before rename.
        if let Some(parent) = final_path.parent() {
            if parent != output_dir && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::export(format!("create output subdir {}: {e}", parent.display()))
                })?;
            }
        }

        // Remove existing file before rename to achieve overwrite semantics.
        if final_path.exists() {
            let _ = std::fs::remove_file(final_path);
        }
        std::fs::rename(&staged, final_path).map_err(|e| {
            AppError::export(format!(
                "finalize rename {} -> {}: {e}",
                staged.display(),
                final_path.display()
            ))
        })?;
    }
    Ok(())
}

/// Drop guard that removes the staging dir. Call `cancel()` to disarm.
struct StagingGuard {
    path: Option<PathBuf>,
}

impl StagingGuard {
    fn new(path: PathBuf) -> Self {
        Self { path: Some(path) }
    }
    fn cancel(&mut self) {
        self.path = None;
    }
}

impl Drop for StagingGuard {
    fn drop(&mut self) {
        if let Some(path) = self.path.take() {
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
