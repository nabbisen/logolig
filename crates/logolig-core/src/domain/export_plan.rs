//! 出力計画 (§7)。
//!
//! 「何を、どのサイズで、どのアルゴリズムで出力するか」を宣言的に保持する。
//! デフォルトは「必要最小限のモダン構成」。詳細はオプトイン (§5.3)。

use std::path::PathBuf;

use crate::domain::resize_algorithm::ResizeAlgorithm;

/// 個別サイズに対するソース画像の差し替え指定。
#[derive(Debug, Clone)]
pub struct SizeOverride {
    pub size: u32,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ExportPlan {
    pub include_ico: bool,
    pub include_apple_touch: bool,
    /// 高解像度 PNG として出力するサイズ群。
    pub png_sizes: Vec<u32>,
    pub include_html_snippet: bool,
    pub algorithm: ResizeAlgorithm,
    pub overrides: Vec<SizeOverride>,
    /// ICO に内包するサイズ群。
    pub ico_sizes: Vec<u32>,
    /// `favicon.svg` を出力する (v1.2.0+)。
    /// SVG ソースなら入力をそのまま、 ラスタ入力なら `vectorize_on_raster` を見て判定。
    pub include_svg: bool,
    /// ラスタ入力 (PNG / WebP) に対して vtracer でベクトル化 SVG を生成するか
    /// (v1.2.0+)。 `false` の場合、 ラスタ入力時は `favicon.svg` 出力をスキップ
    /// する (HTML スニペットからも `<link type="image/svg+xml">` 行が省かれる)。
    pub vectorize_on_raster: bool,
}

impl Default for ExportPlan {
    fn default() -> Self {
        Self {
            include_ico: true,
            include_apple_touch: true,
            // モダンブラウザ向けの最小構成。詳細設定で増やせる。
            png_sizes: vec![32, 192, 512],
            include_html_snippet: true,
            algorithm: ResizeAlgorithm::default(),
            overrides: Vec::new(),
            // ICO は 16/32/48 を内包。各サイズを個別レンダリングして詰める。
            ico_sizes: vec![16, 32, 48],
            // SVG 出力は v1.2.0 のデフォルト。 高 DPI 画面で最美。
            include_svg: true,
            // ラスタ入力に対するベクトル化もデフォルトオン。
            // 写真など vtracer に向かない入力に対しては詳細設定でオフにする。
            vectorize_on_raster: true,
        }
    }
}

impl ExportPlan {
    /// この計画で**確定的に**生成される出力数 (プレビュー表示用)。
    ///
    /// 注: `include_svg` の真偽は実際の出力数を決定づけない。 ラスタ入力で
    /// `vectorize_on_raster=false` の場合、 SVG はスキップされる。 そのため
    /// 「最大数」を返す方針で `include_svg` も加算する。 実際の数は
    /// `services::exporter::run` の `ExportReport.artifacts.len()` で確認する。
    pub fn artifact_count(&self) -> usize {
        let ico = usize::from(self.include_ico);
        let apple = usize::from(self.include_apple_touch);
        let html = usize::from(self.include_html_snippet);
        let svg = usize::from(self.include_svg);
        ico + apple + html + svg + self.png_sizes.len()
    }
}
