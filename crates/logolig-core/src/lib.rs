//! # logolig-core
//!
//! Local-first favicon generator の純粋ロジック層。
//!
//! このクレートは **iced や snora に依存しない**。
//! それは慣習ではなく依存グラフ上の事実であり、これにより:
//! - 画像処理ロジックを GUI 抜きで単体テストできる
//! - 将来 CLI バイナリや WASM フロントエンドから再利用できる
//! - GUI イテレーション中にここがリビルドされない
//!
//! が達成される (§3.1, §16)。

pub mod domain;
pub mod error;
pub mod services;
pub mod settings;

// 上位クレート (logolig-app) から `use logolig_core::SourceAsset;` のように
// 短く参照できるよう、頻出型はクレート直下に re-export する。
pub use domain::{
    ExportPlan, PersistedSettings, PreviewContext, PreviewProfile, ResizeAlgorithm, Rgba8,
    SizeOverride, SourceAsset, SourceKind, ThemeMode, ICO_SIZE_MAX, ICO_SIZE_MIN, PNG_SIZE_MAX,
    PNG_SIZE_MIN,
};
pub use error::AppError;
pub use services::exporter::ExportReport;
pub use services::preview::PreviewCache;
pub use settings::SettingsStore;
