//! 読み込んだソース画像の不変表現 (§6.4 非破壊性)。

use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Png,
    Svg,
    /// 静的 WebP (v1.1.0+)。 アニメーション WebP は最初のフレームのみ扱う。
    Webp,
    /// JPEG (v1.11.0+)。 favicon 用途では背景透過を扱えないため、 入力後に
    /// 教育的な警告 Toast を表示して PNG への変換を促す (`app::main_panel`
    /// の `push_jpeg_warning` 参照)。
    Jpeg,
}

impl SourceKind {
    /// 拡張子から判定。判定不能なら `None`。
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "png" => Some(Self::Png),
            "svg" => Some(Self::Svg),
            "webp" => Some(Self::Webp),
            // JPEG は `.jpg` と `.jpeg` の 2 通りある。 OS / 慣用で両方使われる。
            "jpg" | "jpeg" => Some(Self::Jpeg),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Png => "PNG",
            Self::Svg => "SVG",
            Self::Webp => "WebP",
            Self::Jpeg => "JPEG",
        }
    }
}

/// 読み込み済みソース画像。
///
/// `Message` 経由で UI スレッドへ運ばれることを想定し、
/// 内部バッファは `Arc<[u8]>` でクローン安価かつ不変に保持する。
#[derive(Debug, Clone)]
pub struct SourceAsset {
    /// 元ファイルパス。表示・再読み込みに用いる。
    pub path: PathBuf,
    /// 種別。
    pub kind: SourceKind,
    /// 元データ（PNG ならデコード前バイト列、SVG なら UTF-8 ソース）。
    pub raw: Arc<[u8]>,
    /// PNG なら論理ピクセルサイズ。SVG なら viewBox 由来のヒント。
    pub intrinsic_size: Option<(u32, u32)>,
}

impl SourceAsset {
    pub fn display_name(&self) -> String {
        self.path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "<unnamed>".into())
    }
}
