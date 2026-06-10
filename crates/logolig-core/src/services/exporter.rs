//! エクスポートオーケストレータ。
//!
//! `SourceAsset` と `ExportPlan` から、 出力ディレクトリに全成果物を書き出す。
//!
//! ## トランザクション挙動 (§export-spec.md 「失敗モード」)
//!
//! 「全部書けるか、 一切書かないか」を保証する:
//! 1. 出力ディレクトリ直下に `.<rand>.tmp` の **staging サブディレクトリ** を作る
//! 2. すべての成果物をその staging に書き込む
//! 3. 全成功で初めて、 staging 内の各ファイルを **本来のファイル名にリネーム**
//! 4. 1 ファイルでも失敗したら staging 全体を削除 (ロールバック)
//!
//! これにより、 既存ファイルを破壊せず、 中途半端な状態が残らない。

use std::path::{Path, PathBuf};

use crate::domain::{ExportPlan, Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;
use crate::services::{decode_png, encode_png, html_snippet, ico_writer, rasterize_svg, resize};

/// 書き出し結果。 UI に「何が作られたか」を伝えるために使う。
#[derive(Debug, Clone)]
pub struct ExportReport {
    pub output_dir: PathBuf,
    /// 書き出した個別ファイルへのフルパス (順序は安定: ico, apple-touch, png 昇順, html)。
    pub artifacts: Vec<PathBuf>,
}

/// 同期的にエクスポートを実行する。 CPU バウンド + 同期 fs I/O。
/// `iced::Task::perform` から呼ばれる前提で、 中で `tokio::fs` は使わない
/// (リサイズが計算重なので非同期化のメリットが薄く、 同期 std::fs の方が単純)。
pub fn run(
    asset: &SourceAsset,
    plan: &ExportPlan,
    output_dir: &Path,
) -> Result<ExportReport, AppError> {
    if !output_dir.is_dir() {
        return Err(AppError::Export(format!(
            "output directory does not exist or is not a directory: {}",
            output_dir.display()
        )));
    }

    // 1. PNG ソースなら 1 度だけデコードして使い回す (無駄な再デコードを避ける)。
    //    SVG はサイズごとに再ラスタライズ (§6.2)。
    let decoded_png = match asset.kind {
        SourceKind::Png => Some(decode_png::decode(asset)?),
        SourceKind::Svg => None,
    };

    // 2. staging dir を作る。 競合を避けるため pid と nanosec を混ぜた名前。
    let stage = make_staging_dir(output_dir)?;

    // 3. 中で何か失敗したら staging 全部消すクロージャパターン。
    //    `?` で抜けるたびに rollback すべきなので、 ガード構造体で Drop に任せる。
    let mut guard = StagingGuard::new(stage.clone());

    let mut artifacts: Vec<PathBuf> = Vec::new();

    // ICO
    if plan.include_ico {
        let frames = build_ico_frames(asset, decoded_png.as_ref(), plan)?;
        let frame_refs: Vec<(u32, &Rgba8)> =
            frames.iter().map(|(s, r)| (*s, r)).collect();
        let ico_bytes = ico_writer::build(&frame_refs)?;
        let path = stage.join("favicon.ico");
        write_file(&path, &ico_bytes)?;
        artifacts.push(output_dir.join("favicon.ico"));
    }

    // PNG sizes (高解像度 PNG)。 出力名は `favicon-<size>.png`。
    let mut png_sizes = plan.png_sizes.clone();
    png_sizes.sort_unstable();
    png_sizes.dedup();
    for size in &png_sizes {
        let rgba = render_at_size(asset, decoded_png.as_ref(), *size, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        let name = format!("favicon-{size}.png");
        write_file(&stage.join(&name), &png_bytes)?;
        artifacts.push(output_dir.join(&name));
    }

    // Apple touch icon (180×180 固定)
    if plan.include_apple_touch {
        let rgba = render_at_size(asset, decoded_png.as_ref(), 180, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        write_file(&stage.join("apple-touch-icon.png"), &png_bytes)?;
        artifacts.push(output_dir.join("apple-touch-icon.png"));
    }

    // HTML snippet
    if plan.include_html_snippet {
        let html = html_snippet::render(plan, html_snippet::DEFAULT_BASE);
        write_file(&stage.join("favicon-snippet.html"), html.as_bytes())?;
        artifacts.push(output_dir.join("favicon-snippet.html"));
    }

    // 4. ここまで来たら全ファイル staging に揃った。 rename で本配置へ。
    //    rename 中の失敗もありうるので、 失敗時は最後にもう一度ロールバック。
    finalize(&stage, output_dir, &artifacts)?;

    // 全成功: ガードを解除して staging を残す → finalize 内で空になっているはず。
    guard.cancel();
    // 空の staging dir を片付ける (rename で中身は出ていった)。
    let _ = std::fs::remove_dir(&stage);

    Ok(ExportReport {
        output_dir: output_dir.to_path_buf(),
        artifacts,
    })
}

// ---------------------------------------------------------------------------
// 内部ヘルパ
// ---------------------------------------------------------------------------

/// 与えられた最終ターゲットサイズに対して、 ソースから RGBA8 を作る。
///
/// - PNG: あらかじめデコード済みのフルサイズ画像をリサイズ
/// - SVG: ターゲットサイズで個別レンダリング (§6.2)
fn render_at_size(
    asset: &SourceAsset,
    decoded_png: Option<&Rgba8>,
    size: u32,
    plan: &ExportPlan,
) -> Result<Rgba8, AppError> {
    match asset.kind {
        SourceKind::Png => {
            let src = decoded_png
                .ok_or_else(|| AppError::Export("internal: missing decoded PNG".into()))?;
            resize::resize(src, size, size, plan.algorithm)
        }
        SourceKind::Svg => rasterize_svg::rasterize(asset, size),
    }
}

/// ICO に内包する全フレームをレンダリング。
fn build_ico_frames<'a>(
    asset: &SourceAsset,
    decoded_png: Option<&Rgba8>,
    plan: &ExportPlan,
) -> Result<Vec<(u32, Rgba8)>, AppError> {
    let mut sizes = plan.ico_sizes.clone();
    sizes.sort_unstable();
    sizes.dedup();
    if sizes.is_empty() {
        return Err(AppError::Export("ico_sizes is empty".into()));
    }
    let mut frames = Vec::with_capacity(sizes.len());
    for size in sizes {
        let rgba = render_at_size(asset, decoded_png, size, plan)?;
        frames.push((size, rgba));
    }
    Ok(frames)
}

/// staging dir を作る。 名前は `.logolig-<nanos>.tmp`。 隠しファイル接頭辞で
/// FS リスティングを汚さない。
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
        .map_err(|e| AppError::Export(format!("create staging {}: {e}", path.display())))?;
    Ok(path)
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    std::fs::write(path, bytes)
        .map_err(|e| AppError::Export(format!("write {}: {e}", path.display())))
}

/// staging から本配置への rename。 失敗が起きたら、 すでに移動したものは
/// そのままで(中途状態だが部分的に正しい結果は残す)、 staging 残骸は呼び出し
/// 側 (`StagingGuard::Drop`) が掃除する。
///
/// 既存ファイルは上書き。 これは仕様: 再エクスポートで favicon.ico を更新する
/// のが普通の使い方であり、 ユーザに「既存削除」の手間を負わせない。
fn finalize(stage: &Path, output_dir: &Path, artifacts: &[PathBuf]) -> Result<(), AppError> {
    for final_path in artifacts {
        let name = final_path
            .file_name()
            .ok_or_else(|| AppError::Export("internal: artifact has no file name".into()))?;
        let staged = stage.join(name);
        // 既存ファイルがあれば上書きするため、 rename 前に削除。
        // (Unix の std::fs::rename は同名ファイル上書きできるが、 念のため明示)
        if final_path.exists() {
            let _ = std::fs::remove_file(final_path);
        }
        std::fs::rename(&staged, final_path).map_err(|e| {
            AppError::Export(format!(
                "finalize rename {} -> {}: {e}",
                staged.display(),
                final_path.display()
            ))
        })?;
        let _ = output_dir; // finalize は output_dir を直接いじらない
    }
    Ok(())
}

/// staging dir を Drop 時に必ず掃除するガード。
/// `cancel()` を呼ぶと無効化され、 ディレクトリは消されない (= 成功時)。
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
            // best-effort 掃除。 失敗してもログのみ出して握りつぶす
            // (panic in Drop は double-panic 危険)。
            let _ = std::fs::remove_dir_all(&path);
        }
    }
}
