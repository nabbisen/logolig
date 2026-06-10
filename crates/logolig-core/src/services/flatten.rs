//! 透過フラット化 (v1.21.0)。
//!
//! 「透過 (アルファ) を維持しない」 設定 (= `ExportPlan::keep_transparency =
//! false`) のとき、 各ピクセルのアルファ値を **白背景** で合成して、 全
//! ピクセル alpha=255 (完全不透明) の `Rgba8` を返すサービス。
//!
//! ## 用途
//!
//! favicon 用途で「透過の無い `.ico` / `.png` がほしい」 ニーズに応える:
//! - 古いブラウザ / 古い OS (透過 PNG / 透過 ICO の互換性が怪しい時代) との
//!   互換性を意識する場合
//! - アイコン生成パイプライン下流で「常に背景がある」 前提のツールを使う場合
//! - 単純にアルファチャンネルを使いたくないユーザの好み
//!
//! ## 合成色の選択 (Q1-a)
//!
//! 合成先の背景色は **白固定** (`#FFFFFF`)。 多くの favicon ツールのデフォルト
//! と一致し、 ユーザに選択肢を増やさない (UI 複雑度最小)。 v1.x で需要があれば
//! ユーザ指定に拡張可能 (`flatten_color: [u8; 3]` を `ExportPlan` に追加)。
//!
//! ## 合成式 (Porter-Duff "over" with white background)
//!
//! 各ピクセル `(R, G, B, A)` (0-255 整数) に対して:
//!
//! ```text
//! a = A / 255
//! R' = round(R * a + 255 * (1 - a))
//! G' = round(G * a + 255 * (1 - a))
//! B' = round(B * a + 255 * (1 - a))
//! A' = 255
//! ```
//!
//! `A=255` のピクセルは色変化なし (`R'=R` 等)。 `A=0` のピクセルは完全な白
//! `(255, 255, 255, 255)` になる。 中間アルファは線形補間。
//!
//! 浮動小数演算 (`f32`) で計算してから u8 に丸める方式を採用。 ビット演算
//! 最適化版より少し遅いが、 favicon サイズ (最大 1024×1024 = 1M ピクセル) では
//! 実害なし (<10ms)。 monochrome.rs と同じ実装方針で整合。
//!
//! ## アルファ仕様 (sRGB と premultiplied)
//!
//! `Rgba8` のピクセル形式は **straight (un-premultiplied) sRGB**。 これは
//! `decode_png` / `decode_webp` が image crate の `to_rgba8()` 経由で生成する
//! 形式と一致する。 もし将来 premultiplied alpha に切り替えるなら、 ここの
//! 合成式も `R' = R + 255 * (1 - a)` に変える必要がある (現状は不要)。

use std::sync::Arc;

use crate::domain::Rgba8;

/// 白背景でアルファをフラット化する。 戻り値は全ピクセル alpha=255 の Rgba8。
///
/// 入力サイズが 0 であってもエラーにせず、 そのまま空の Rgba8 を返す
/// (Rgba8::try_from_raw が空入力を許容するかは別問題で、 通常 export パイプ
/// ラインでは空 Rgba8 は来ない)。
pub fn flatten_to_white(src: &Rgba8) -> Rgba8 {
    let pixel_count = (src.width as usize) * (src.height as usize);
    let expected_len = pixel_count * 4;
    debug_assert_eq!(
        src.pixels.len(),
        expected_len,
        "Rgba8 invariant: pixels.len() == width * height * 4"
    );

    let mut out: Vec<u8> = Vec::with_capacity(expected_len);
    let src_bytes = src.pixels.as_ref();

    for chunk in src_bytes.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];

        // ホットパスの最適化: alpha=255 (完全不透明) は色変化なし、
        // alpha=0 (完全透明) は白固定にする。 favicon の典型的入力では
        // 大半のピクセルがこの 2 つのどちらかなので、 短絡で f32 演算を
        // 回避すると目に見える速度差が出る。
        let (rp, gp, bp) = match a {
            255 => (r, g, b),
            0 => (255, 255, 255),
            _ => {
                let af = a as f32 / 255.0;
                let one_minus = 1.0 - af;
                let rp = (r as f32 * af + 255.0 * one_minus).round() as u8;
                let gp = (g as f32 * af + 255.0 * one_minus).round() as u8;
                let bp = (b as f32 * af + 255.0 * one_minus).round() as u8;
                (rp, gp, bp)
            }
        };
        out.extend_from_slice(&[rp, gp, bp, 255]);
    }

    // try_from_raw は (width × height × 4) 不変条件を保証する。 我々は
    // 同じ pixel_count で書き込んでいるので必ず Some で返る。 unwrap_or で
    // 防御的に扱うが、 ここに到達することは無い。
    Rgba8::try_from_raw(src.width, src.height, Arc::<[u8]>::from(out))
        .unwrap_or_else(|| src.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Rgba8 {
        Rgba8::try_from_raw(width, height, Arc::<[u8]>::from(pixels))
            .expect("valid Rgba8 fixture")
    }

    #[test]
    fn fully_opaque_pixels_are_unchanged() {
        // 全ピクセル alpha=255 の場合、 RGB は完全に同一のはず。
        let src = make_rgba(2, 1, vec![100, 150, 200, 255, 50, 75, 25, 255]);
        let out = flatten_to_white(&src);
        assert_eq!(out.pixels.as_ref(), &[100, 150, 200, 255, 50, 75, 25, 255]);
    }

    #[test]
    fn fully_transparent_pixels_become_white() {
        // alpha=0 のピクセルは元の RGB を無視して白 (255,255,255) になる。
        let src = make_rgba(1, 1, vec![100, 150, 200, 0]);
        let out = flatten_to_white(&src);
        assert_eq!(out.pixels.as_ref(), &[255, 255, 255, 255]);
    }

    #[test]
    fn half_alpha_blends_halfway_to_white() {
        // alpha=128 (≈ 50%) で R=0 → R' は約 127 (中間グレー)、 alpha は 255 に
        // なる。
        let src = make_rgba(1, 1, vec![0, 0, 0, 128]);
        let out = flatten_to_white(&src);
        let bytes = out.pixels.as_ref();
        // 線形合成: round(0 * 128/255 + 255 * (1 - 128/255)) = round(127.0) = 127
        // 浮動小数の丸めで ±1 ぶれる可能性があるので幅を持たせる。
        assert!(
            (126..=128).contains(&bytes[0]),
            "expected ~127, got {}",
            bytes[0]
        );
        assert_eq!(bytes[3], 255, "alpha must be saturated");
    }

    #[test]
    fn output_alpha_is_always_saturated() {
        // 入力アルファ値が何であれ、 出力は 255 で揃う。
        let src = make_rgba(
            4,
            1,
            vec![
                0, 0, 0, 0, //
                10, 20, 30, 64, //
                40, 80, 120, 192, //
                200, 100, 50, 255,
            ],
        );
        let out = flatten_to_white(&src);
        let bytes = out.pixels.as_ref();
        assert_eq!(bytes[3], 255);
        assert_eq!(bytes[7], 255);
        assert_eq!(bytes[11], 255);
        assert_eq!(bytes[15], 255);
    }

    #[test]
    fn dimensions_are_preserved() {
        // 縦横が変わらないことを保証 (合成は per-pixel なので当然だが回帰防止)。
        let src = make_rgba(7, 3, vec![0; 7 * 3 * 4]);
        let out = flatten_to_white(&src);
        assert_eq!(out.width, 7);
        assert_eq!(out.height, 3);
        assert_eq!(out.pixels.len(), 7 * 3 * 4);
    }
}
