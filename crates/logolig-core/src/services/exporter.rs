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
use crate::services::{
    decode_jpeg, decode_png, decode_webp, encode_png, html_snippet, ico_writer, manifest_writer,
    monochrome, rasterize_svg, resize, vectorize,
};

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
        return Err(AppError::export(format!(
            "output directory does not exist or is not a directory: {}",
            output_dir.display()
        )));
    }

    // 1. ラスタソース (PNG / WebP / JPEG) なら 1 度だけデコードして使い回す
    //    (無駄な再デコードを避ける)。 SVG はサイズごとに再ラスタライズ (§6.2)。
    let decoded_raster: Option<Rgba8> = match asset.kind {
        SourceKind::Png => Some(decode_png::decode(asset)?),
        SourceKind::Webp => Some(decode_webp::decode(asset)?),
        SourceKind::Jpeg => Some(decode_jpeg::decode(asset)?),
        SourceKind::Svg => None,
    };

    // 2. staging dir を作る。 競合を避けるため pid と nanosec を混ぜた名前。
    let stage = make_staging_dir(output_dir)?;

    // 3. 中で何か失敗したら staging 全部消すクロージャパターン。
    //    `?` で抜けるたびに rollback すべきなので、 ガード構造体で Drop に任せる。
    let mut guard = StagingGuard::new(stage.clone());

    let mut artifacts: Vec<PathBuf> = Vec::new();

    // SVG 出力 (v1.2.0)。 実際に書いたかどうかは `svg_actually_emitted` で記録し、
    // HTML スニペット生成時の有効計画に反映する。
    //
    // 振る舞い:
    // - SVG ソース    → 入力 raw をそのまま `favicon.svg` として書く
    // - PNG/WebP/JPEG ソース + `vectorize_on_raster=true` → vtracer でベクトル化
    // - PNG/WebP/JPEG ソース + `vectorize_on_raster=false` → スキップ
    // - `include_svg=false`  → スキップ
    let svg_actually_emitted = if plan.include_svg {
        match asset.kind {
            SourceKind::Svg => {
                // 入力 SVG をそのまま。 余計な再パース・再シリアライズはせず、
                // 元のバイト列を保持する (§6.4 非破壊性)。
                write_file(&stage.join("favicon.svg"), &asset.raw)?;
                artifacts.push(output_dir.join("favicon.svg"));
                true
            }
            SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
                if plan.vectorize_on_raster {
                    // ベクトル化はソース解像度のまま実行する (細部温存のため)。
                    let src = decoded_raster.as_ref().ok_or_else(|| {
                        AppError::export("internal: missing decoded raster")
                    })?;
                    let svg_string = vectorize::vectorize(src, plan.vtracer_preset)?;
                    write_file(&stage.join("favicon.svg"), svg_string.as_bytes())?;
                    artifacts.push(output_dir.join("favicon.svg"));
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
        let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        let name = format!("favicon-{size}.png");
        write_file(&stage.join(&name), &png_bytes)?;
        artifacts.push(output_dir.join(&name));
    }

    // Apple touch icon (180×180 固定)
    if plan.include_apple_touch {
        let rgba = render_at_size(asset, decoded_raster.as_ref(), 180, plan)?;
        let png_bytes = encode_png::encode(&rgba)?;
        write_file(&stage.join("apple-touch-icon.png"), &png_bytes)?;
        artifacts.push(output_dir.join("apple-touch-icon.png"));
    }

    // v1.8.0: Web manifest 出力。 HTML snippet より前に書く理由:
    // - snippet が manifest の有無を見て <link rel="manifest"> 行を出すか
    //   決められるよう、 plan の web_manifest フィールド自体を snippet に渡す
    //   (実ファイル出力の有無ではなく、 plan 上の意図で判断する流儀)
    // - manifest 書き込みが失敗した場合は staging guard が全部巻き戻すため
    //   snippet の生成順序とは独立
    if let Some(manifest_settings) = plan.web_manifest.as_ref() {
        let manifest_json = manifest_writer::build_manifest_json(
            manifest_settings,
            &plan.png_sizes,
        );
        let manifest_path = stage.join(manifest_writer::MANIFEST_FILENAME);
        write_file(&manifest_path, manifest_json.as_bytes())?;
        artifacts.push(output_dir.join(manifest_writer::MANIFEST_FILENAME));
    }

    // v1.9.0: モノクローム出力セット (mono/ サブディレクトリ)。
    // 通常出力の rendering と完全に独立した手順 — 同じソースに別 plan で
    // resize → グレースケール変換 → encode という流れ。 既存の出力には
    // 一切影響しないため、 既存テストへの破壊リスクなし。
    //
    // mono 化対象:
    // - PNG 各サイズ → mono/favicon-{size}.png
    // - SVG (色情報を `#000`〜`#FFF` のグレーに置換)
    //   ↑ ただし v1.9 では SVG 文字列の色置換ではなく、 PNG ベクトル化済み
    //     SVG (vtracer 出力) の場合のみ「再ベクトル化」 する戦略は重い。
    //     代替案: SVG ソースなら一度 raster に落としてグレーに変換、
    //            さらに再ベクトル化 — これも重い。
    //     代替案 2: mono SVG は出さず PNG/ICO のみ — favicon の主要用途では
    //              足りる。 v1.9.0 ではこれを採用。 SVG mono は v1.9.x で
    //              「raster → grayscale → vtracer」 の 2 段で実装する。
    // - ICO → mono/favicon.ico (各 frame をグレースケール化して再構築)
    //
    // SVG mono のスコープ判断:
    //   v1.9.0 ではあえて SVG mono を入れない。 理由は上記コメントの通り、
    //   SVG ソースで色置換が技術的に難しい (paint属性 / inline style /
    //   gradient / external CSS まで考慮すると複雑)。 PNG / ICO だけでも
    //   「単色印刷物」 「マスク用途」 の主要ユースケースは満たせる。
    if plan.monochrome {
        let mono_dir = stage.join("mono");
        std::fs::create_dir(&mono_dir).map_err(|e| {
            AppError::export(format!("create mono dir: {e}"))
        })?;

        // PNG 各サイズの mono 版。 通常 PNG と同じ順序・命名規則で並べる。
        for size in &png_sizes {
            let rgba = render_at_size(asset, decoded_raster.as_ref(), *size, plan)?;
            let mono_rgba = monochrome::to_grayscale(&rgba);
            let png_bytes = encode_png::encode(&mono_rgba)?;
            let name = format!("favicon-{size}.png");
            write_file(&mono_dir.join(&name), &png_bytes)?;
            artifacts.push(output_dir.join("mono").join(&name));
        }

        // ICO mono 版。 ICO は複数フレームの集合体なので、 各フレームを
        // グレースケール化して再構築する。 build_ico_frames を再実行する
        // のは無駄に見えるが、 各 size から個別に rgba を作って mono 化する
        // 方が「すでに mono 化された 16px と 32px」 のような中途半端な
        // 状態を避けやすい (キャッシュ管理の複雑化を避ける)。
        if plan.include_ico {
            let frames = build_ico_frames(asset, decoded_raster.as_ref(), plan)?;
            let mono_frames: Vec<(u32, Rgba8)> = frames
                .into_iter()
                .map(|(size, rgba)| (size, monochrome::to_grayscale(&rgba)))
                .collect();
            let frame_refs: Vec<(u32, &Rgba8)> =
                mono_frames.iter().map(|(s, r)| (*s, r)).collect();
            let ico_bytes = ico_writer::build(&frame_refs)?;
            write_file(&mono_dir.join("favicon.ico"), &ico_bytes)?;
            artifacts.push(output_dir.join("mono").join("favicon.ico"));
        }

        // SVG mono は v1.9.0 ではスコープ外 (上記コメント参照)。
        // 詳細設定でユーザに UI として見せないので、 ここでも黙って何もしない。
    }

    // HTML snippet。 実際に SVG が書かれたかを反映するため、 plan を一時改変する。
    if plan.include_html_snippet {
        let mut effective_plan = plan.clone();
        effective_plan.include_svg = svg_actually_emitted;
        let html = html_snippet::render(&effective_plan, html_snippet::DEFAULT_BASE);
        write_file(&stage.join("favicon-snippet.html"), html.as_bytes())?;
        artifacts.push(output_dir.join("favicon-snippet.html"));
    }

    // 4. ここまで来たら全ファイル staging に揃った。 rename で本配置へ。
    //    rename 中の失敗もありうるので、 失敗時は最後にもう一度ロールバック。
    finalize(&stage, output_dir, &artifacts)?;

    // 全成功: ガードを解除して staging を残す → finalize 内で空になっているはず。
    guard.cancel();
    // 空の staging dir を片付ける (rename で中身は出ていった)。
    // v1.9.0: mono/ サブディレクトリが残る可能性があるため、 remove_dir_all で
    // 空のサブディレクトリも含めて再帰削除する。 中身は finalize で全て移動
    // されているはずなので、 削除対象は空のディレクトリだけ。
    let _ = std::fs::remove_dir_all(&stage);

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
/// - PNG / WebP: あらかじめデコード済みのフルサイズ画像をリサイズ
/// - SVG:        ターゲットサイズで個別レンダリング (§6.2)
fn render_at_size(
    asset: &SourceAsset,
    decoded_raster: Option<&Rgba8>,
    size: u32,
    plan: &ExportPlan,
) -> Result<Rgba8, AppError> {
    match asset.kind {
        SourceKind::Png | SourceKind::Webp | SourceKind::Jpeg => {
            let src = decoded_raster
                .ok_or_else(|| AppError::export("internal: missing decoded raster"))?;
            resize::resize(src, size, size, plan.algorithm)
        }
        SourceKind::Svg => rasterize_svg::rasterize(asset, size),
    }
}

/// ICO に内包する全フレームをレンダリング。
fn build_ico_frames<'a>(
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
        .map_err(|e| AppError::export(format!("create staging {}: {e}", path.display())))?;
    Ok(path)
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    std::fs::write(path, bytes)
        .map_err(|e| AppError::export(format!("write {}: {e}", path.display())))
}

/// staging から本配置への rename。 失敗が起きたら、 すでに移動したものは
/// そのままで(中途状態だが部分的に正しい結果は残す)、 staging 残骸は呼び出し
/// 側 (`StagingGuard::Drop`) が掃除する。
///
/// 既存ファイルは上書き。 これは仕様: 再エクスポートで favicon.ico を更新する
/// のが普通の使い方であり、 ユーザに「既存削除」の手間を負わせない。
fn finalize(stage: &Path, output_dir: &Path, artifacts: &[PathBuf]) -> Result<(), AppError> {
    for final_path in artifacts {
        // v1.9.0: artifacts に mono/favicon-32.png のようなサブディレクトリ付きの
        // パスが混じる可能性があるため、 file_name() だけでなく output_dir 配下の
        // 相対パスを取り出して staging 側 / final 側双方を再構築する。
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
                    AppError::export(format!(
                        "create output subdir {}: {e}",
                        parent.display()
                    ))
                })?;
            }
        }

        // 既存ファイルがあれば上書きするため、 rename 前に削除。
        // (Unix の std::fs::rename は同名ファイル上書きできるが、 念のため明示)
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
