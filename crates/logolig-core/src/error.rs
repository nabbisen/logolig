//! Shared error type (i18n-keyed since v1.5.0).
//!
//! - Must satisfy `Clone + Send + 'static` to be carried in `Message` (logolig-app).
//! - Therefore `std::io::Error` etc. are not stored directly; they are normalised to strings.
//! - Errors are presented to the user via `Toast<Message>` in the UI layer.
//!
//! ## v1.5.0: i18n keying
//!
//! `Display` is kept as **English-only**. It is used for:
//! - Log output (locale-independent)
//! - Debug formatting (`{:?}` etc.)
//! - Fallback display when i18n fails
//!
//! Translated text for the UI is obtained by passing `key()` and `args()` to
//! `Translator`. See `logolig_i18n::Translator::translate_error` for details.
//!
//! Each variant carries **structured arguments**. For example `Io { path, cause }`
//! stores path and cause as separate fields so that the translation template can
//! use `{path}` and `{cause}` as placeholders.

use thiserror::Error;

use crate::message_key::MessageKey;

#[derive(Debug, Clone, Error)]
pub enum AppError {
    /// Unsupported file format.
    #[error("unsupported file format: {path}")]
    UnsupportedFile { path: String },

    /// File I/O error.
    #[error("I/O error on {path}: {cause}")]
    Io { path: String, cause: String },

    /// Raster image decode failure.
    #[error("image decode failed: {detail}")]
    Decode { detail: String },

    /// SVG parse / rasterise failure.
    #[error("SVG rasterize failed: {detail}")]
    Rasterize { detail: String },

    /// Resize failure.
    #[error("resize failed: {detail}")]
    Resize { detail: String },

    /// Output generation failure (ICO / PNG / HTML snippet).
    #[error("export failed: {detail}")]
    Export { detail: String },

    /// Unimplemented feature (placeholder for incremental development).
    #[error("not implemented: {feature}")]
    NotImplemented { feature: &'static str },
}

impl AppError {
    /// i18n key.
    ///
    /// `logolig_i18n::Translator` uses this key to look up the matching translation template.
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

    /// Arguments inserted into the translation template.
    ///
    /// The template may use `{path}` / `{cause}` / `{detail}` / `{feature}`
    /// as placeholders; they are replaced with the values returned here.
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
    // Constructor helpers for smooth migration from the old tuple-variant API
    // (pre-v1.5.0). Call sites used to write `AppError::Io(format!(...))`;
    // these factory functions let those sites be updated with minimal diff.
    //
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
