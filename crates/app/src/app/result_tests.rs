//! `update` tests: leaving the result screen and showing the last result again.

use super::*;

use super::test_support::{dummy_result_assets, dummy_source_asset};
use crate::result::ResultAssets;

#[test]
fn edit_cancelled_preserves_last_result_history() {
    let mut state = AppState {
        screen: Screen::Result,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        result_preview_open: true,
        file_drag_hovering: true,
        ..AppState::default()
    };

    let _ = update(&mut state, Message::EditCancelled);

    assert_eq!(state.screen, Screen::Empty);
    assert!(state.source_asset.is_some());
    assert!(state.result_assets.is_some());
    assert!(state.preview.is_none());
    assert!(state.preview_cache.is_none());
    assert!(state.transparency.is_none());
    assert!(!state.result_preview_open);
    assert!(!state.file_drag_hovering);
}

#[test]
fn show_last_result_requested_reopens_result_without_reconversion() {
    let mut state = AppState {
        screen: Screen::Empty,
        source_asset: Some(dummy_source_asset()),
        result_assets: Some(dummy_result_assets()),
        ..AppState::default()
    };

    let _ = update(&mut state, Message::ShowLastResultRequested);

    assert_eq!(state.screen, Screen::Result);
    assert_eq!(
        state.result_assets.as_ref().map(ResultAssets::count),
        Some(1)
    );
    assert_eq!(
        state
            .source_asset
            .as_ref()
            .map(|asset| asset.display_name()),
        Some("logo.png".to_string())
    );
}
