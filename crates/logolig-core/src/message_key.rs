//! 翻訳キー (v1.5.0)。
//!
//! UI 文言とエラーメッセージのすべてをこの enum で参照する。 `logolig-i18n`
//! クレートが各ロケールの辞書を持ち、 `MessageKey` を引数に取って文字列を返す。
//!
//! ## なぜ enum か
//!
//! 文字列キー (`"error.io"` のような形式) でも実装は可能だが、 enum にする
//! ことで:
//!
//! 1. **網羅性チェック**: 翻訳辞書側で `match key { ... }` を書けば、 core が
//!    新キーを追加した瞬間に翻訳側がコンパイルエラー。 翻訳の追従漏れが
//!    型レベルで検出される
//! 2. **リファクタ安全性**: `ErrorIo` を `ErrorReadFailed` に改名すると IDE
//!    一発で全箇所更新される
//! 3. **使われていないキー検出**: `dead_code` 警告で発見可能
//!
//! ## なぜ logolig-core 配置か
//!
//! `AppError::key()` が `MessageKey` を返す責務を core が持つため。 もし
//! logolig-i18n に置くと、 core が i18n に依存してしまい依存方向が逆になる。
//!
//! ## カテゴリ
//!
//! 列挙の構造は flat。 「app.title」 のようなネスト名前空間は採らず、
//! `AppTitle` のように prefix で表現する。 enum 1 つに全文言が並ぶことで、
//! 翻訳作業時に「全 N 個のキー」 を一望できる。

use serde::{Deserialize, Serialize};

/// UI 文言・エラーメッセージのすべてを表すキー。
///
/// 新しい文言を UI に追加する時は:
/// 1. ここにバリアントを追加
/// 2. 各 locale 辞書 (en.toml など) にキー追加
/// 3. `Translator` の網羅性 match がコンパイルエラーになるので埋める
///
/// この 3 ステップを型レベルで強制することで、 翻訳が漏れない仕組みになっている。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageKey {
    // --- アプリ全体 ---
    /// アプリのタイトル ("Logolig")。
    AppTitle,

    // --- ドロップゾーン ---
    DropZoneInstruction,
    DropZoneSecondary,
    DropZoneAcceptedFormats,
    ChooseFileButton,
    ChangeFileButton,
    ImportingMessage,

    // --- プレビュー画面 ---
    PreviewBrowserTab,
    PreviewSmartphoneHome,
    PreviewBackgroundLight,
    PreviewBackgroundDark,
    PreviewBackgroundSystem,
    PreviewSourceLabel,
    PreviewNoSource,

    // --- 詳細設定ドロワー ---
    AdvancedTitle,
    AdvancedBlurb,
    SectionResize,
    SectionResizeBlurb,
    AlgorithmLabel,
    SectionSvg,
    SectionSvgBlurb,
    IncludeSvgLabel,
    VectorizeOnRasterLabel,
    PresetLabel,
    SectionFiles,
    SectionFilesBlurb,
    IncludeIcoLabel,
    IncludeAppleTouchLabel,
    IncludeHtmlSnippetLabel,
    SectionPngSizes,
    SectionPngSizesBlurb,
    SectionIcoSizes,
    SectionIcoSizesBlurb,
    SizeAddButton,
    SizeChipRemove,
    SizeInputPlaceholder,
    EmptySetLabel,
    ResetButton,
    CloseButton,

    // --- ボタン・操作 ---
    ExportButton,
    ToggleAdvancedButton,
    ToggleThemeButton,

    // --- リサイズアルゴリズム名 ---
    AlgorithmLanczos3,
    AlgorithmMitchellNetravali,
    AlgorithmCatmullRom,
    AlgorithmBilinear,
    AlgorithmNearest,

    // --- vtracer プリセット名 ---
    VtracerPresetSharp,
    VtracerPresetDefault,
    VtracerPresetPhotoRich,

    // --- 言語選択 (v1.5.0) ---
    SectionLanguage,
    SectionLanguageBlurb,
    LanguageEnglish,
    LanguageJapanese,
    LanguageSystemDefault,

    // --- Toast タイトル / 内容 ---
    ToastExportTitle,
    ToastExportBody,
    ToastResetTitle,
    ToastResetBody,
    ToastSettingsLoadFailedTitle,
    ToastSettingsLoadFailedBody,
    ToastSettingsSaveFailedTitle,
    ToastSettingsSaveFailedBody,
    ToastSizeAlreadyInSetTitle,
    ToastPngSizeAlreadyInSetBody,
    ToastIcoSizeAlreadyInSetBody,
    ToastInvalidSizeTitle,
    ToastInvalidSizeBody,
    ToastSizeOutOfRangeTitle,
    ToastPngSizeOutOfRangeBody,
    ToastIcoSizeOutOfRangeBody,

    // --- エラー (AppError キー化) ---
    ErrorUnsupportedFile,
    ErrorIo,
    ErrorDecode,
    ErrorRasterize,
    ErrorResize,
    ErrorExport,
    ErrorNotImplemented,
}
