//! Key events into a focused text field (RFC 027, key ownership; job J13b).
//!
//! A command chord is a shortcut, never text; typing, Backspace, the arrows, Enter and Esc stay the
//! field's. Events are delivered the way the window does, to the whole interface; "Ignored" is what
//! the application's own subscription receives (it passes only events nothing captured).
//!
//! The `text` of each event is the one **measured on the real stacks** (J13 spike, X11; J13b step
//! 0, native Wayland, `evidence/wayland-text-run.txt`): Ctrl+1 carries `"1"`, Ctrl+2 `"\0"`,
//! Ctrl+3 ESC, Ctrl+O SI, Ctrl+S DC3 (control characters, which the field refuses), F1 none. Where a
//! stack might send the plain letter instead, a second case carries it, so the result does not
//! depend on that accident.

use iced::Size;
use iced::keyboard::{Key, Modifiers, key::Named};
use iced::mouse::Cursor;
use iced_test::core::event::Status;
use iced_test::core::widget::operation::focusable;
use iced_test::core::{Event, keyboard};

use super::support::{Case, Combo, Kind, Probe, probe_of};
use crate::app::{AppState, Message, NavPage};
use crate::shortcuts::{Shortcut, shortcut_for};

fn event(key: Key, mods: Modifiers, text: Option<&str>) -> Event {
    Event::Keyboard(keyboard::Event::KeyPressed {
        key: key.clone(),
        modified_key: key,
        // The Wayland virtual keyboard of `wtype` reports one physical key for every key; the
        // logical key is what the shortcuts use, so the same is done here.
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Escape),
        location: keyboard::Location::Standard,
        modifiers: mods,
        repeat: false,
        text: text.map(Into::into),
    })
}

/// The events the window delivers for one key: the modifier change first (a text field tracks the
/// held modifiers from it, not from the key event), then the key.
fn with_mods(ev: Event) -> Vec<Event> {
    let mods = match &ev {
        Event::Keyboard(keyboard::Event::KeyPressed { modifiers, .. }) => *modifiers,
        _ => Modifiers::empty(),
    };
    vec![Event::Keyboard(keyboard::Event::ModifiersChanged(mods)), ev]
}

fn ch(c: &str, mods: Modifiers, text: Option<&str>) -> Event {
    event(Key::Character(c.into()), mods, text)
}

fn named(n: Named, mods: Modifiers, text: Option<&str>) -> Event {
    event(Key::Named(n), mods, text)
}

/// The Customize page with the custom-size field focused and "12" typed into it.
fn with_field() -> (AppState, Size) {
    let size = Size::new(1280.0, 720.0);
    let mut state = Combo {
        case: Case::CustomizeAdvanced,
        size,
        locale: logolig_i18n::Locale::En,
        dark: false,
    }
    .state();
    state.export_plan = logolig::ExportPlan::default();
    state.advanced_extras_open = false;
    (state, size)
}

fn focused_field<'a>(state: &'a AppState, size: Size) -> Probe<'a, Message> {
    let mut p = probe_of(state, size);
    // The only focus target of this page with the default plan is the field.
    p.run_chained(focusable::focus_next::<()>());
    for c in ['1', '2'] {
        let s = c.to_string();
        let _ = p.send(
            &with_mods(ch(&s, Modifiers::empty(), Some(&s))),
            Cursor::Unavailable,
        );
    }
    p
}

fn field_text(p: &mut Probe<'_, Message>) -> String {
    p.walk()
        .into_iter()
        .find(|s| s.kind == Kind::TextInput)
        .map(|s| s.text)
        .unwrap_or_default()
}

/// What the application would do with the event if nothing captured it.
fn shortcut_of(ev: &Event) -> Option<Shortcut> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        repeat,
        ..
    }) = ev
    else {
        return None;
    };
    shortcut_for(key, *physical_key, *modifiers, *repeat)
}

