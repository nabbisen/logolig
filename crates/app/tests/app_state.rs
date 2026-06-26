//! Smoke tests for `AppState::default()`.
//!
//! Behavioural tests for `update()` are in Step 2+.
//!
//! logolig-app is a binary crate; internal types (AppState, Screen) are
//! not directly visible from tests/. Step 1 tests are therefore limited
//! to properties that can be verified through logolig (core).

// logolig-app is a binary crate; its internal types (AppState, Screen)
// are not directly visible from tests/.
//
// Step 1 tests are therefore limited to properties verifiable through logolig (core).
// State tests for the app layer will be added in Step 2 after exposing the necessary types.

use logolig::{ExportPlan, ResizeAlgorithm, ThemeMode};

#[test]
fn defaults_match_spec() {
    // Verify that the defaults depended on by AppState::default()
    // are consistent with spec §4.2 / §5.3 / §6.2 / §7.1.
    assert_eq!(ThemeMode::default(), ThemeMode::System);
    assert_eq!(ResizeAlgorithm::default(), ResizeAlgorithm::Lanczos3);

    let plan = ExportPlan::default();
    assert!(plan.include_ico);
    assert!(plan.include_apple_touch);
    assert!(plan.include_html_snippet);
    assert!(plan.overrides.is_empty());
}
