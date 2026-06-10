//! エクスポートオーケストレータ。
//!
//! `SourceAsset` と `ExportPlan` から、 favicon 一式 (.ico / .svg / .png /
//! .html / .webmanifest) を組み立てる。 出力には 2 系統の API がある:
//!
//! - [`run_in_memory`] — メモリ完結。 戻り値は `Vec<InMemoryArtifact>`
//!   (各 artifact は「相対パス + バイト列」)。 ディスク I/O 一切なし。
//!   v1.16.0 で導入された「ファイル投入 → 自動変換 → Result 画面で
//!   個別 DL or ZIP 一括 DL」 のフローで使う。 ブラウザ移行 (= file system
//!   API なしで動く) も視野に入れた API 形状。
//!
//! - [`run`] — ディスク書出し。 旧 v1.15 までの「Export ボタン → 出力先選択
//!   → atomic 書出」 動線で使われていた API。 内部は [`run_in_memory`] を
//!   呼んで結果をディスクに書く薄いラッパに整理 (v1.19.0)。
//!
//! ## トランザクション挙動 (§export-spec.md 「失敗モード」、 [`run`] のみ)
//!
//! 「全部書けるか、 一切書かないか」 を保証する:
//! 1. 出力ディレクトリ直下に `.<rand>.tmp` の **staging サブディレクトリ** を作る
//! 2. すべての成果物をその staging に書き込む
//! 3. 全成功で初めて、 staging 内の各ファイルを **本来のファイル名にリネーム**
//! 4. 1 ファイルでも失敗したら staging 全体を削除 (ロールバック)
//!
//! [`run_in_memory`] はそもそもディスクに触らないため、 トランザクション
//! 性は不要 (戻り値の `Vec` を渡すかどうかで「全部 or なし」 が決まる)。

use std::path::{Path, PathBuf};

