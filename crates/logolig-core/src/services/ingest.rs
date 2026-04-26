//! ファイル読み込み (ingest)。受け入れる形式は PNG / SVG (§6.1)。
//!
//! 設計:
//! - **非同期**。`tokio::fs::read` でファイル全バイトを読み、UI スレッドを止めない (§2.4)
//! - **非破壊**。元ファイルは触らない。返す `SourceAsset.raw` は読み込んだバイトの
//!   `Arc<[u8]>` ラップ。以降、変換のたびにここから再展開する (§6.4)
//! - **判定は二段階**。まず拡張子で当たりをつけ、次に先頭バイトで本物の形式を確認する。
//!   拡張子だけだと ".png" を名乗る別形式を取りこぼすため
//! - **論理サイズ**。PNG はヘッダの IHDR から、SVG は usvg 経由で size を取得し
//!   `intrinsic_size: Option<(u32, u32)>` に格納する。プレビューと出力計画で使う

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::domain::{SourceAsset, SourceKind};
use crate::error::AppError;

/// ファイルを読み込んで `SourceAsset` を返す。
///
/// この関数は UI スレッドをブロックしないために
/// `iced::Task::perform` から呼び出される想定 (§2.4)。
pub async fn ingest(path: PathBuf) -> Result<SourceAsset, AppError> {
    // 1. 拡張子で第一段階の絞り込み（unknown → 受け付けない）
    let ext_kind = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceKind::from_extension);

    // 2. ファイルを非同期に読む
    let raw = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::Io(format!("{}: {e}", path.display())))?;

    // 3. 先頭バイトで実形式を確定。マジックバイト未一致なら UnsupportedFile。
    let kind = detect_kind(&raw, ext_kind)
        .ok_or_else(|| AppError::UnsupportedFile(path.display().to_string()))?;

    // 4. 論理サイズの推定
    let intrinsic_size = match kind {
        SourceKind::Png => parse_png_size(&raw),
        SourceKind::Svg => parse_svg_size(&raw),
    };

    Ok(SourceAsset {
        path,
        kind,
        raw: Arc::<[u8]>::from(raw),
        intrinsic_size,
    })
}

/// 同期版。テスト時に手で書いたバイト列をそのまま投入したい場合に使う。
/// プロダクトの I/O パスではこれを直接呼ばない（必ず async 版経由）。
pub fn ingest_bytes(
    path: impl AsRef<Path>,
    bytes: Vec<u8>,
) -> Result<SourceAsset, AppError> {
    let path = path.as_ref().to_path_buf();
    let ext_kind = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(SourceKind::from_extension);

    let kind = detect_kind(&bytes, ext_kind)
        .ok_or_else(|| AppError::UnsupportedFile(path.display().to_string()))?;

    let intrinsic_size = match kind {
        SourceKind::Png => parse_png_size(&bytes),
        SourceKind::Svg => parse_svg_size(&bytes),
    };

    Ok(SourceAsset {
        path,
        kind,
        raw: Arc::<[u8]>::from(bytes),
        intrinsic_size,
    })
}

// ---------------------------------------------------------------------------
// 内部: マジックバイト判定とヘッダ解析
// ---------------------------------------------------------------------------

const PNG_MAGIC: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

fn detect_kind(bytes: &[u8], hint: Option<SourceKind>) -> Option<SourceKind> {
    if bytes.starts_with(PNG_MAGIC) {
        return Some(SourceKind::Png);
    }
    if looks_like_svg(bytes) {
        return Some(SourceKind::Svg);
    }
    // マジックバイトが無いものは拡張子を信用しない方針。
    // 例えば偽装 "*.png" を受け取らない。
    let _ = hint;
    None
}

/// 軽量な SVG 判定。XML 宣言や DOCTYPE の前にコメント・BOM が来る場合があるので、
/// 先頭側 1KB に `<svg` が含まれていれば SVG として扱う。
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head_len = bytes.len().min(1024);
    let head = &bytes[..head_len];
    if let Ok(s) = std::str::from_utf8(head) {
        let low = s.to_ascii_lowercase();
        low.contains("<svg")
    } else {
        false
    }
}

/// PNG IHDR から幅と高さを読む。
/// PNG は固定の 8 バイトマジックの直後に IHDR チャンクが来る:
///   8 (magic) + 4 (length) + 4 ("IHDR") + 4 (width BE) + 4 (height BE) ...
fn parse_png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    // IHDR の width フィールドが始まるオフセット
    const W_OFF: usize = 16;
    if bytes.len() < W_OFF + 8 {
        return None;
    }
    if !bytes.starts_with(PNG_MAGIC) {
        return None;
    }
    if &bytes[12..16] != b"IHDR" {
        return None;
    }
    let w = u32::from_be_bytes(bytes[W_OFF..W_OFF + 4].try_into().ok()?);
    let h = u32::from_be_bytes(bytes[W_OFF + 4..W_OFF + 8].try_into().ok()?);
    if w == 0 || h == 0 {
        None
    } else {
        Some((w, h))
    }
}

/// SVG のサイズは usvg にパースさせて求める。
/// width/height 属性が無い SVG (viewBox のみ) でも usvg が補完してくれる。
/// パース失敗時は `None` を返し、ingest 自体は成功させる
/// (ラスタライズ時に再度パースして失敗を見せる方が UI 上の文脈が明確)。
fn parse_svg_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(bytes, &opt).ok()?;
    let size = tree.size();
    Some((size.width().ceil() as u32, size.height().ceil() as u32))
}
