//! リサイズアルゴリズム (§6.2)。
//! 既定値は品質重視 (Lanczos3)。

use std::fmt;

use fast_image_resize::{FilterType, ResizeAlg};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ResizeAlgorithm {
    #[default]
    Lanczos3,
    MitchellNetravali,
    CatmullRom,
    Bilinear,
    /// ピクセルアート向け（補間しない）。
    Nearest,
}

impl ResizeAlgorithm {
    pub fn label(self) -> &'static str {
        match self {
            Self::Lanczos3 => "Lanczos3 (default, high quality)",
            Self::MitchellNetravali => "Mitchell–Netravali",
            Self::CatmullRom => "Catmull–Rom",
            Self::Bilinear => "Bilinear (fast)",
            Self::Nearest => "Nearest (pixel art)",
        }
    }

    /// 詳細設定 UI でループ列挙するための一覧。
    pub fn all() -> [Self; 5] {
        [
            Self::Lanczos3,
            Self::MitchellNetravali,
            Self::CatmullRom,
            Self::Bilinear,
            Self::Nearest,
        ]
    }

    /// `fast_image_resize` の `ResizeAlg` への変換。
    ///
    /// 直接 `FilterType` ではなく `ResizeAlg` を返すのは、
    /// `Nearest` だけが Convolution ではなく独立バリアント (`ResizeAlg::Nearest`)
    /// として表現されているため。
    pub fn to_resize_alg(self) -> ResizeAlg {
        match self {
            Self::Lanczos3 => ResizeAlg::Convolution(FilterType::Lanczos3),
            Self::MitchellNetravali => ResizeAlg::Convolution(FilterType::Mitchell),
            Self::CatmullRom => ResizeAlg::Convolution(FilterType::CatmullRom),
            Self::Bilinear => ResizeAlg::Convolution(FilterType::Bilinear),
            Self::Nearest => ResizeAlg::Nearest,
        }
    }
}

/// `Display` 実装は iced の `pick_list` widget が要求する (`T: ToString`)。
/// `label()` と同じ文字列を返す。
impl fmt::Display for ResizeAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