use crate::domain::{ExportPlan, Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;
use crate::services::{
    decode_jpeg, decode_png, decode_webp, encode_png, html_snippet, ico_writer, manifest_writer,
    monochrome, rasterize_svg, resize, vectorize,
};

/// ディスク書き出し結果。 UI に「何が作られたか」 を伝えるために使う。
#[derive(Debug, Clone)]
pub struct ExportReport {
    pub output_dir: PathBuf,
    /// 書き出した個別ファイルへのフルパス (順序は安定: ico, apple-touch, png 昇順, html)。
    pub artifacts: Vec<PathBuf>,
}

/// メモリ上のアセット 1 件分。
///
/// - `relative_path`: 出力ディレクトリ起点の相対パス (例: `favicon.ico`、
///   `favicon-16.png`、 `mono/favicon-32.png`、 `manifest.webmanifest`)。
///   サブディレクトリ付きの可能性 (mono/) があるため `String` ではなく
///   `PathBuf` で持つ。
/// - `bytes`: ファイル内容そのもの。
///
/// 生成順序は [`run_in_memory`] 戻り値内で安定 (svg → ico → png 昇順 →
/// apple-touch → manifest → mono/ → html)。 これは UI のカードグリッド表示
/// の並び順 (favicon.ico を最初に見せるなど) を直接決める。
#[derive(Debug, Clone)]
pub struct InMemoryArtifact {
    pub relative_path: PathBuf,
    pub bytes: Vec<u8>,
}

// ---------------------------------------------------------------------------
// 公開 API
// ---------------------------------------------------------------------------

/// メモリ上で全成果物を生成する (v1.19.0)。 ディスク I/O 一切なし。
///
/// CPU バウンド + 同期。 `iced::Task::perform` から呼ばれる前提で、 中で
/// `tokio::*` は使わない。 favicon 一式の合計バイト数は通常 1 MB 未満なので
/// メモリ保持コストは実質ゼロ。
///
/// エラーハンドリング: 1 成果物でも生成に失敗したら即 `Err`。 部分的な
/// 結果 (ico だけ成功 / png 一部成功) は返さない (= UI 側で「途中まで」 を
/// 表示する複雑さを避ける)。
pub fn run_in_memory(
    asset: &SourceAsset,
    plan: &ExportPlan,
) -> Result<Vec<InMemoryArtifact>, AppError> {
    let mut artifacts: Vec<InMemoryArtifact> = Vec::new();

    // 1. ラスタソース (PNG / WebP / JPEG) なら 1 度だけデコードして使い回す
    //    (無駄な再デコードを避ける)。 SVG はサイズごとに再ラスタライズ (§6.2)。
    let decoded_raster: Option<Rgba8> = match asset.kind {
        SourceKind::Png => Some(decode_png::decode(asset)?),
        SourceKind::Webp => Some(decode_webp::decode(asset)?),
        SourceKind::Jpeg => Some(decode_jpeg::decode(asset)?),
        SourceKind::Svg => None,
    };

    // SVG 出力 (v1.2.0)。 実際に書いたかどうかは `svg_actually_emitted` で記録し、
    // HTML スニペット生成時の有効計画に反映する。
    let svg_actually_emitted = if plan.include_svg {
        match asset.kind {
            SourceKind::Svg => {
                // 入力 SVG をそのまま。 余計な再パース・再シリアライズはせず、
                // 元のバイト列を保持する (§6.4 非破壊性)。
                push_artifact(&mut artifacts, "favicon.svg", asset.raw.to_vec());
                true
            }
            SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
                if plan.vectorize_on_raster {
                    let src = decoded_raster.as_ref().ok_or_else(|| {
                        AppError::export("internal: missing decoded raster")
                    })?;
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

    // PNG sizes (高解像度 PNG)。 出力名は `favicon-<size>.png`。
    let mut png_sizes = plan.png_sizes.clone();
    png_sizes.sort_unstable();
    png_sizes.dedup();
    for size in &png_sizes {
        let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        let name = format!("favicon-{size}.png");
        push_artifact(&mut artifacts, &name, png_bytes);
    }

    // Apple touch icon (180×180 固定)
    if plan.include_apple_touch {
        let rgba = render_at_size(asset, decoded_raster.as_ref(), 180, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        push_artifact(&mut artifacts, "apple-touch-icon.png", png_bytes);
    }

    // v1.8.0: Web manifest 出力。
    if let Some(manifest_settings) = plan.web_manifest.as_ref() {
        let manifest_json =
            manifest_writer::build_manifest_json(manifest_settings, &plan.png_sizes);
        push_artifact(
            &mut artifacts,
            manifest_writer::MANIFEST_FILENAME,
            manifest_json.into_bytes(),
        );
    }

    // v1.9.0: モノクローム出力セット (mono/ サブディレクトリ)。
    if plan.monochrome {
        // PNG 各サイズの mono 版。 通常 PNG と同じ順序・命名規則で並べる。
        for size in &png_sizes {
            let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
            let mono_rgba = monochrome::to_grayscale(&rgba);
            let png_bytes = encode_png::encode(&mono_rgba)?;
            let name = format!("mono/favicon-{size}.png");
            push_artifact(&mut artifacts, &name, png_bytes);
        }

        // ICO mono 版
        if plan.include_ico {
            let frames = build_ico_frames(asset, decoded_raster.as_ref(), plan)?;
            let mono_frames: Vec<(u32, Rgba8)> = frames
                .into_iter()
                .map(|(size, rgba)| (size, monochrome::to_grayscale(&rgba)))
                .collect();
            let frame_refs: Vec<(u32, &Rgba8)> =
                mono_frames.iter().map(|(s, r)| (*s, r)).collect();
            let ico_bytes = ico_writer::build(&frame_refs)?;
            push_artifact(&mut artifacts, "mono/favicon.ico", ico_bytes);
        }

        // SVG mono は v1.9.0 ではスコープ外 (詳細は git log)。
    }

    // HTML snippet。 実際に SVG が書かれたかを反映するため、 plan を一時改変する。
    if plan.include_html_snippet {
        let mut effective_plan = plan.clone();
        effective_plan.include_svg = svg_actually_emitted;
        let html = html_snippet::render(&effective_plan, html_snippet::DEFAULT_BASE);
        push_artifact(&mut artifacts, "favicon-snippet.html", html.into_bytes());
    }

    Ok(artifacts)
}

/// ディスクに書き出す (v1.15 までの旧来動線)。 v1.19.0 で内部実装を整理:
/// `run_in_memory` でメモリ上に全成果物を組み上げてから、 staging dir 経由で
/// atomic に書き出す薄いラッパとなった。
///
/// 既存テスト (`tests/exporter.rs` の 12 ケース) はこの API を直接呼ぶため、
/// シグネチャは v1.18 まで と一致を保つ。
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

    // 1. 全成果物をメモリ上で組み立てる。 ここで失敗したらディスクには
    //    一切触らずに即 Err (= 旧 staging guard と同等の挙動が自然に達成される)。
    let in_memory = run_in_memory(asset, plan)?;

    // 2. staging dir を作る。 競合を避けるため pid + nanosec を混ぜた名前。
    let stage = make_staging_dir(output_dir)?;
    let mut guard = StagingGuard::new(stage.clone());

    // 3. 各 artifact を staging に書き出す。 サブディレクトリ (mono/) が
    //    必要な場合は parent を mkdir。
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

    // 4. 全ファイル staging に揃った。 rename で本配置へ。
    finalize(&stage, output_dir, &artifacts)?;

    // 全成功: ガードを解除して staging を残す → finalize 内で空になっているはず。
    guard.cancel();
    // 空の staging dir を片付ける (rename で中身は出ていった)。
    let _ = std::fs::remove_dir_all(&stage);

    Ok(ExportReport {
        output_dir: output_dir.to_path_buf(),
        artifacts,
    })
}

// ---------------------------------------------------------------------------
// 内部ヘルパ
// ---------------------------------------------------------------------------

/// `Vec<InMemoryArtifact>` への push を簡略化するヘルパ。
fn push_artifact(artifacts: &mut Vec<InMemoryArtifact>, relative_path: &str, bytes: Vec<u8>) {
    artifacts.push(InMemoryArtifact {
        relative_path: PathBuf::from(relative_path),
        bytes,
    });
}

/// 与えられた最終ターゲットサイズに対して、 ソースから RGBA8 を作る。
///
/// - PNG / WebP / JPEG: あらかじめデコード済みのフルサイズ画像をリサイズ
/// - SVG: ターゲットサイズで個別レンダリング (§6.2)
///
/// v1.21.0: `plan.keep_transparency == false` の場合、 最終段階で
/// [`flatten::flatten_to_white`] を適用してアルファを白背景で合成する。
/// これは PNG / ICO フレーム / apple-touch / mono PNG / mono ICO フレーム
/// 全てに適用される (= 全 raster 出力経路で `render_at_size` を経由する
/// ため、 ここで一括処理するのが最もキレイ)。 SVG 出力 (`asset.raw` を
/// 直接 push する経路、 もしくは `vectorize::vectorize` 経路) はここを通ら
/// ないので、 Q2-a の方針通り影響を受けない。
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

/// ICO に内包する全フレームをレンダリング。
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

/// staging dir を作る。 名前は `.logolig-<pid>-<nanos>.tmp`。 隠しファイル
/// 接頭辞で FS リスティングを汚さない。
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

/// staging から本配置への rename。
///
/// 既存ファイルは上書き。 これは仕様: 再エクスポートで favicon.ico を更新する
/// のが普通の使い方であり、 ユーザに「既存削除」 の手間を負わせない。
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

        // 出力先のサブディレクトリ (例: mono/) が無ければ作る。 mono/ は
        // staging 側にしか存在しないので、 rename 前に出力側にも mkdir する。
        if let Some(parent) = final_path.parent() {
            if parent != output_dir && !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    AppError::export(format!("create output subdir {}: {e}", parent.display()))
                })?;
            }
        }

        // 既存ファイルがあれば上書きするため、 rename 前に削除。
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

/// staging dir を Drop 時に必ず掃除するガード。 `cancel()` を呼ぶと無効化。
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
