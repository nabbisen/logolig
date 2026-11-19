//! Where the focus is after something changes the page (RFC 027, job J14b §3).
//!
//! A keyboard user must never be trapped: after a control that held the focus goes away (Back, a
//! page change), after a conversion ends, after a toast closes, after Esc in a text field, Tab must
//! still go somewhere sensible on a page that has stops. iced keeps widget state (the focus)
//! across a rebuilt view, so the checks run as a **session**: the real view is rebuilt after every
//! message, with the widget state of the previous build.

use iced::Size;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::mouse::Cursor;
use iced_test::runtime::user_interface::Cache;
use logolig_i18n::Locale;

use super::support::{Case, Combo, Probe, key_press_with};
use crate::app::{AppState, Message, runtime_event, update_for_test};

pub(super) struct Session {
    pub(super) state: AppState,
    size: Size,
    cache: Option<Cache>,
}

impl Session {
    pub(super) fn new(case: Case, size: Size) -> Self {
        Self {
            state: Combo {
                case,
                size,
                locale: Locale::En,
                dark: false,
            }
            .state(),
            size,
            cache: None,
        }
    }

    /// Runs `f` on the real view of the current state, with the widget state kept from before.
    pub(super) fn with<R>(&mut self, f: impl FnOnce(&mut Probe<'_, Message>) -> R) -> R {
        let cache = self.cache.take().unwrap_or_default();
        let mut probe = Probe::with_cache(crate::shell::view(&self.state), self.size, cache);
        let result = f(&mut probe);
        self.cache = Some(probe.into_cache());
        result
    }

    /// Applies a message as the runtime does: `update`, then the widget operations of its task.
    pub(super) fn apply(&mut self, message: Message) {
        let task = update_for_test(&mut self.state, message);
        self.with(|p| p.run_task(task));
    }

    /// A key press as the window delivers it: to the tree, then, if nothing used it, to the
    /// subscription's filter; every message that comes out is applied.
    pub(super) fn press(&mut self, key: Named, modifiers: Modifiers) {
        let event = key_press_with(Key::Named(key), modifiers);
        let (statuses, messages) =
            self.with(|p| p.send(std::slice::from_ref(&event), Cursor::Unavailable));
        let mut all = messages;
        all.extend(runtime_event(event, *statuses.last().expect("a status")));
        for message in all {
            self.apply(message);
        }
    }

    pub(super) fn tab(&mut self) {
        self.press(Named::Tab, Modifiers::empty());
    }

    /// The Tab-order index of the focused stop, and how many stops the page has.
    pub(super) fn focus(&mut self) -> (Option<usize>, usize) {
        self.with(|p| {
            let stops = p.focus_targets();
            (stops.iter().position(|t| t.focused), stops.len())
        })
    }

    /// What the focused stop is, by the text it holds (empty when nothing is focused).
    pub(super) fn focused_text(&mut self) -> String {
        self.with(|p| {
            let Some(stop) = p.focus_targets().into_iter().find(|t| t.focused) else {
                return String::new();
            };
            p.walk()
                .iter()
                .find(|s| {
                    s.kind == super::support::Kind::Text && stop.bounds.contains(s.bounds.center())
                })
                .map(|s| s.text.clone())
                .unwrap_or_else(|| "(no text)".into())
        })
    }
}

const SIZE: Size = Size::new(1280.0, 4000.0);

/// The navigation's three items come first in the Tab order on a desktop window.
const FIRST: usize = 0;

/// Ctrl+digit as the window delivers it (the physical digit row, a command chord).
fn ctrl_digit(s: &mut Session, digit: u8) {
    let key = Key::Character(digit.to_string().into());
    let code = match digit {
        1 => iced_test::core::keyboard::key::Code::Digit1,
        2 => iced_test::core::keyboard::key::Code::Digit2,
        _ => iced_test::core::keyboard::key::Code::Digit3,
    };
    let event = iced_test::core::Event::Keyboard(iced_test::core::keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        physical_key: iced_test::core::keyboard::key::Physical::Code(code),
        location: iced_test::core::keyboard::Location::Standard,
        modifiers: Modifiers::CTRL,
        repeat: false,
        text: None,
    });
    let (statuses, messages) =
        s.with(|p| p.send(std::slice::from_ref(&event), Cursor::Unavailable));
    let mut all = messages;
    all.extend(runtime_event(event, *statuses.last().expect("a status")));
    for message in all {
        s.apply(message);
    }
}

/// Tabs `n` times.
fn tabs(s: &mut Session, n: usize) {
    for _ in 0..n {
        s.tab();
    }
}

/// Back removes the focused control and changes the page: nothing is focused, and the next Tab
/// starts again at the first stop of the Home page (snora's rule: after a press that changes the
/// number of controls "focus is wherever the application put it, or nowhere").
#[test]
fn back_leaves_nothing_focused_and_tab_starts_again() {
    let mut s = Session::new(Case::ResultDefault, SIZE);
    tabs(&mut s, 4); // three navigation items, then Back
    assert_eq!(s.focused_text(), "← Back");
    s.press(Named::Enter, Modifiers::empty());
    assert_eq!(s.state.screen, crate::app::Screen::Empty, "Back went home");
    let (focus, stops) = s.focus();
    assert_eq!(focus, None, "the control that held the focus is gone");
    assert!(stops >= 4, "the Home page has stops: {stops}");
    s.tab();
    assert_eq!(
        s.focus().0,
        Some(FIRST),
        "Tab starts again at the first stop"
    );
}

