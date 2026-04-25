//! 共通エラー型。
//!
//! - `Message` (logolig-app 側) で運ぶため `Clone + Send + 'static` を満たす。
//! - そのため `std::io::Error` などはここに直接持たず、文字列に正規化して保持する。
//! - エラーは UI 層で `Toast<Message>` 経由でユーザーに提示される。

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    /// 受け付けられないファイル形式。
    #[error("対応していないファイル形式です: {0}")]
    UnsupportedFile(String),

    /// ファイル I/O エラー。
    #[error("入出力エラー: {0}")]
    Io(String),

    /// ラスタ画像のデコード失敗。
    #[error("画像のデコードに失敗しました: {0}")]
    Decode(String),

    /// SVG のパース/ラスタライズ失敗。
    #[error("SVG のラスタライズに失敗しました: {0}")]
    Rasterize(String),

    /// リサイズ処理失敗。
    #[error("リサイズに失敗しました: {0}")]
    Resize(String),

    /// 出力 (ICO / PNG / HTML スニペット) の生成失敗。
    #[error("出力の生成に失敗しました: {0}")]
    Export(String),

    /// 未実装機能（段階的開発のためのプレースホルダ）。
    #[error("未実装: {0}")]
    NotImplemented(&'static str),
}
