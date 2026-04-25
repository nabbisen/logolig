//! リサイズアルゴリズム (§6.2)。
//! 既定値は品質重視 (Lanczos3)。

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
}