/// Every command shortcut, with the text it carries on the real stacks, while the field is
/// focused: the field takes nothing, the event stays unused (so the application gets it), no text
/// message is published, and the application would act on it.
#[test]
fn a_command_shortcut_is_not_typed_into_the_focused_field() {
    let ctl = Modifiers::CTRL;
    let cases: Vec<(&str, Event, Shortcut)> = vec![
        (
            "Ctrl+1, text \"1\" (X11 and Wayland)",
            ch("1", ctl, Some("1")),
            Shortcut::Page(NavPage::Home),
        ),
        (
            "Ctrl+2, text NUL (Wayland)",
            ch("2", ctl, Some("\0")),
            Shortcut::Page(NavPage::Customize),
        ),
        (
            "Ctrl+3, text ESC (Wayland)",
            ch("3", ctl, Some("\u{1b}")),
            Shortcut::Page(NavPage::Settings),
        ),
        (
            "Ctrl+O, text SI (Wayland)",
            ch("o", ctl, Some("\u{f}")),
            Shortcut::ChooseFile,
        ),
        (
            "Ctrl+S, text DC3 (Wayland)",
            ch("s", ctl, Some("\u{13}")),
            Shortcut::DownloadAll,
        ),
        // A stack that sends the letter instead of a control character.
        (
            "Ctrl+2, text \"2\"",
            ch("2", ctl, Some("2")),
            Shortcut::Page(NavPage::Customize),
        ),
        (
            "Ctrl+3, text \"3\"",
            ch("3", ctl, Some("3")),
            Shortcut::Page(NavPage::Settings),
        ),
        (
            "Ctrl+O, text \"o\"",
            ch("o", ctl, Some("o")),
            Shortcut::ChooseFile,
        ),
        (
            "Ctrl+S, text \"s\"",
            ch("s", ctl, Some("s")),
            Shortcut::DownloadAll,
        ),
        (
            "F1 (no text)",
            named(Named::F1, Modifiers::empty(), None),
            Shortcut::Help,
        ),
    ];
    let (state, size) = with_field();
    let mut problems: Vec<String> = Vec::new();
    for (name, ev, expected) in cases {
        assert_eq!(
            shortcut_of(&ev),
            Some(expected),
            "{name}: the shortcut itself"
        );
        let mut p = focused_field(&state, size);
        assert_eq!(field_text(&mut p), "12", "{name}: precondition");
        let (statuses, messages) = p.send(&with_mods(ev), Cursor::Unavailable);
        if statuses.last() != Some(&Status::Ignored) {
            problems.push(format!(
                "{name}: the field captured the key ({:?})",
                statuses.last()
            ));
        }
        if !messages.is_empty() {
            problems.push(format!(
                "{name}: the field published {} message(s)",
                messages.len()
            ));
        }
        let text = field_text(&mut p);
        if text != "12" {
            problems.push(format!("{name}: the field text became {text:?}"));
        }
    }
    assert!(
        problems.is_empty(),
        "command keys reached the field:\n  {}",
        problems.join("\n  ")
    );
}

/// What the field keeps: typing, Backspace, the arrows, Enter, and the text of AltGr (Ctrl+Alt on
/// Windows), plus its own Ctrl+A and Ctrl+C.
#[test]
fn the_field_keeps_typing_editing_and_its_own_commands() {
    let none = Modifiers::empty();
    let (state, size) = with_field();

    let mut p = focused_field(&state, size);
    let (st, msgs) = p.send(&with_mods(ch("3", none, Some("3"))), Cursor::Unavailable);
    assert_eq!(st.last(), Some(&Status::Captured));
    assert_eq!(field_text(&mut p), "123", "plain typing");
    assert_eq!(msgs.len(), 1);

    let mut p = focused_field(&state, size);
    let (st, _) = p.send(
        &with_mods(named(Named::Backspace, none, None)),
        Cursor::Unavailable,
    );
    assert_eq!(st.last(), Some(&Status::Captured));
    assert_eq!(field_text(&mut p), "1", "Backspace");

    for (n, key) in [("left", Named::ArrowLeft), ("right", Named::ArrowRight)] {
        let mut p = focused_field(&state, size);
        let (st, _) = p.send(&with_mods(named(key, none, None)), Cursor::Unavailable);
        assert_eq!(st.last(), Some(&Status::Captured), "arrow {n}");
    }

    let mut p = focused_field(&state, size);
    let (st, msgs) = p.send(
        &with_mods(named(Named::Enter, none, None)),
        Cursor::Unavailable,
    );
    assert_eq!(st.last(), Some(&Status::Captured), "Enter");
    assert!(
        msgs.iter()
            .any(|m| matches!(m, Message::PngSizeAddRequested)),
        "Enter submits: {msgs:?}"
    );

    // Ctrl+A (select all) and Ctrl+C (copy) are the field's: it captures them, and they stay out of the shortcuts.
    for (name, ev) in [
        ("Ctrl+A", ch("a", Modifiers::CTRL, Some("\u{1}"))),
        ("Ctrl+C", ch("c", Modifiers::CTRL, Some("\u{3}"))),
        ("Ctrl+A, text \"a\"", ch("a", Modifiers::CTRL, Some("a"))),
    ] {
        let mut p = focused_field(&state, size);
        let (st, _) = p.send(&with_mods(ev), Cursor::Unavailable);
        assert_eq!(
            st.last(),
            Some(&Status::Captured),
            "{name}: the field handles it"
        );
        assert_eq!(field_text(&mut p), "12", "{name}: and types nothing");
    }

    // AltGr arrives as Ctrl+Alt on Windows and types a character: it must still type.
    let mut p = focused_field(&state, size);
    let (st, _) = p.send(
        &[ch("q", Modifiers::CTRL | Modifiers::ALT, Some("@"))],
        Cursor::Unavailable,
    );
    assert_eq!(st.last(), Some(&Status::Captured), "AltGr");
    assert_eq!(field_text(&mut p), "12@", "AltGr text is kept");
}

