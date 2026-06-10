//! ラスタ → SVG ベクトル化 (v1.2.0)。
//!
//! `vtracer` 0.6 をラップし、 `Rgba8` を SVG 文字列に変換する。
//!
//! ## 設計判断
//!
//! - **vtracer の defaults をそのまま使う** (`Config::default()`)。 favicon 用途
//!   (ロゴ・アイコン中心) ではこれで十分。 ユーザに迷わせない (§5)
//! - **入力サイズはソース解像度のまま**。 ベクトル化前のリサイズはしない。
//!   元画像の細部をできるだけ保ったまま輪郭抽出する方が結果が良い
//! - **失敗は `AppError::Export(_)` に集約**。 vtracer は `Result<_, String>` を
//!   返すのでメッセージをそのまま埋め込む
//!
//! ## 適性の限界 (呼び出し側への注意)
//!
//! ベクトル化は写真や複雑なグラデーションには向かない。 v1.2.0 では
//! `ExportPlan::vectorize_on_raster` でユーザがオフにできる。 自動判定 (色数
//! ヒューリスティックなど) は v2 以降の検討事項。

use vtracer::{Config, ColorImage};

use crate::domain::Rgba8;
use crate::error::AppError;

/// `Rgba8` をベクトル化し、 SVG 文字列を返す。
///
/// 内部で vtracer のデフォルト設定 (`color`, `stacked`, `spline`, `filter_speckle=4`,
/// `color_precision=6`, ...) を使う。
pub fn vectorize(rgba: &Rgba8) -> Result<String, AppError> {
    if rgba.width == 0 || rgba.height == 0 {
        return Err(AppError::Export(
            "vectorize: zero-sized raster".into(),
        ));
    }

    // ColorImage は RGBA 4-byte/pixel。 Rgba8 と完全互換。
    // pixels は所有バッファを取られるので 1 度だけコピー。
    let color_image = ColorImage {
        pixels: rgba.as_bytes().to_vec(),
        width: rgba.width as usize,
        height: rgba.height as usize,
    };

    let svg_file = vtracer::convert(color_image, Config::default())
        .map_err(|e| AppError::Export(format!("vtracer: {e}")))?;

    // SvgFile は Display 実装で SVG 文字列を出す。
    Ok(format!("{svg_file}"))
}
