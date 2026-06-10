//! 共通エラー型 (v1.5.0 でキー化)。
//!
//! - `Message` (logolig-app 側) で運ぶため `Clone + Send + 'static` を満たす。
//! - そのため `std::io::Error` などはここに直接持たず、文字列に正規化して保持する。
//! - エラーは UI 層で `Toast<Message>` 経由でユーザーに提示される。
//!
//! ## v1.5.0: i18n キー化
//!
//! `Display` 実装は **英語固定** で残す。 これは:
//! - ログ出力 (環境固有の言語に依存しないログを残せる)
//! - デバッグ表示 (`{:?}` などで使われる)
//! - i18n 失敗時のフォールバック表示
//!
//! UI で表示する翻訳済み文言は `key()` と `args()` を `Translator` に渡して
//! 取得する。 詳しくは `logolig_i18n::Translator::translate_error` を参照。
//!
//! 各バリアントは **構造化された引数** を持つ。 例えば `Io { path, source }`
//! は path と source を別フィールドで保持し、 翻訳テンプレートで `{path}` と
//! `{source}` を使えるようにしている。

use thiserror::Error;

use crate::message_key::MessageKey;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    /// 受け付けられないファイル形式。
    #[error("unsupported file format: {path}")]
    UnsupportedFile { path: String },

    /// ファイル I/O エラー。
    #[error("I/O error on {path}: {cause}")]
    Io { path: String, cause: String },

    /// ラスタ画像のデコード失敗。
    #[error("image decode failed: {detail}")]
    Decode { detail: String },

    /// SVG のパース/ラスタライズ失敗。
    #[error("SVG rasterize failed: {detail}")]
    Rasterize { detail: String },

    /// リサイズ処理失敗。
    #[error("resize failed: {detail}")]
    Resize { detail: String },

    /// 出力 (ICO / PNG / HTML スニペット) の生成失敗。
    #[error("export failed: {detail}")]
    Export { detail: String },

    /// 未実装機能(段階的開発のためのプレースホルダ)。
    #[error("not implemented: {feature}")]
    NotImplemented { feature: &'static str },
}

impl AppError {
    /// 翻訳キー。
    ///
    /// `logolig_i18n::Translator` がこのキーで対応する翻訳テンプレートを引き出す。
    pub fn key(&self) -> MessageKey {
        match self {
            Self::UnsupportedFile { .. } => MessageKey::ErrorUnsupportedFile,
            Self::Io { .. } => MessageKey::ErrorIo,
            Self::Decode { .. } => MessageKey::ErrorDecode,
            Self::Rasterize { .. } => MessageKey::ErrorRasterize,
            Self::Resize { .. } => MessageKey::ErrorResize,
            Self::Export { .. } => MessageKey::ErrorExport,
            Self::NotImplemented { .. } => MessageKey::ErrorNotImplemented,
        }
    }

    /// 翻訳テンプレートに差し込む引数。
    ///
    /// テンプレート側で `{path}` / `{source}` / `{detail}` / `{feature}` の
    /// プレースホルダを使うと、 ここで返した値で置換される。
    pub fn args(&self) -> Vec<(&'static str, String)> {
        match self {
            Self::UnsupportedFile { path } => vec![("path", path.clone())],
            Self::Io { path, cause } => {
                vec![("path", path.clone()), ("cause", cause.clone())]
            }
            Self::Decode { detail } => vec![("detail", detail.clone())],
            Self::Rasterize { detail } => vec![("detail", detail.clone())],
            Self::Resize { detail } => vec![("detail", detail.clone())],
            Self::Export { detail } => vec![("detail", detail.clone())],
            Self::NotImplemented { feature } => vec![("feature", feature.to_string())],
        }
    }

    // ---------------------------------------------------------------
    // 旧 API (タプルバリアント時代) の呼び出し箇所をスムーズに移行する
    // ためのコンストラクタヘルパ。 v1.4.x までの呼び出しサイトはほとんど
    // `AppError::Io(format!(...))` のような形だった。 これらを最小変更で
    // 新形式に揃えるためのファクトリ関数群。
    // ---------------------------------------------------------------

    pub fn unsupported_file(path: impl Into<String>) -> Self {
        Self::UnsupportedFile { path: path.into() }
    }

    pub fn io(path: impl Into<String>, cause: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            cause: cause.into(),
        }
    }

    pub fn decode(detail: impl Into<String>) -> Self {
        Self::Decode {
            detail: detail.into(),
        }
    }

    pub fn rasterize(detail: impl Into<String>) -> Self {
        Self::Rasterize {
            detail: detail.into(),
        }
    }

    pub fn resize(detail: impl Into<String>) -> Self {
        Self::Resize {
            detail: detail.into(),
        }
    }

    pub fn export(detail: impl Into<String>) -> Self {
        Self::Export {
            detail: detail.into(),
        }
    }

    pub fn not_implemented(feature: &'static str) -> Self {
        Self::NotImplemented { feature }
    }
}
