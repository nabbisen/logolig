//! Toast expiry through the stored `Holds` (RFC 032).
//!
//! The handlers read the clock themselves (`Instant::now()`), so these tests control time by
//! setting a toast's `created_at` in the past: a toast whose age is above its lifetime is
//! expired at the next `ToastTick`.

use super::*;

use std::time::Duration;

use snora::{Toast, ToastAttention, ToastIntent, ToastLifetime};

const LIFETIME_SECS: u64 = 4;

/// A transient toast of `LIFETIME_SECS` seconds that was created `age_secs` seconds ago.
fn transient_toast(id: u64, age_secs: u64) -> Toast<Message> {
    let mut toast = Toast::new(
        id,
        ToastIntent::Success,
        "title",
        "body",
        Message::DismissToast(id),
    )
    .with_lifetime(ToastLifetime::seconds(LIFETIME_SECS));
    toast.created_at = Instant::now()
        .checked_sub(Duration::from_secs(age_secs))
        .expect("the monotonic clock is older than a minute");
    toast
}

fn state_with(toasts: Vec<Toast<Message>>) -> AppState {
    AppState {
        toasts,
        ..AppState::default()
    }
}

fn ids(state: &AppState) -> Vec<u64> {
    state.toasts.iter().map(|t| t.id).collect()
}

fn attend(state: &mut AppState, id: u64, attention: ToastAttention) {
    let _ = update(state, Message::ToastAttention(id, attention));
}

fn tick(state: &mut AppState) {
    let _ = update(state, Message::ToastTick);
}

/// Make the toast look `age_secs` old again (time passing without waiting).
fn age(state: &mut AppState, id: u64, age_secs: u64) {
    let toast = state
        .toasts
        .iter_mut()
        .find(|t| t.id == id)
        .expect("the toast is still there");
    toast.created_at = Instant::now()
        .checked_sub(Duration::from_secs(age_secs))
        .expect("the monotonic clock is older than a minute");
}

#[test]
fn a_tick_removes_an_expired_toast_and_keeps_a_fresh_one() {
    let mut state = state_with(vec![transient_toast(1, 60), transient_toast(2, 0)]);
    tick(&mut state);
    assert_eq!(ids(&state), vec![2]);
}

#[test]
fn a_toast_the_pointer_is_on_survives_its_deadline() {
    let mut state = state_with(vec![transient_toast(1, 60)]);
    attend(&mut state, 1, ToastAttention::PointerEntered);
    tick(&mut state);
    assert_eq!(ids(&state), vec![1], "held toast must not expire");
    // Still held on a later tick.
    tick(&mut state);
    assert_eq!(ids(&state), vec![1]);
}

#[test]
fn leaving_restarts_the_full_countdown() {
    let mut state = state_with(vec![transient_toast(1, 60)]);
    attend(&mut state, 1, ToastAttention::PointerEntered);
    attend(&mut state, 1, ToastAttention::PointerLeft);
    tick(&mut state);
    assert_eq!(
        ids(&state),
        vec![1],
        "the countdown restarted in full when the pointer left"
    );
    // After the full lifetime has passed again, it goes.
    age(&mut state, 1, 60);
    tick(&mut state);
    assert!(ids(&state).is_empty());
}

#[test]
fn only_the_attended_toast_is_held() {
    let mut state = state_with(vec![transient_toast(1, 60), transient_toast(2, 60)]);
    attend(&mut state, 2, ToastAttention::PointerEntered);
    tick(&mut state);
    assert_eq!(ids(&state), vec![2]);
}

#[test]
fn keyboard_attention_holds_in_the_same_way_and_is_passed_through() {
    let mut state = state_with(vec![transient_toast(1, 60)]);
    attend(&mut state, 1, ToastAttention::FocusGained);
    tick(&mut state);
    assert_eq!(ids(&state), vec![1]);
    // The pointer comes and goes while the keyboard is still on the toast: still held, and the
    // old deadline is not revived.
    attend(&mut state, 1, ToastAttention::PointerEntered);
    attend(&mut state, 1, ToastAttention::PointerLeft);
    tick(&mut state);
    assert_eq!(ids(&state), vec![1]);
    // Only when the keyboard leaves too does the countdown restart (in full).
    attend(&mut state, 1, ToastAttention::FocusLost);
    tick(&mut state);
    assert_eq!(ids(&state), vec![1]);
    age(&mut state, 1, 60);
    tick(&mut state);
    assert!(ids(&state).is_empty());
}

#[test]
fn a_persistent_error_toast_is_unchanged_by_attention_and_ticks() {
    let mut state = AppState::default();
    push_error_toast(&mut state, AppError::unsupported_file("x.png"));
    let id = state.toasts[0].id;
    attend(&mut state, id, ToastAttention::PointerEntered);
    attend(&mut state, id, ToastAttention::PointerLeft);
    tick(&mut state);
    assert_eq!(state.toasts[0].lifetime, ToastLifetime::Persistent);
    assert_eq!(ids(&state), vec![id]);
    // The close button still dismisses it.
    let _ = update(&mut state, Message::DismissToast(id));
    assert!(state.toasts.is_empty());
}

#[test]
fn dismissing_a_held_toast_does_not_break_the_next_sweep() {
    let mut state = state_with(vec![transient_toast(1, 60), transient_toast(2, 0)]);
    attend(&mut state, 1, ToastAttention::PointerEntered);
    let _ = update(&mut state, Message::DismissToast(1));
    assert_eq!(ids(&state), vec![2]);
    tick(&mut state); // the sweep forgets the stale hold of toast 1

    // A new toast that happens to carry the same id (ids are never reused by the application;
    // this only proves that no hold of the dismissed toast is left over) expires normally.
    state.toasts.push(transient_toast(1, 60));
    tick(&mut state);
    assert_eq!(ids(&state), vec![2], "a stale hold would have kept toast 1");
}

#[test]
fn attention_for_an_unknown_toast_is_ignored() {
    let mut state = state_with(vec![transient_toast(1, 0)]);
    attend(&mut state, 999, ToastAttention::PointerEntered);
    attend(&mut state, 999, ToastAttention::PointerLeft);
    tick(&mut state);
    assert_eq!(ids(&state), vec![1]);
}
