use iced::keyboard::key::{Code, Named, Physical};
use iced::keyboard::{Key, Modifiers};

use super::{Shortcut, command_key_label, shortcut_for};
use crate::app::NavPage;

const CMD: Modifiers = Modifiers::COMMAND;

fn ch(s: &str) -> Key {
    Key::Character(s.into())
}

fn press(key: Key, code: Code, modifiers: Modifiers) -> Option<Shortcut> {
    shortcut_for(&key, Physical::Code(code), modifiers, false)
}

#[test]
fn command_combinations_map_to_their_shortcut() {
    let table = [
        (ch("o"), Code::KeyO, Shortcut::ChooseFile),
        (ch("s"), Code::KeyS, Shortcut::DownloadAll),
        (ch("1"), Code::Digit1, Shortcut::Page(NavPage::Home)),
        (ch("2"), Code::Digit2, Shortcut::Page(NavPage::Customize)),
        (ch("3"), Code::Digit3, Shortcut::Page(NavPage::Settings)),
    ];
    for (key, code, want) in table {
        assert_eq!(press(key.clone(), code, CMD), Some(want), "{key:?}");
    }
}

#[test]
fn caps_lock_does_not_change_a_letter_shortcut() {
    // With Caps Lock the logical key is upper case but Shift is not held.
    assert_eq!(press(ch("O"), Code::KeyO, CMD), Some(Shortcut::ChooseFile));
}

#[test]
fn escape_and_f1_need_no_modifier() {
    assert_eq!(
        press(Key::Named(Named::Escape), Code::Escape, Modifiers::NONE),
        Some(Shortcut::EscapeOrBack)
    );
    assert_eq!(
        press(Key::Named(Named::F1), Code::F1, Modifiers::NONE),
        Some(Shortcut::Help)
    );
}

#[test]
fn escape_and_f1_with_any_modifier_do_nothing() {
    for m in [
        Modifiers::SHIFT,
        Modifiers::ALT,
        Modifiers::CTRL,
        Modifiers::LOGO,
    ] {
        assert_eq!(
            press(Key::Named(Named::Escape), Code::Escape, m),
            None,
            "{m:?}"
        );
        assert_eq!(press(Key::Named(Named::F1), Code::F1, m), None, "{m:?}");
    }
}

#[test]
fn plain_letters_and_digits_never_trigger() {
    assert_eq!(press(ch("o"), Code::KeyO, Modifiers::NONE), None);
    assert_eq!(press(ch("s"), Code::KeyS, Modifiers::NONE), None);
    assert_eq!(press(ch("1"), Code::Digit1, Modifiers::NONE), None);
}

#[test]
fn extra_modifiers_turn_a_command_combination_into_something_else() {
    for extra in [Modifiers::SHIFT, Modifiers::ALT] {
        let m = CMD | extra;
        assert_eq!(press(ch("o"), Code::KeyO, m), None, "{m:?}");
        assert_eq!(press(ch("s"), Code::KeyS, m), None, "{m:?}");
        assert_eq!(press(ch("1"), Code::Digit1, m), None, "{m:?}");
    }
    // The non-command modifier alone is not Cmd either.
    assert_eq!(press(ch("o"), Code::KeyO, Modifiers::ALT), None);
    assert_eq!(press(ch("o"), Code::KeyO, Modifiers::SHIFT), None);
}

#[test]
fn a_key_repeat_never_triggers() {
    let cases = [
        (ch("o"), Code::KeyO, CMD),
        (ch("s"), Code::KeyS, CMD),
        (ch("1"), Code::Digit1, CMD),
        (Key::Named(Named::Escape), Code::Escape, Modifiers::NONE),
        (Key::Named(Named::F1), Code::F1, Modifiers::NONE),
    ];
    for (key, code, m) in cases {
        assert_eq!(
            shortcut_for(&key, Physical::Code(code), m, true),
            None,
            "{key:?}"
        );
        assert!(
            shortcut_for(&key, Physical::Code(code), m, false).is_some(),
            "{key:?}"
        );
    }
}

#[test]
fn unbound_keys_do_nothing() {
    let cases = [
        (Key::Named(Named::Tab), Code::Tab, Modifiers::NONE),
        (Key::Named(Named::Tab), Code::Tab, CMD),
        (
            Key::Named(Named::ArrowLeft),
            Code::ArrowLeft,
            Modifiers::NONE,
        ),
        (Key::Named(Named::ArrowDown), Code::ArrowDown, CMD),
        (Key::Named(Named::F6), Code::F6, Modifiers::NONE),
        (ch("w"), Code::KeyW, CMD),
        (ch("4"), Code::Digit4, CMD),
        (ch("0"), Code::Digit0, CMD),
        (Key::Named(Named::Enter), Code::Enter, Modifiers::NONE),
        (Key::Named(Named::Space), Code::Space, Modifiers::NONE),
    ];
    for (key, code, m) in cases {
        assert_eq!(press(key.clone(), code, m), None, "{key:?} {m:?}");
    }
}

#[test]
fn a_non_latin_layout_is_matched_by_the_physical_key() {
    // Japanese layout: the logical key is a kana, the physical key is that of `O`.
    assert_eq!(press(ch("ち"), Code::KeyO, CMD), Some(Shortcut::ChooseFile));
    assert_eq!(press(ch("ち"), Code::KeyO, CMD | Modifiers::SHIFT), None);
    // A Cyrillic logical key on the physical `S` key.
    assert_eq!(press(ch("ы"), Code::KeyS, CMD), Some(Shortcut::DownloadAll));
    // A non-Latin key on an unrelated physical key stays unbound.
    assert_eq!(press(ch("ち"), Code::KeyA, CMD), None);
}

#[test]
fn an_unidentified_physical_key_with_a_non_latin_logical_key_does_nothing() {
    use iced::keyboard::key::NativeCode;
    let got = shortcut_for(
        &ch("ち"),
        Physical::Unidentified(NativeCode::Unidentified),
        CMD,
        false,
    );
    assert_eq!(got, None);
}

#[test]
fn the_command_key_label_is_ctrl_on_this_platform() {
    if cfg!(target_os = "macos") {
        assert_eq!(command_key_label(), "Cmd");
    } else {
        assert_eq!(command_key_label(), "Ctrl");
    }
}
