//! ラスタ画像を任意サイズに拡縮する (§6.2)。
//!
//! - 既定アルゴリズムは Lanczos3 (`ResizeAlgorithm::default()`)
//! - 縮小時のジャギー・にじみを抑えるため `fast_image_resize` の畳み込み実装を使う
//! - **ピクセルサイズが等しい場合は短絡**して再計算しない (細部の二度掛けを避ける)
//! - **アルファ前乗算** を有効化 (`mul_div_alpha = true` がデフォルト) — 透明端の色滲みを防ぐ

use std::num::NonZeroU32;
use std::sync::Arc;

use fast_image_resize as fr;
use fr::{ResizeOptions, Resizer, images::Image};

use crate::domain::{ResizeAlgorithm, Rgba8};
use crate::error::AppError;

/// 入力 RGBA8 を `(target_w, target_h)` に拡縮した新しい `Rgba8` を返す。
pub fn resize(
    src: &Rgba8,
    target_w: u32,
    target_h: u32,
    algorithm: ResizeAlgorithm,
) -> Result<Rgba8, AppError> {
    if target_w == 0 || target_h == 0 {
        return Err(AppError::resize("target dimensions must be > 0"));
    }
    // ターゲットが入力と同寸ならそのまま返す（短絡）。
    if src.width == target_w && src.height == target_h {
        return Ok(src.clone());
    }

    // 0 サイズチェックは NonZero に通すついでに行う。
    let (sw, sh) = (
        NonZeroU32::new(src.width)
            .ok_or_else(|| AppError::resize("source width is 0"))?,
        NonZeroU32::new(src.height)
            .ok_or_else(|| AppError::resize("source height is 0"))?,
    );

    // src は読み専用 view、 dst は書き換え可能なバッファとして用意する。
    // fast_image_resize 5.x は Image::from_vec_u8 で所有バッファを取り、
    // RGBA8 を表すには PixelType::U8x4 を渡す。
    let mut src_buf: Vec<u8> = src.pixels.to_vec();
    let src_view = Image::from_slice_u8(sw.get(), sh.get(), &mut src_buf, fr::PixelType::U8x4)
        .map_err(|e| AppError::resize(format!("src view: {e}")))?;

    let mut dst = Image::new(target_w, target_h, fr::PixelType::U8x4);

    // 5.x: Resizer は state を持つ。各リサイズで再利用する場合は使い回せるが、
    // 個別呼び出しでは毎回 new() で十分速い。
    let mut resizer = Resizer::new();
    let opts = ResizeOptions::new().resize_alg(algorithm.to_resize_alg());
    resizer
        .resize(&src_view, &mut dst, &opts)
        .map_err(|e| AppError::resize(format!("resize: {e}")))?;

    // dst はピクセル所有バッファ。Vec に取り出して Arc 化。
    let pixels: Vec<u8> = dst.into_vec();
    Rgba8::try_from_raw(target_w, target_h, Arc::<[u8]>::from(pixels))
        .ok_or_else(|| AppError::resize("internal: rgba length mismatch"))
}
