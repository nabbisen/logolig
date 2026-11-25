//! `update` tests of the Interface size setting (RFC 034, job J24).

use super::*;

use logolig::InterfaceSize;

/// The lines of the settings file's snapshot that differ between `before` and `after` (the
/// pretty-printed debug form, one leaf per line): the ones only in `before`, the ones only in `after`.
fn changed(
    before: &logolig::PersistedSettings,
    after: &logolig::PersistedSettings,
) -> (Vec<String>, Vec<String>) {
    let lines = |s: &logolig::PersistedSettings| -> Vec<String> {
        format!("{s:#?}")
            .lines()
            .map(|l| l.trim().to_string())
            .collect()
    };
    let (b, a) = (lines(before), lines(after));
    (
        b.iter().filter(|l| !a.contains(l)).cloned().collect(),
        a.iter().filter(|l| !b.contains(l)).cloned().collect(),
    )
}

#[test]
fn the_factor_iced_gets_is_the_chosen_step() {
    let mut state = AppState::default();
    assert_eq!(scale_for(&state), 1.0, "the default is Normal");
    for (step, factor) in [
        (InterfaceSize::Normal, 1.0),
        (InterfaceSize::Large, 1.25),
        (InterfaceSize::ExtraLarge, 1.5),
    ] {
        state.interface_size = step;
        assert_eq!(scale_for(&state), factor, "{step:?}");
    }
}

#[test]
fn choosing_a_step_stores_it_and_changes_the_factor() {
    let mut state = AppState::default();
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Large),
    );
    assert_eq!(state.interface_size, InterfaceSize::Large);
    assert_eq!(scale_for(&state), 1.25);
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::ExtraLarge),
    );
    assert_eq!(scale_for(&state), 1.5);
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Normal),
    );
    assert_eq!(scale_for(&state), 1.0);
}

/// The settings file changes exactly its own leaf (RFC 011's rule): the interface size, and nothing
/// else; and the snapshot carries it, so the next start reads it back.
#[test]
fn choosing_a_step_saves_only_its_leaf() {
    let mut state = AppState::default();
    let before = snapshot_persisted(&state);
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::ExtraLarge),
    );
    let after = snapshot_persisted(&state);
    assert_eq!(after.interface_size, InterfaceSize::ExtraLarge);
    let (removed, added) = changed(&before, &after);
    assert_eq!(removed, ["interface_size: Normal,"]);
    assert_eq!(added, ["interface_size: ExtraLarge,"]);
}

/// Other settings do not touch it: a theme change leaves the interface size where it is.
#[test]
fn other_settings_leave_the_interface_size_alone() {
    let mut state = AppState {
        interface_size: InterfaceSize::Large,
        ..AppState::default()
    };
    let _ = update(&mut state, Message::ThemePicked(ThemeMode::Dark));
    assert_eq!(
        snapshot_persisted(&state).interface_size,
        InterfaceSize::Large
    );
    let _ = update(&mut state, Message::ExportPlanResetRequested);
    assert_eq!(
        state.interface_size,
        InterfaceSize::Large,
        "'Reset to defaults' resets the export plan only"
    );
}

/// A change asks the window to hold its minimum size again (the logical minimum is converted with
/// the scale in force at the moment of the request); the same step again asks for nothing.
#[test]
fn a_change_reissues_the_minimum_window_size_and_the_same_step_does_not() {
    let mut state = AppState::default();
    for step in [
        InterfaceSize::Large,
        InterfaceSize::ExtraLarge,
        InterfaceSize::Normal,
    ] {
        let task = update(&mut state, Message::InterfaceSizeChosen(step));
        assert!(
            task.units() > 0,
            "{step:?}: the minimum size is asked again"
        );
    }
    let again = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Normal),
    );
    assert_eq!(again.units(), 0, "the same step: no task");
}

/// J24b: a change returns the minimum-size request **and** the focus on the group (two tasks); the
/// same step returns none of them.
#[test]
fn a_change_also_asks_for_the_focus_on_the_group() {
    let mut state = AppState::default();
    let change = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Large),
    );
    assert_eq!(
        change.units(),
        min_window_size_task().units() + crate::focus::keep_size_group_focus().units(),
        "the minimum size and the focus"
    );
}

