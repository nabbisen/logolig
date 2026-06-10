//! モノクローム変換 (v1.9.0)。
//!
//! favicon の追加出力としてグレースケール版を生成するためのサービス。
//! 単色印刷物 / テーマ対応アイコン (CSS `mask-image` 用) / ステンシル等の
//! ロゴ再利用ユースケースを想定。
//!
//! ## 変換アルゴリズム
//!
//! ITU-R BT.709 の輝度公式 (sRGB 整合) を採用:
//!
//! ```text
//! Y = 0.2126 R + 0.7152 G + 0.0722 B
//! ```
//!
//! 単純平均 `(R+G+B)/3` は青が強く見えすぎる。 BT.601 (`0.299R+0.587G+0.114B`)
//! も世間でよく使われるが、 BT.709 は HDTV / sRGB / Web 用途のモダンな標準。
//! 整合性として BT.709 を採用。
//!
//! ## アルファチャネル
//!
//! アルファ値は **そのまま保持**。 透明部分は透明のまま、 不透明部分は
//! グレースケール化される。 これは favicon 用途で必須:
//! - 透明部分が黒で塗りつぶされたら市松背景のチェッカービュー (v1.7) が
//!   無意味になる
//! - apple-touch-icon 等の上位互換 PNG として使えるようにしておく
//!   (本ライブラリでは apple-touch は mono 化しないが、 将来余地のため)
//!
//! ## v1.9.0 のスコープ判断
//!
//! シンプル変換 (BT.709 グレースケール) のみ。 二値化 (閾値で黒/白に振る) は
//! v1.9.1 で別オプションとして追加予定。

use std::sync::Arc;

use crate::domain::Rgba8;

/// BT.709 輝度係数 (sRGB 整合)。 浮動小数で持って u8 に丸める方式が最も素直。
const COEF_R: f32 = 0.2126;
const COEF_G: f32 = 0.7152;
const COEF_B: f32 = 0.0722;

/// 1 ピクセル (R, G, B) を BT.709 輝度値 (u8) に変換する。
///
/// 浮動小数演算で計算後 u8 に丸める。 速度はビット演算最適化版より遅いが、
/// favicon サイズ (最大 1024×1024 = 1M ピクセル) では実害なし (< 10ms)。
#[inline]
fn luma_bt709(r: u8, g: u8, b: u8) -> u8 {
    let y = (r as f32) * COEF_R + (g as f32) * COEF_G + (b as f32) * COEF_B;
    // round して u8 範囲にクランプ。 BT.709 係数は和が 1 なので 0..=255 から
    // 出ないはずだが、 浮動小数の丸め誤差に念のため備える。
    y.round().clamp(0.0, 255.0) as u8
}

/// `Rgba8` をグレースケール `Rgba8` に変換する。 アルファは元のまま保持。
///
/// 戻り値は新しい `Rgba8` (元の画像は変更しない)。 ピクセルバッファは
/// 新規確保され、 `Arc<[u8]>` でラップされる。
pub fn to_grayscale(image: &Rgba8) -> Rgba8 {
    let mut buf = Vec::with_capacity(image.pixels.len());
    for chunk in image.pixels.chunks_exact(4) {
        let y = luma_bt709(chunk[0], chunk[1], chunk[2]);
        buf.push(y); // R = Y
        buf.push(y); // G = Y
        buf.push(y); // B = Y
        buf.push(chunk[3]); // alpha 保持
    }
    Rgba8::try_from_raw(image.width, image.height, Arc::from(buf.into_boxed_slice()))
        .expect("monochrome: input dimensions match buffer length")
}

/// `Rgba8` を借用ではなく所有して変換する場合のヘルパ。 同じ実装。
/// 呼び出し側がすでに所有 `Rgba8` を持つときに `&` を取る無駄を省く。
pub fn into_grayscale(image: Rgba8) -> Rgba8 {
    to_grayscale(&image)
}
