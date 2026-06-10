//! `SettingsStore` のネイティブ実装 (v1.4.0)。
//!
//! `app-json-settings` v2 の `ConfigManager<T>` を薄くラップし、 logolig-core の
//! `SettingsStore` trait を満たす。
//!
//! ## なぜラップするのか
//!
//! `ConfigManager` を直接使えば動作はする。 しかし「保存方式は内部実装、
//! UI コードは trait 経由でしか触らない」 という規律を持ち込むことで、
//! v2 で `BrowserStore` (LocalStorage 実装) に差し替える時の影響範囲が
//! `native_store.rs` 1 ファイルに閉じる。
//!
//! ## なぜ ConfigError を newtype で包むのか
//!
//! v2.0.2 の `app_json_settings::ConfigError` は `Debug` derive のみで
//! `Display` も `std::error::Error` も実装していない。 logolig-core の
//! `SettingsStore::Error` は `StdError + Send + Sync + 'static` を要求するため、
//! ここで newtype `NativeStoreError` を挟んで両者を実装する。
//!
//! ## 保存場所とファイル名
//!
//! - 場所: OS 標準の config dir / `logolig` (バイナリ名から自動解決)
//!   - Linux:   `$XDG_CONFIG_HOME/logolig/settings.json`
//!     (or `~/.config/logolig/settings.json`)
//!   - macOS:   `~/Library/Application Support/logolig/settings.json`
//!   - Windows: `%APPDATA%/logolig/settings.json`
//! - ファイル名: `settings.json` (default `config.json` を上書き; 「設定」 とい
//!   う意味を明示する一般的な命名)
//! - フォーマット: pretty JSON (default)。 ユーザが手で編集して読みやすい

use std::fmt;

use app_json_settings::{ConfigError, ConfigManager};

use logolig_core::{PersistedSettings, SettingsStore};

/// ネイティブ環境向けの SettingsStore。
pub struct NativeStore {
    inner: ConfigManager<PersistedSettings>,
}

impl std::fmt::Debug for NativeStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // ConfigManager 自体は Debug を実装していないため、 path を出すことで
        // 「どこに保存しているか」 だけ可視化する。
        f.debug_struct("NativeStore")
            .field("path", &self.inner.path())
            .finish()
    }
}

impl NativeStore {
    /// 標準の場所 (OS config dir / logolig / settings.json) で初期化する。
    pub fn new() -> Self {
        Self {
            inner: ConfigManager::new().with_filename("settings.json"),
        }
    }
}

impl SettingsStore<PersistedSettings> for NativeStore {
    type Error = NativeStoreError;

    fn load_or_default(&self) -> Result<PersistedSettings, Self::Error> {
        self.inner.load_or_default().map_err(NativeStoreError)
    }

    fn save(&self, config: &PersistedSettings) -> Result<(), Self::Error> {
        self.inner.save(config).map_err(NativeStoreError)
    }

    fn update<F>(&self, f: F) -> Result<PersistedSettings, Self::Error>
    where
        F: FnOnce(&mut PersistedSettings),
    {
        self.inner.update(f).map_err(NativeStoreError)
    }
}

/// `app_json_settings::ConfigError` のラッパ。 `Display` と `std::error::Error`
/// を提供する。
#[derive(Debug)]
pub struct NativeStoreError(pub ConfigError);

impl fmt::Display for NativeStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            ConfigError::Io(e) => write!(f, "settings I/O error: {e}"),
            ConfigError::Serialize(e) => write!(f, "settings serialize error: {e}"),
            ConfigError::Deserialize(e) => write!(f, "settings deserialize error: {e}"),
        }
    }
}

impl std::error::Error for NativeStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.0 {
            ConfigError::Io(e) => Some(e),
            ConfigError::Serialize(e) | ConfigError::Deserialize(e) => Some(e),
        }
    }
}