/// Completeness (RFC 027, J13b): every text input of every page state, focused in turn, ignores a
/// command chord that carries text. The wrapper sits at the root of the view, so a field added
/// later is covered; this test fails if a future change takes it out of that path.
#[test]
fn every_text_input_of_every_page_state_ignores_a_command_chord() {
    let size = Size::new(1280.0, 720.0);
    let mut inputs_seen = 0;
    let mut problems: Vec<String> = Vec::new();
    for case in super::support::CASES {
        let state = Combo {
            case,
            size,
            locale: logolig_i18n::Locale::En,
            dark: false,
        }
        .state();
        let count = probe_of(&state, size)
            .walk()
            .iter()
            .filter(|s| s.kind == Kind::TextInput)
            .count();
        inputs_seen += count;
        for k in 1..=count {
            let mut p = probe_of(&state, size);
            for _ in 0..k {
                p.run_chained(focusable::focus_next::<()>());
            }
            for (name, ev) in [
                ("Ctrl+1 \"1\"", ch("1", Modifiers::CTRL, Some("1"))),
                ("Ctrl+2 \"2\"", ch("2", Modifiers::CTRL, Some("2"))),
                ("Ctrl+O \"o\"", ch("o", Modifiers::CTRL, Some("o"))),
            ] {
                let (statuses, messages) = p.send(&with_mods(ev), Cursor::Unavailable);
                if statuses.last() != Some(&Status::Ignored) || !messages.is_empty() {
                    problems.push(format!(
                        "{case:?}, input {k} of {count}: {name} was taken ({:?}, {} message(s))",
                        statuses.last(),
                        messages.len()
                    ));
                }
            }
        }
    }
    assert!(
        inputs_seen >= 5,
        "the scan found only {inputs_seen} text inputs"
    );
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The strip is for command chords only: plain typing, Shift+1 ("!") and Ctrl+Alt (AltGr on
/// Windows) keep their text.
#[test]
fn only_a_command_chord_loses_its_text() {
    let (state, size) = with_field();
    for (name, ev, typed) in [
        ("plain 3", ch("3", Modifiers::empty(), Some("3")), "123"),
        ("Shift+1", ch("1", Modifiers::SHIFT, Some("!")), "12!"),
        (
            "Ctrl+Alt+q",
            ch("q", Modifiers::CTRL | Modifiers::ALT, Some("@")),
            "12@",
        ),
    ] {
        let mut p = focused_field(&state, size);
        let (statuses, _) = p.send(&with_mods(ev), Cursor::Unavailable);
        assert_eq!(statuses.last(), Some(&Status::Captured), "{name}");
        assert_eq!(field_text(&mut p), typed, "{name}");
    }
}

/// The path the application uses: the subscription's filter turns a command chord that nothing
/// captured into its shortcut, and turns a captured one into nothing (a key a widget used is not
/// a shortcut). Together with the tests above (the field leaves a chord `Ignored`), Ctrl+1 in a
/// focused field navigates once and types nothing.
#[test]
fn the_subscription_filter_answers_an_ignored_chord_once_and_a_captured_key_not_at_all() {
    use crate::app::runtime_event;
    let chord = ch("1", Modifiers::CTRL, Some("1"));
    assert!(matches!(
        runtime_event(chord.clone(), Status::Ignored),
        Some(Message::ShortcutPressed(Shortcut::Page(NavPage::Home)))
    ));
    assert!(runtime_event(chord, Status::Captured).is_none());
    // A plain digit is not a shortcut at all.
    assert!(runtime_event(ch("1", Modifiers::empty(), Some("1")), Status::Ignored).is_none());
}
