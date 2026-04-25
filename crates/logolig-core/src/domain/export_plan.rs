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
        }
    }
}

impl ExportPlan {
    /// この計画で生成される出力数（プレビュー表示用）。
    pub fn artifact_count(&self) -> usize {
        let ico = usize::from(self.include_ico);
        let apple = usize::from(self.include_apple_touch);
        let html = usize::from(self.include_html_snippet);
        ico + apple + html + self.png_sizes.len()
    }
}
