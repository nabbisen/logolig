//! コンテキストプレビュー設定 (§5.2)。
//!
//! プレビューは「画像を表示する」ことではなく
//! 「使われる文脈での見え方を確認する」ことが目的。

use crate::domain::theme_mode::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewContext {
    /// ブラウザタブを模した 16x16 相当の表示。
    BrowserTab16,
    /// スマートフォンのホーム画面アイコンを模した表示。
    SmartphoneIcon,
}

impl PreviewContext {
    pub fn label(self) -> &'static str {
        match self {
            Self::BrowserTab16 => "Browser tab (16×16)",
            Self::SmartphoneIcon => "Smartphone home",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::BrowserTab16 => Self::SmartphoneIcon,
            Self::SmartphoneIcon => Self::BrowserTab16,
        }
    }
}

/// プレビュー設定。
///
/// `background` はプレビュー表示用であり、画像自体は変更しない (§5.2 後段)。
#[derive(Debug, Clone, Copy)]
pub struct PreviewProfile {
    pub context: PreviewContext,
    pub background: ThemeMode,
}

impl Default for PreviewProfile {
    fn default() -> Self {
        Self {
            context: PreviewContext::BrowserTab16,
            background: ThemeMode::System,
        }
    }
}
