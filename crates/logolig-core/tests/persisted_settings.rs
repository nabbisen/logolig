//! 永続化された設定の round-trip + SettingsStore trait の挙動 (v1.4.0)。
//!
//! `InMemoryStore` を fixture として使い、 ファイル I/O 抜きで trait の
//! 振る舞いを検証する。 これは「v2 で BrowserStore が出ても同じ trait を
//! 満たすかどうか」 を見るための設計テストでもある。
//!
//! テスト対象:
//! - JSON への serialize / deserialize round-trip (フィールド欠落耐性も)
//! - `load_or_default()` の初回・既存読み出しの両ケース
//! - `update()` の read-modify-write が正しく作用すること
//! - `serde(default)` による前方互換 (将来フィールド追加時の保護)

use std::cell::RefCell;
use std::convert::Infallible;

use logolig_core::{
    ExportPlan, PersistedSettings, ResizeAlgorithm, SettingsStore, ThemeMode,
};
use serde::{de::DeserializeOwned, Serialize};

// ---------------------------------------------------------------------------
// テスト用フィクスチャ: メモリ上のストア
// ---------------------------------------------------------------------------

/// 単一の `Option<String>` (JSON) を保持するシンプルな store。
/// `RefCell` で内部可変性を持たせる。 並行アクセスは想定しない (テスト用)。
struct InMemoryStore {
    blob: RefCell<Option<String>>,
}

impl InMemoryStore {
    fn empty() -> Self {
        Self {
            blob: RefCell::new(None),
        }
    }
}

impl<T> SettingsStore<T> for InMemoryStore
where
    T: Serialize + DeserializeOwned + Default,
{
    type Error = Infallible;

    fn load_or_default(&self) -> Result<T, Self::Error> {
        // 取得は短い borrow に閉じる。 None 時の borrow_mut とぶつからないよう、
        // clone で取り出してから match。 RefCell の常套手段。
        let snapshot = self.blob.borrow().clone();
        match snapshot {
            Some(s) => Ok(serde_json::from_str(&s).expect("test JSON should parse")),
            None => {
                let d = T::default();
                let s = serde_json::to_string(&d).expect("default should serialize");
                *self.blob.borrow_mut() = Some(s);
                Ok(d)
            }
        }
    }

    fn save(&self, config: &T) -> Result<(), Self::Error> {
        let s = serde_json::to_string(config).expect("config should serialize");
        *self.blob.borrow_mut() = Some(s);
        Ok(())
    }

    fn update<F>(&self, f: F) -> Result<T, Self::Error>
    where
        F: FnOnce(&mut T),
    {
        let mut cfg = self.load_or_default()?;
        f(&mut cfg);
        self.save(&cfg)?;
        Ok(cfg)
    }
}

// ---------------------------------------------------------------------------
// 1. round-trip
// ---------------------------------------------------------------------------

#[test]
fn persisted_settings_round_trip_through_json() {
    let original = PersistedSettings {
        export_plan: {
            let mut p = ExportPlan::default();
            p.png_sizes = vec![32, 64, 192];
            p.algorithm = ResizeAlgorithm::Bilinear;
            p.include_svg = false;
            p
        },
        theme: ThemeMode::Dark,
        locale: Some("ja".to_string()),
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: PersistedSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.export_plan.png_sizes, vec![32, 64, 192]);
    assert_eq!(restored.export_plan.algorithm, ResizeAlgorithm::Bilinear);
    assert!(!restored.export_plan.include_svg);
    assert_eq!(restored.theme, ThemeMode::Dark);
    assert_eq!(restored.locale.as_deref(), Some("ja"));
}

// ---------------------------------------------------------------------------
// 2. load_or_default()
// ---------------------------------------------------------------------------

#[test]
fn load_or_default_returns_default_on_empty_store() {
    let store: InMemoryStore = InMemoryStore::empty();
    let loaded: PersistedSettings = store.load_or_default().unwrap();
    // ExportPlan のデフォルト値が返ってきていること
    assert_eq!(loaded.export_plan.png_sizes, vec![32, 192, 512]);
    assert_eq!(loaded.theme, ThemeMode::System);
    assert!(loaded.locale.is_none());

    // store には今 default が書き込まれているはず (load_or_default の副作用)。
    // 続く load は同じ値を返す。
    let again: PersistedSettings = store.load_or_default().unwrap();
    assert_eq!(again.export_plan.png_sizes, loaded.export_plan.png_sizes);
}

// ---------------------------------------------------------------------------
// 3. update()
// ---------------------------------------------------------------------------

#[test]
fn update_modifies_and_persists() {
    let store: InMemoryStore = InMemoryStore::empty();
    // 初回 load_or_default で default が書かれている前提で update を呼ぶ
    let _: PersistedSettings = store.load_or_default().unwrap();

    let after: PersistedSettings = store
        .update(|s: &mut PersistedSettings| {
            s.theme = ThemeMode::Light;
            s.export_plan.include_apple_touch = false;
        })
        .unwrap();
    assert_eq!(after.theme, ThemeMode::Light);
    assert!(!after.export_plan.include_apple_touch);

    // 直後の load_or_default で同じ値が読める
    let reloaded: PersistedSettings = store.load_or_default().unwrap();
    assert_eq!(reloaded.theme, ThemeMode::Light);
    assert!(!reloaded.export_plan.include_apple_touch);
}

// ---------------------------------------------------------------------------
// 4. 前方互換 (serde(default))
// ---------------------------------------------------------------------------

#[test]
fn missing_fields_are_filled_from_default() {
    // 古い v1.4.0-pre のような JSON を想定: export_plan のサブフィールドが
    // いくつか欠けている。 例えば v1.2.0 までは include_svg / vectorize_on_raster
    // が無かった。
    let legacy_json = r#"{
        "export_plan": {
            "include_ico": true,
            "png_sizes": [32, 192, 512]
        },
        "theme": "Dark"
    }"#;

    let restored: PersistedSettings = serde_json::from_str(legacy_json).unwrap();
    // 欠けていたフィールドは default で埋まる
    assert!(restored.export_plan.include_svg); // default = true
    assert!(restored.export_plan.vectorize_on_raster);
    assert_eq!(restored.theme, ThemeMode::Dark);
    assert!(restored.locale.is_none()); // 欠けている → None
}

#[test]
fn entirely_empty_json_object_yields_full_default() {
    let restored: PersistedSettings = serde_json::from_str("{}").unwrap();
    let default = PersistedSettings::default();
    assert_eq!(restored.export_plan.png_sizes, default.export_plan.png_sizes);
    assert_eq!(restored.theme, default.theme);
    assert_eq!(restored.locale, default.locale);
}
