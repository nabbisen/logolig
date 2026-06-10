//! 透過チェッカー (v1.7.0)。
//!
//! favicon ユースケースで起きやすい「画像の透過状態が意図とずれている」
//! ケースを、 入力直後にユーザに知らせるための解析。
//!
//! ## 検出するケース
//!
//! - **`FullyOpaque`** — 全ピクセルが alpha=255。 透過 PNG として書き出しても
//!   意味がなく、 ダークテーマのブラウザタブで「白い四角の中にロゴ」 が出る
//!   失敗パターンの典型
//! - **`FullyTransparent`** — 全ピクセルが alpha=0。 何も描かれていない画像
//!   (空のレイヤーや誤った素材) を選んだ可能性が高い
//! - **`HasTransparency`** — 透明部分と不透明部分が混在する正常ケース
//!
//! ## なぜハロー検出やアルファ事前乗算検査をやらないか
//!
//! これらの判定には閾値設計が必要で、 **正しい画像にも誤警告が出る** 可能性が
//! 高い。 例: ロゴの細部に意図的なアンチエイリアスがあると「半透明残り」 と
//! 誤判定される。 v1.7.0 では「判定が二値で明確 (alpha が完全に 255 か 0 か
//! 否か)」 のケースに絞る。 誤検出のリスクが低い分、 ユーザの信頼を損ねない。
//!
//! 将来 (v1.7.x 以降) で実機の誤警告率を見ながら段階的に追加するか、 もしくは
//! 入れない判断をする。
//!
//! ## パフォーマンス
//!
//! 全ピクセル走査だが、 alpha チャネル 1 バイトを順に読むだけなので 1024×1024
//! でも 1ms 程度。 preview 生成と同じ非同期パイプラインに乗せるが、 重い処理
//! ではない。

use crate::domain::Rgba8;

/// 入力画像の透過状態の分類。
///
/// favicon ユースケースで重要な 3 ケース。 「Indeterminate」 (まだ調べていない)
/// は持たない — `audit` を呼んだら必ずこの 3 つのうちのどれかになる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransparencyReport {
    /// 完全に不透明 (全ピクセル alpha=255)。 透過 PNG として書き出しても効果なし。
    /// ダークテーマで白い背景が浮いて見える典型的な失敗パターン。
    FullyOpaque,
    /// 完全に透明 (全ピクセル alpha=0)。 空のレイヤーや誤った素材を選んだ可能性。
    FullyTransparent,
    /// 透明部分と不透明部分が混在 — 正常な favicon 素材。
    HasTransparency,
}

impl TransparencyReport {
    /// 警告すべき状態か (UI 層で Toast を出すかどうかの判断に使う)。
    /// `HasTransparency` は警告不要。
    pub fn needs_warning(self) -> bool {
        matches!(self, Self::FullyOpaque | Self::FullyTransparent)
    }
}

/// 入力画像の透過状態を解析する。
///
/// 全ピクセルを走査して alpha の min/max を取り、 3 ケースに分類する。
/// 空画像 (width=0 or height=0) は安全側に倒して `FullyTransparent` を返す。
pub fn audit(image: &Rgba8) -> TransparencyReport {
    if image.width == 0 || image.height == 0 || image.pixels.is_empty() {
        // ピクセル 0 個 → 「描かれていない」 と同義
        return TransparencyReport::FullyTransparent;
    }

    // RGBA は 4 バイト 1 ピクセル。 alpha は 4 番目のバイト。
    debug_assert!(
        image.pixels.len() == (image.width as usize) * (image.height as usize) * 4,
        "Rgba8 pixel buffer length mismatch"
    );

    let mut min_alpha: u8 = 255;
    let mut max_alpha: u8 = 0;
    for chunk in image.pixels.chunks_exact(4) {
        // chunk[3] が alpha
        let a = chunk[3];
        if a < min_alpha {
            min_alpha = a;
        }
        if a > max_alpha {
            max_alpha = a;
        }
        // 早期終了 (混在を確認できたらこれ以上見る必要はない)
        if min_alpha == 0 && max_alpha == 255 {
            return TransparencyReport::HasTransparency;
        }
    }

    match (min_alpha, max_alpha) {
        (255, 255) => TransparencyReport::FullyOpaque,
        (0, 0) => TransparencyReport::FullyTransparent,
        // それ以外 (例: alpha が常に 128 のような半透明一様画像) も
        // 「混在ではない」 が favicon 用途で実害は限定的。 警告対象から外し、
        // HasTransparency 扱いにする。 これは将来 (v1.7.x) で見直す余地あり。
        _ => TransparencyReport::HasTransparency,
    }
}
