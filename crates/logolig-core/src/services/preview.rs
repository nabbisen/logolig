//! プレビュー用ラスタ生成。
//!
//! プレビューパネル (§5.2) は、 ソース画像を **使われる文脈の解像度** で見せる。
//! このサービスはソースから次の 2 サイズを生成する:
//!
//! - **16×16** — ブラウザタブ表示の実寸 favicon
//! - **120×120** — スマホホーム画面アイコン (60pt の高 DPI 想定)
//!
//! どちらも §6.2 の品質方針通りに生成する:
//! - SVG はターゲットサイズで個別レンダリング (拡大縮小由来の劣化を避ける)
//! - PNG はソースをデコードしてから fast_image_resize でサイズ別に展開

use std::path::PathBuf;

use crate::domain::{ResizeAlgorithm, Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;
use crate::services::{decode_png, rasterize_svg, resize};

/// プレビュー専用のリサイズ済みラスタ群。
///
/// `source_path` と `algorithm` を覚えておくのは、 上位レイヤ (UI) が
/// 「いまのキャッシュは現在の状態に対応しているか」を判定できるようにするため。
#[derive(Debug, Clone)]
pub struct PreviewCache {
    pub source_path: PathBuf,
    pub algorithm: ResizeAlgorithm,
    /// ブラウザタブ用 (16×16, 実寸表示)。
    pub tab_16: Rgba8,
    /// スマートフォンホーム画面用 (120×120, 高 DPI 想定)。
    pub icon_120: Rgba8,
}

/// ソースから両サイズを生成する。CPU バウンドな処理。
///
/// 本サービスは `iced::Task::perform` から非同期に呼ばれる想定 (§2.4)。
/// `async` ブロックで包めるが内部は同期。
pub fn build_preview(
    asset: &SourceAsset,
    algorithm: ResizeAlgorithm,
) -> Result<PreviewCache, AppError> {
    let tab_16 = render_at(asset, 16, algorithm)?;
    let icon_120 = render_at(asset, 120, algorithm)?;
    Ok(PreviewCache {
        source_path: asset.path.clone(),
        algorithm,
        tab_16,
        icon_120,
    })
}

/// 単一サイズのレンダリング:
/// - SVG → ターゲットサイズで直接ラスタライズ
/// - PNG → デコード → リサイズ
fn render_at(
    asset: &SourceAsset,
    size: u32,
    algorithm: ResizeAlgorithm,
) -> Result<Rgba8, AppError> {
    match asset.kind {
        SourceKind::Svg => rasterize_svg::rasterize(asset, size),
        SourceKind::Png => {
            let decoded = decode_png::decode(asset)?;
            resize::resize(&decoded, size, size, algorithm)
        }
    }
}
