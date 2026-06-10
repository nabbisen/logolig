//! vtracer プリセット (v1.4.1)。
//!
//! v1.2.0 で導入したラスタ → SVG ベクトル化機能の品質を、 入力種別に応じて
//! 切り替えられるよう 3 つのプリセットを提供する。
//!
//! ## なぜプリセットか
//!
//! vtracer の生 `Config` はパラメータが多く (color_precision, filter_speckle,
//! corner_threshold, layer_difference, ...)、 ユーザに直接弄らせると「何が
//! 効いているか分からない」 状態に陥る。 仕様 §5「迷いを減らす UI」 に従い、
//! 用途別に 3 つの代表点を用意するアプローチを取る。
//!
//! ## プリセットの設計
//!
//! - **Sharp**: ロゴ・アイコン・線画向け。 色精度は中、 細部 (`filter_speckle=2`)
//!   と角 (`corner_threshold=80`) を保つ
//! - **Default**: vtracer 既定値そのまま。 v1.2.0 ユーザの挙動を保つ互換用
//! - **PhotoRich**: 写真風 / グラデーション向け。 最大色精度 (`color_precision=8`)、
//!   小さなノイズは無視 (`filter_speckle=8`)、 角閾値は緩め (`corner_threshold=45`)
//!
//! ## なぜ core に置くのか
//!
//! UI 層で文字列を持って vtracer に渡すと永続化と整合しない (v2 で localStorage に
//! 保存するときも同じ型が要る)。 ドメイン型として core で定義し、 UI も
//! 永続化も同じ enum を使う。

use serde::{Deserialize, Serialize};

/// vtracer のチューニングプリセット。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum VtracerPreset {
    /// ロゴ・アイコン・線画向け。 シャープな輪郭を維持。
    Sharp,
    /// v1.2.0 と同じ vtracer 既定値。 互換用 (default)。
    #[default]
    Default,
    /// 写真風 / グラデーション。 色階層を細かく残す。
    PhotoRich,
}

impl VtracerPreset {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sharp => "Sharp (logos, icons)",
            Self::Default => "Default (balanced)",
            Self::PhotoRich => "Photo-rich (gradients)",
        }
    }

    /// pick_list 用の全列挙。
    pub fn all() -> [Self; 3] {
        [Self::Sharp, Self::Default, Self::PhotoRich]
    }
}

/// `Display` 実装は iced の `pick_list` widget が要求する (`T: ToString`)。
impl std::fmt::Display for VtracerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}