#[test]
fn the_minimum_window_size_is_the_smallest_size_the_layout_is_tested_at() {
    assert_eq!(min_window_size(), iced::Size::new(360.0, 480.0));
    assert!(min_window_size_task().units() > 0);
}

/// `run` hands iced the factor and the minimum size at boot. A source test: the wiring is a builder
/// chain that nothing else exercises.
#[test]
fn the_application_hands_iced_the_factor_and_asks_for_the_minimum_at_boot() {
    let src = include_str!("../app.rs");
    let run =
        &src[src.find("pub fn run()").expect("run")..src.find("fn window_title").expect("end")];
    assert!(
        run.contains(".scale_factor(scale_for)"),
        "the factor reaches iced"
    );
    assert!(
        run.contains("min_window_size_task()"),
        "the minimum is asked at boot"
    );
}

/// The breakpoint reads `window_size`, in **scaled** logical pixels (RFC 034 D2): 767 is mobile and 768
/// is desktop whatever the step.
#[test]
fn the_mobile_breakpoint_is_at_768_scaled_pixels() {
    let mut state = AppState::default();
    let _ = update(
        &mut state,
        Message::WindowResized(iced::Size::new(767.0, 600.0)),
    );
    assert!(is_mobile(&state));
    let _ = update(
        &mut state,
        Message::WindowResized(iced::Size::new(768.0, 600.0)),
    );
    assert!(!is_mobile(&state));
}

/// iced sends no `Resized` event when only the scale changes (the real run of J24: the sidebar stayed
/// on screen in a 694 px viewport), so choosing a step rescales the window size the breakpoint reads.
/// The window does not change: its size in scaled pixels is its physical size over the factor.
#[test]
fn choosing_a_step_rescales_the_window_size_the_breakpoint_reads() {
    // The 1042 x 1144 window of the real run, reported at Normal.
    let mut state = AppState::default();
    let _ = update(
        &mut state,
        Message::WindowResized(iced::Size::new(1042.0, 1144.0)),
    );
    assert!(!is_mobile(&state));
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Large),
    );
    assert!(
        (state.window_size.width - 833.6).abs() < 0.1,
        "{:?}",
        state.window_size
    );
    assert!(!is_mobile(&state), "833.6 is above the breakpoint");
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::ExtraLarge),
    );
    assert!(
        (state.window_size.width - 694.67).abs() < 0.1,
        "{:?}",
        state.window_size
    );
    assert!(
        (state.window_size.height - 762.67).abs() < 0.1,
        "{:?}",
        state.window_size
    );
    assert!(
        is_mobile(&state),
        "694.7 is below the breakpoint: the phone layout, as after a restart"
    );
    // Back: the same window, the size it had.
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Normal),
    );
    assert!(
        (state.window_size.width - 1042.0).abs() < 0.01,
        "{:?}",
        state.window_size
    );
    assert!(!is_mobile(&state));
    // Choosing the step already in force changes nothing.
    let before = state.window_size;
    let _ = update(
        &mut state,
        Message::InterfaceSizeChosen(InterfaceSize::Normal),
    );
    assert_eq!(state.window_size, before);
}

/// What the app reports at start at a step is already scaled: a restart at Extra large (the real run:
/// opened at 694.4 x 762.8) is mobile, a 1280 px window at that step reports 853 and is not.
#[test]
fn a_window_reported_at_a_step_is_judged_in_scaled_pixels() {
    let mut state = AppState {
        interface_size: InterfaceSize::ExtraLarge,
        ..AppState::default()
    };
    let _ = update(
        &mut state,
        Message::WindowResized(iced::Size::new(694.4, 762.8)),
    );
    assert!(is_mobile(&state));
    let _ = update(
        &mut state,
        Message::WindowResized(iced::Size::new(853.0, 480.0)),
    );
    assert!(!is_mobile(&state));
}
