//! ラスタ → SVG ベクトル化 (v1.2.0、 v1.4.1 でプリセット対応)。
//!
//! `vtracer` 0.6 をラップし、 `Rgba8` を SVG 文字列に変換する。
//!
//! ## 設計判断
//!
//! - **入力サイズはソース解像度のまま**。 ベクトル化前のリサイズはしない。
//!   元画像の細部をできるだけ保ったまま輪郭抽出する方が結果が良い
//! - **3 つのプリセット**で挙動を切り替える (v1.4.1)。 `VtracerPreset` enum に
//!   応じて vtracer `Config` を組み立てる。 ユーザに生パラメータを露出しない (§5)
//! - **失敗は `AppError::export(_)` に集約**。 vtracer は `Result<_, String>` を
//!   返すのでメッセージをそのまま埋め込む
//!
//! ## 適性の限界 (呼び出し側への注意)
//!
//! ベクトル化は写真や複雑なグラデーションには向かない。 v1.2.0 から
//! `ExportPlan::vectorize_on_raster` でユーザがオフにできる。 v1.4.1 で
//! プリセット選択肢を追加した目的の一つは、 `Sharp` プリセットでロゴの輪郭が
//! 鋭く出るようにすること (vtracer 既定の `corner_threshold=60` ではロゴで
//! 角が丸まりがち)。

use vtracer::{ColorImage, ColorMode, Config, Hierarchical};

use crate::domain::{Rgba8, VtracerPreset};
use crate::error::AppError;

/// `Rgba8` をベクトル化し、 SVG 文字列を返す。
/// プリセットに応じて vtracer の `Config` を組み立てる。
pub fn vectorize(rgba: &Rgba8, preset: VtracerPreset) -> Result<String, AppError> {
    if rgba.width == 0 || rgba.height == 0 {
        return Err(AppError::export("vectorize: zero-sized raster"));
    }

    // ColorImage は RGBA 4-byte/pixel。 Rgba8 と完全互換。
    // pixels は所有バッファを取られるので 1 度だけコピー。
    let color_image = ColorImage {
        pixels: rgba.as_bytes().to_vec(),
        width: rgba.width as usize,
        height: rgba.height as usize,
    };

    let config = config_for(preset);

    let svg_file = vtracer::convert(color_image, config)
        .map_err(|e| AppError::export(format!("vtracer: {e}")))?;

    // SvgFile は Display 実装で SVG 文字列を出す。
    Ok(format!("{svg_file}"))
}

/// プリセットに対応する vtracer `Config` を生成する。
///
/// ユーザの生 Config 直接編集は受け付けない方針 (§5「迷いを減らす」)。
/// プリセット → Config の対応はここで一元管理し、 必要なら将来カスタム
/// プリセット (`Custom { ... }` バリアント) を増やす。
///
/// ## v1.4.2 における Sharp の調整方針
///
/// v1.4.1 では `filter_speckle=2`、 `path_precision=Some(3)` を含む 4 パラメータ
/// 同時変更だった。 実機検証の結果、 `filter_speckle=2` (細部を残す) と
/// `path_precision=3` (制御点増加) はロゴ用途で輪郭を荒らす方向に作用していた
/// 可能性が高いと判明した。
///
/// v1.4.2 では Default との差分を `corner_threshold` 1 つだけに絞り、
/// 「角を丸めない」 効果のみを単独で観察可能にする。 これは「実証ベースで
/// プリセットを詰める」 アプローチ — 1 パラメータ変更なら効果が判定しやすい。
fn config_for(preset: VtracerPreset) -> Config {
    match preset {
        VtracerPreset::Sharp => Config {
            // ロゴ向け: 角を丸めない。 他は default 維持で副作用を避ける。
            // v1.4.1 で `filter_speckle=2`/`path_precision=3` を入れていたが、
            // ロゴ品質を下げる方向に働いた可能性があり、 v1.4.2 で削除。
            corner_threshold: 80, // default 60 → 80
            ..Config::default()
        },
        VtracerPreset::Default => {
            // v1.2.0 の挙動と完全互換 (vtracer の defaults を尊重)
            Config::default()
        }
        VtracerPreset::PhotoRich => Config {
            // 写真風: 色階層を細かく残す、 小ノイズは無視して面を綺麗に
            color_precision: 8, // 最大精度
            filter_speckle: 8,  // 小さなノイズは無視
            corner_threshold: 45,
            hierarchical: Hierarchical::Stacked,
            color_mode: ColorMode::Color,
            ..Config::default()
        },
    }
}
