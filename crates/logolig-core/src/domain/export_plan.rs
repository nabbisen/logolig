//! 出力計画 (§7)。
//!
//! 「何を、どのサイズで、どのアルゴリズムで出力するか」を宣言的に保持する。
//! デフォルトは「必要最小限のモダン構成」。詳細はオプトイン (§5.3)。
//!
//! v1.4.0 から `Serialize` / `Deserialize` を実装し、 `PersistedSettings` の
//! 一部としてディスクに保存可能。 既知のフィールドが欠ける旧バージョン JSON
//! に出会った時のための `serde(default)` を全フィールドに付ける (前方互換)。

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::resize_algorithm::ResizeAlgorithm;
use crate::domain::vtracer_preset::VtracerPreset;

/// PNG サイズの実用下限 (px)。 これ以下は描画破綻する。
pub const PNG_SIZE_MIN: u32 = 16;
/// PNG サイズの実用上限 (px)。 これ以上はファイルサイズが過大で favicon 用途で意味がない。
pub const PNG_SIZE_MAX: u32 = 1024;

/// ICO サイズの実用下限 (px)。
pub const ICO_SIZE_MIN: u32 = 16;
/// ICO サイズの実用上限 (px)。
///
/// 256 は ICO のフォーマット仕様上の上限である (BMP モードの寸法フィールドが
/// `u8` であり、 256 は `0` で表現される慣習)。 ico crate もこれを尊重する。
pub const ICO_SIZE_MAX: u32 = 256;

/// 個別サイズに対するソース画像の差し替え指定。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeOverride {
    pub size: u32,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
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
    /// vtracer のチューニングプリセット (v1.4.1+)。
    /// `Sharp` はロゴ・アイコン向け、 `Default` は v1.2.0 と同じ既定値、
    /// `PhotoRich` は写真風 / グラデーション向け。
    pub vtracer_preset: VtracerPreset,
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
            // vtracer プリセットは v1.2.0 互換として Default を採用 (vtracer 既定値)。
            // Sharp / PhotoRich はユーザがオプトインで選択する。
            vtracer_preset: VtracerPreset::Default,
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

    /// PNG サイズ集合への追加 (v1.3.0)。 重複や範囲外を弾き、 昇順を保つ。
    /// 戻り値は **追加されたかどうか**: 既に存在する / 範囲外なら `false`。
    pub fn add_png_size(&mut self, size: u32) -> bool {
        Self::add_into_sorted_set(&mut self.png_sizes, size, PNG_SIZE_MIN, PNG_SIZE_MAX)
    }

    /// PNG サイズの削除 (v1.3.0)。 戻り値は **削除されたかどうか**。
    pub fn remove_png_size(&mut self, size: u32) -> bool {
        Self::remove_from_set(&mut self.png_sizes, size)
    }

    /// ICO サイズ集合への追加 (v1.3.0)。
    pub fn add_ico_size(&mut self, size: u32) -> bool {
        Self::add_into_sorted_set(&mut self.ico_sizes, size, ICO_SIZE_MIN, ICO_SIZE_MAX)
    }

    /// ICO サイズの削除 (v1.3.0)。
    /// `ico_sizes` を空にすることは許容する (`include_ico=false` 相当の運用)。
    pub fn remove_ico_size(&mut self, size: u32) -> bool {
        Self::remove_from_set(&mut self.ico_sizes, size)
    }

    fn add_into_sorted_set(set: &mut Vec<u32>, size: u32, min: u32, max: u32) -> bool {
        if size < min || size > max {
            return false;
        }
        if set.contains(&size) {
            return false;
        }
        set.push(size);
        set.sort_unstable();
        true
    }

    fn remove_from_set(set: &mut Vec<u32>, size: u32) -> bool {
        if let Some(pos) = set.iter().position(|s| *s == size) {
            set.remove(pos);
            true
        } else {
            false
        }
    }
}