/// A shortcut that changes the page while a control of the old page holds the focus.
#[test]
fn a_page_change_by_shortcut_leaves_no_trap() {
    let mut s = Session::new(Case::Home, SIZE);
    tabs(&mut s, 4); // the drop-zone card
    assert!(s.focused_text().starts_with("Drop"), "{}", s.focused_text());
    ctrl_digit(&mut s, 2);
    assert_eq!(s.state.nav_page, crate::app::NavPage::Customize);
    assert_eq!(s.focus().0, None, "the card is gone with its page");
    s.tab();
    assert_eq!(s.focus().0, Some(FIRST));
}

/// A conversion ends while the focus is on a navigation item: the item is still there, the focus
/// stays on it, and Tab goes on to the next stop (then the new page's Back).
#[test]
fn a_conversion_ending_keeps_the_focus_on_the_navigation() {
    let mut s = Session::new(Case::Converting, SIZE);
    s.tab();
    assert_eq!(s.focus().0, Some(0));
    s.state.screen = crate::app::Screen::Result;
    s.state.result_assets = Some(crate::app::test_support::result_assets_of_cards(false));
    let (focus, stops) = s.focus();
    assert_eq!(focus, Some(0), "still on the first navigation item");
    assert!(
        stops > 10,
        "the Result page's controls are stops now: {stops}"
    );
    s.tab();
    assert_eq!(s.focus().0, Some(1));
}

/// Enter on a navigation item changes the page. snora drops the focus after a press that changes
/// the number of stops (documented), so nothing is focused and Tab starts again: no trap.
#[test]
fn enter_on_a_navigation_item_changes_the_page_and_tab_starts_again() {
    let mut s = Session::new(Case::Home, SIZE);
    tabs(&mut s, 2); // the Customize item
    s.press(Named::Enter, Modifiers::empty());
    assert_eq!(s.state.nav_page, crate::app::NavPage::Customize);
    assert_eq!(s.focus().0, None);
    s.tab();
    assert_eq!(s.focus().0, Some(FIRST));
    // The item of the page already shown: nothing changes, and the focus stays.
    let mut s = Session::new(Case::Home, SIZE);
    s.tab();
    s.press(Named::Enter, Modifiers::empty());
    assert_eq!(
        s.focus().0,
        Some(0),
        "pressing the current page's item changes nothing"
    );
}

/// A toast adds its close control as the last stop; the focus is not taken from where it was;
/// closing it with the keyboard leaves nothing focused and Tab starts again.
#[test]
fn a_toast_adds_a_stop_and_closing_it_by_key_leaves_no_trap() {
    let mut s = Session::new(Case::Home, SIZE);
    let before = s.focus().1;
    s.tab();
    s.apply(Message::DialogUnavailable(crate::app::DialogKind::Chooser));
    let (focus, stops) = s.focus();
    assert_eq!(stops, before + 1, "the toast's close is one more stop");
    assert_eq!(focus, Some(0), "the focus stays where it was");
    tabs(&mut s, stops - 1); // on to the last stop: the close control
    assert_eq!(s.focus().0, Some(stops - 1));
    s.press(Named::Enter, Modifiers::empty());
    assert!(s.state.toasts.is_empty(), "the toast is closed");
    assert_eq!(s.focus().0, None, "the close control is gone");
    s.tab();
    assert_eq!(s.focus().0, Some(FIRST));
}

/// Esc in a text field leaves it (the field takes the key first); Tab then starts again.
#[test]
fn esc_in_a_field_leaves_it_and_tab_starts_again() {
    let mut s = Session::new(Case::CustomizeAdvanced, SIZE);
    s.with(|p| p.focus_text_input(0));
    assert!(s.focus().0.is_some(), "the field has the focus");
    s.press(Named::Escape, Modifiers::empty());
    assert_eq!(s.focus().0, None, "Esc left the field");
    assert_eq!(
        s.state.nav_page,
        crate::app::NavPage::Customize,
        "and went nowhere else"
    );
    s.tab();
    assert_eq!(s.focus().0, Some(FIRST));
}

/// The guarantee behind all of these: on **every** page state and size of the matrix that has
/// stops, from nothing focused, Tab focuses the first stop; after the last stop one more Tab
/// leaves the group and the next starts again. Tab never does nothing, and never traps.
#[test]
fn tab_never_does_nothing_on_a_page_that_has_stops() {
    use super::focus_keys::{focused, tab};
    use super::support::{layout_matrix, probe_of};
    let mut pages = 0;
    for combo in layout_matrix() {
        let state = combo.state();
        let mut p = probe_of(&state, combo.size);
        let stops = p.focus_targets().len();
        if stops == 0 {
            continue;
        }
        pages += 1;
        assert_eq!(focused(&mut p), None);
        tab(&mut p);
        assert_eq!(
            focused(&mut p),
            Some(0),
            "[{}] the first Tab",
            combo.describe()
        );
        for _ in 1..stops {
            tab(&mut p);
        }
        assert_eq!(
            focused(&mut p),
            Some(stops - 1),
            "[{}] the last stop",
            combo.describe()
        );
        tab(&mut p);
        assert_eq!(
            focused(&mut p),
            None,
            "[{}] Tab leaves the group",
            combo.describe()
        );
        tab(&mut p);
        assert_eq!(
            focused(&mut p),
            Some(0),
            "[{}] and starts again",
            combo.describe()
        );
    }
    assert!(pages > 100, "{pages} page states checked");
}
