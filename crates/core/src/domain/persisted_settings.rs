//! Persisted settings (v1.4.0).
//!
//! Loaded on startup via `SettingsStore::load_or_default()` and saved
//! immediately whenever a user action changes a value.
//!
//! ## What is persisted
//!
//! - `export_plan`: full output plan (PNG/ICO sizes, algorithm, toggles)
//! - `theme`:       theme mode selection (System / Light / Dark)
//! - `locale`:      optional locale override; `Some("ja")` overrides the
//!   OS locale. Unused in v1.4 but wired up in v1.5+.
//!
//! ## What is NOT persisted
//!
//! UI-only state (`advanced_open`, active previews, in-flight tasks) is
//! ephemeral and intentionally excluded. Restarting the app restores
//! settings but starts fresh on screen state.

use serde::{Deserialize, Serialize};

use crate::domain::{ExportPlan, ThemeMode};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PersistedSettings {
    /// Output plan (§7). Includes PNG/ICO sizes, algorithm selection, etc.
    pub export_plan: ExportPlan,
    /// Theme mode.
    pub theme: ThemeMode,
    /// Locale override (used by i18n since v1.5).
    /// `None` follows the OS-detected locale.
    /// Value is an IETF BCP-47-style tag such as `"en"` or `"ja"`.
    pub locale: Option<String>,
}
