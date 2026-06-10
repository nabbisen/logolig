//! コンテキストプレビュー設定 (§5.2)。
//!
//! プレビューは「画像を表示する」ことではなく
//! 「使われる文脈での見え方を確認する」ことが目的。
//!
//! ## v1.10.0: TransparencyChecker 昇格
//!
//! v1.7.0 で導入した `state.preview_checker: bool` を、 こちらの
//! `PreviewContext::TransparencyChecker` バリアントに統合。 これは:
//!
//! - 「View as: Browser tab / Phone home / Checker」 の 3 択を 1 つの enum で
//!   排他的に表現できる (型レベルで「タブ風 + チェッカー」 のような無意味な
//!   同時 ON 状態を排除)
//! - view 関数が `match preview_context` の 1 箇所で分岐できる
//! - 永続化対象外 (Checker は一時的な確認用 — アプリ再起動時に元に戻る)
//!
//! `background: ThemeMode` は引き続き独立軸として保持。 Checker のときは
//! 背景塗りより市松模様が優先されるが、 構造としては独立を維持して
//! 「Checker 表示中も次にタブ風に戻したときの背景設定を覚えている」 を
//! 可能にする。

use crate::domain::theme_mode::ThemeMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewContext {
    /// ブラウザタブを模した 16x16 相当の表示。
    BrowserTab16,
    /// スマートフォンのホーム画面アイコンを模した表示。
    SmartphoneIcon,
    /// 透過チェッカー表示 (v1.10.0)。 framing なし、 市松模様の上にアイコンを
    /// 実寸で重ねる。 v1.7 の `state.preview_checker: bool` から昇格。
    TransparencyChecker,
}

impl PreviewContext {
    pub fn label(self) -> &'static str {
        match self {
            Self::BrowserTab16 => "Browser tab (16×16)",
            Self::SmartphoneIcon => "Smartphone home",
            Self::TransparencyChecker => "Transparency checker",
        }
    }

    /// 全バリアントを順序付きで返す。 UI 層で「View as」 ボタン群の
    /// 並びを構築する用途。
    pub fn all() -> [Self; 3] {
        [
            Self::BrowserTab16,
            Self::SmartphoneIcon,
            Self::TransparencyChecker,
        ]
    }

    /// 背景 (`Surface`) 設定が意味を持つコンテキストか。
    /// Checker は市松模様で塗りつぶすため Surface 設定は無視される。 UI 層で
    /// 「Surface」 ボタン群を disabled / hidden にするかの判定に使う。
    pub fn respects_surface(self) -> bool {
        match self {
            Self::BrowserTab16 | Self::SmartphoneIcon => true,
            Self::TransparencyChecker => false,
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
