//! SVG ラスタライザ。
//!
//! 仕様 §6.2「文字や細線の潰れを最小化する」を守るため、
//! ターゲットサイズに対して **個別に** レンダリングする。
//! 大きいサイズで 1 度ラスタライズして縮小すると、結局縮小由来のジャギーが
//! 出てしまうため、各サイズで Transform をかけ直して描く。

use std::sync::Arc;

use crate::domain::{Rgba8, SourceAsset, SourceKind};
use crate::error::AppError;

/// SVG ソースを指定サイズの正方形 RGBA8 ビットマップに展開する。
///
/// - 出力は `target_size × target_size` の正方形 (favicon 用途のため)
/// - SVG の論理 viewBox を `target_size` に合わせて等比拡縮する
///   (アスペクト比が 1:1 でない場合は中央に配置し余白は透明)
pub fn rasterize(asset: &SourceAsset, target_size: u32) -> Result<Rgba8, AppError> {
    if asset.kind != SourceKind::Svg {
        return Err(AppError::UnsupportedFile(format!(
            "rasterize_svg called on non-SVG source ({})",
            asset.kind.label()
        )));
    }
    if target_size == 0 {
        return Err(AppError::Rasterize("target_size must be > 0".into()));
    }

    // 1. usvg でツリーを得る
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(&asset.raw, &opt)
        .map_err(|e| AppError::Rasterize(format!("usvg parse: {e}")))?;

    let svg_size = tree.size();
    let svg_w = svg_size.width();
    let svg_h = svg_size.height();
    if svg_w <= 0.0 || svg_h <= 0.0 {
        return Err(AppError::Rasterize("SVG has zero or negative size".into()));
    }

    // 2. アスペクト比を保ったまま target_size に収まる scale を選ぶ
    let target = target_size as f32;
    let scale = (target / svg_w).min(target / svg_h);
    let drawn_w = svg_w * scale;
    let drawn_h = svg_h * scale;
    let tx = (target - drawn_w) * 0.5;
    let ty = (target - drawn_h) * 0.5;

    // 3. Pixmap を確保して resvg で描く
    let mut pixmap = tiny_skia::Pixmap::new(target_size, target_size).ok_or_else(|| {
        AppError::Rasterize(format!(
            "tiny_skia: cannot allocate pixmap {target_size}x{target_size}"
        ))
    })?;

    let transform = tiny_skia::Transform::from_scale(scale, scale).post_translate(tx, ty);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // 4. tiny-skia は **premultiplied alpha** を返すため、
    //    そのまま PNG/ICO に渡すと色が暗く見える。straight alpha に戻す。
    let pixels: Vec<u8> = pixmap
        .pixels()
        .iter()
        .flat_map(|p| {
            let d = p.demultiply();
            [d.red(), d.green(), d.blue(), d.alpha()]
        })
        .collect();

    Rgba8::try_from_raw(target_size, target_size, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::Rasterize("internal: rgba length mismatch".into()))
}
