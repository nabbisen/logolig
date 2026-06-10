//! v1.16.0 — メモリ上に保持する変換結果アセット束。
//!
//! 旧モデル (v1.15 まで):
//! ```text
//! ファイル投入 → Preview 確認 → ディレクトリ選択 → 一括書き出し
//! ```
//!
//! 新モデル (v1.16):
//! ```text
//! ファイル投入 → 自動変換 (メモリ完結) → Result 画面で個別 DL or ZIP DL
//! ```
//!
//! Result 画面では各アセットがカードとして並び、 ユーザは:
//! - 個別の DL ボタンで「単一ファイル」 をファイルダイアログで保存
//! - 「すべてダウンロード (ZIP)」 ボタンで全アセットを zip に固めて保存
//!
//! このため AppState は変換が終わった全アセットをメモリ上に保持する必要が
//! ある。 favicon 一式の合計は通常 1 MB 未満なので、 メモリ保持は実質コスト
//! ゼロ。
//!
//! ## 設計判断
//!
//! - 各アセットは「ファイル名 + バイト列 + 表示用メタ情報」 のセットで保持。
//!   メタ情報には: 種別 (PNG / ICO / SVG / HTML / Webmanifest)、 ピクセル寸法
//!   (画像系のみ)、 概算サイズ表示 (`46 KB` 等)、 サムネ用ラスタ画像 (画像
//!   系のみ、 表示プレビュー用に既に decode 済みのもの)。
//!
//! - 表示順序は Vec の順序で固定 (favicon.ico → apple-touch → favicon-16 →
//!   ... → snippet → manifest)。 export plan で OFF にされた成果物は単に
//!   含まれない。
//!
//! - サムネは「カードに表示するための小さな decode 済みラスタ」 で、
//!   PreviewCache とは別物 (PreviewCache は元画像の固定サイズキャッシュ)。
//!
//! ## v1.16 phase A の段階性
//!
//! 本モジュールは v1.16.0 phase A では型定義のみ。 実際にこの構造体を埋め
//! ->表示する流れは phase B で実装する (Converting 完了時に変換結果をここ
//! に詰めて Screen::Result に遷移)。 そのため現状は dead code として扱い、
//! `#[allow(dead_code)]` を付与してコンパイラ警告を抑止している。

#![allow(dead_code)]

use logolig_core::Rgba8;

/// 変換結果一式 (メモリ上保持)。
#[derive(Debug, Clone)]
pub struct ResultAssets {
    /// 個々のアセット (表示順)。
    pub items: Vec<ResultAssetItem>,
}

impl ResultAssets {
    /// 全アセットの合計バイト数 (UI 表示用)。
    pub fn total_bytes(&self) -> usize {
        self.items.iter().map(|i| i.bytes.len()).sum()
    }

    /// アセット件数。
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// 個別アセット 1 件分。
#[derive(Debug, Clone)]
pub struct ResultAssetItem {
    /// 出力ファイル名 (例: `favicon.ico`、 `favicon-16.png`、 `manifest.webmanifest`)。
    pub file_name: String,
    /// 出力バイト列 (これが個別 DL / ZIP DL の中身)。
    pub bytes: Vec<u8>,
    /// 種別。 カード表示の判別に使う。
    pub kind: ResultAssetKind,
    /// 画像系アセットの寸法 (例: `(16, 16)`)。 テキスト系は None。
    pub dimensions: Option<(u32, u32)>,
    /// カード上のサムネ用にあらかじめ decode 済みの小さなラスタ。
    /// 画像系のみ Some。 テキスト系 (snippet / manifest) は None でアイコン表示。
    pub thumbnail: Option<Rgba8>,
}

impl ResultAssetItem {
    /// 「46 KB」 「1.2 KB」 のような human-readable な size 表示。
    pub fn size_display(&self) -> String {
        let bytes = self.bytes.len();
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 100 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 {
            format!("{} KB", bytes / 1024)
        } else {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    /// 寸法 (幅×高さ) の表示文字列。 画像系のみ意味を持つ。
    pub fn dimensions_display(&self) -> Option<String> {
        self.dimensions.map(|(w, h)| format!("{} × {}", w, h))
    }
}

/// アセットの種別。
///
/// カードのサムネ表示で「画像のラスタを直接表示する」 か「文書アイコンで
/// プレースホルダ」 かを切り替えるために使う。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultAssetKind {
    /// PNG (各サイズの favicon-N.png、 apple-touch-icon.png)
    Png,
    /// ICO (favicon.ico、 マルチサイズの可能性あり)
    Ico,
    /// SVG (favicon.svg、 元 SVG をそのまま or vtracer 経由)
    Svg,
    /// PNG monochrome (mono サブディレクトリ相当)
    PngMono,
    /// HTML snippet (favicon-snippet.html)
    HtmlSnippet,
    /// Web manifest (manifest.webmanifest)
    WebManifest,
}

impl ResultAssetKind {
    /// カードのバッジに表示するラベル。
    pub fn badge_label(self) -> &'static str {
        match self {
            ResultAssetKind::Png => "PNG",
            ResultAssetKind::Ico => "ICO",
            ResultAssetKind::Svg => "SVG",
            ResultAssetKind::PngMono => "PNG mono",
            ResultAssetKind::HtmlSnippet => "HTML",
            ResultAssetKind::WebManifest => "JSON",
        }
    }

    /// サムネにラスタ画像を描けるか (= 画像系か)。
    pub fn has_visual_thumbnail(self) -> bool {
        matches!(
            self,
            ResultAssetKind::Png
                | ResultAssetKind::Ico
                | ResultAssetKind::Svg
                | ResultAssetKind::PngMono
        )
    }
}
