# UI and Accessibility

This document records the ABDD commitments (Accessible by Default and
by Design — spec §1, §12) that the UI implementation must uphold.
These are not nice-to-haves; they are acceptance criteria.

## Logical, not physical, layout

Direction is expressed via `snora::LayoutDirection::{Ltr, Rtl}` and
edges via `snora::Edge::{Start, End}`. This means a single
configuration value flips the whole UI for RTL languages — header
end-controls, sidebar side, toast anchor, custom rows authored with
`snora::direction::row_dir`. We do not author "left" / "right"
anywhere in the layout code.

## Keyboard support (current)

**Goal.** Full keyboard parity: every operation reachable by mouse is also
reachable by keyboard. This is **not met yet**. The main task (choose a file,
download all, go back, switch pages) has a keyboard path through a small set of
global shortcuts (RFC 019 phase 2). Moving between buttons and fields with Tab,
and operating the Customize page, are still to come
([RFC 019](../../rfcs/accepted/019-keyboard-accessibility.md),
[RFC 027](../../rfcs/accepted/027-keyboard-focus-model.md)).

Everything below was verified by running a debug build of the development tree
(after RFC 019 phase 2) on Linux under the niri compositor, with an English
interface: through the X11 backend (XWayland) and on native Wayland. Windows and
macOS have not been tested (on macOS the command key is Cmd instead of Ctrl; that
is the intended mapping and is not verified). Non-Latin keyboard layouts
(for example a Japanese one) are covered by unit tests of the key mapping only,
not by a run on such a layout. The statements about Tab, Enter, Space and the focus
indicator were verified in phase 1 on release 1.26.2 and have not changed.

**Supported today: global shortcuts**

The list is also shown in the application, on the Settings page (press F1).

| Keys | Action | Works when |
|---|---|---|
| Ctrl+O | Choose a file (opens the operating system's file chooser) | Home page, start or Result screen |
| Ctrl+S | Download all (ZIP) (opens the save dialog) | Home page, Result screen |
| Esc | Close the newest message; with no message, go back to the start screen | message: any page; back: Home page, Result screen |
| Ctrl+1 / 2 / 3 | Go to Home / Customize / Settings | always |
| F1 | Go to Settings (the shortcut list) | always |

A shortcut works exactly when the button it stands for is available to the
mouse. While a file is being converted, Ctrl+O and Ctrl+S do nothing. Keys
held down (key repeat) do nothing. Letters and digits are matched by their
position on the keyboard, so a non-Latin layout is meant to work too. Shift or
Alt on top of Ctrl make a different combination that is not bound.

- Text fields (custom PNG size, web manifest name, short name and colours)
  accept typed text once they have been focused with a mouse click. Enter in
  the custom-size field adds the size.
- The operating system's file chooser and save dialogs can be operated with the
  keyboard once they are open.

**Known behaviour with text fields and long conversions (observed)**

- While the custom-size field has focus, Ctrl+O does nothing (no file chooser
  opens), Ctrl+3 and F1 still change the page, and Ctrl+1 types a "1" into the
  field instead of going Home (the field takes the key first). Observed on
  X11 only.
- When a conversion takes a very long time (a 50-second test image), shortcuts
  pressed meanwhile were not acted on at once: Ctrl+O and Ctrl+S opened nothing,
  and Ctrl+2 took effect only after the conversion had finished. Typical logos
  convert in well under a second.

**Not supported yet (RFC 019 / RFC 027)**

- Tab and Shift+Tab move no focus: no button or control can be reached, including
  text fields, and Tab does not move between text fields.
- Enter and Space activate nothing: not the drop-zone card or "Choose file…",
  the navigation items, the download buttons, "Back", the toggles, the sliders,
  or "Advanced settings".
- The Customize page (size checkboxes, sliders, toggles, text fields) can only be
  operated with the mouse. Individual downloads ("Save" on a card) need the mouse.
- Arrow and paging keys do not scroll.
- There is no visible keyboard-focus indicator. The drop-zone card also shows no
  hover feedback; the navigation items and the download buttons do.
- Window-close shortcuts are not handled by the application; they depend on the
  window manager.

## Status, not just color

State is communicated by **text markers** in addition to color, so
users with low vision or color-blindness do not lose information:

- Busy: prefixed with `⏳` (`marker::BUSY`)
- Error: prefixed with `⚠` (`marker::ERROR`)
- Ready: prefixed with `✓` (`marker::READY`)

These live in `crate::ui::accessibility::marker` and are reused
across screens.

## Labels, not glyphs

Every interactive widget has a meaningful text label even when its
visual representation is short. The canonical labels are centralized
in `crate::ui::accessibility::label`:

- `APP_TITLE`, `DROP_ZONE`, `CHOOSE_FILE_BTN`, `TOGGLE_THEME_BTN`,
  `TOGGLE_ADVANCED_BTN`, `EXPORT_BTN`.

Centralization is deliberate: when a screen reader pronounces a
button, it should match what the user just heard the same button
pronounce on a different screen.

## Errors as toasts, not screens

Errors do not hijack the screen. They appear as **persistent**
`snora::Toast` notifications, so the user's loaded image, preview, and
progress remain visible. The user dismisses an error explicitly when
they have read it (`Message::DismissToast(id)`). Successes use a
transient lifetime — long enough to read, short enough not to stack.

## Detail kept hidden by default

Per §5.3, advanced settings are **not** shown on the Home screen.
They live on the full-page Customize view, with rare options grouped
under a collapsible section. This keeps the startup path simple while
leaving power-user controls discoverable.

## What we do not do

- We do **not** rely on hover-only interactions.
- We do **not** use color as the only indicator of state.
- We do **not** require drag-and-drop to choose a file: the file chooser is the
  alternative. It is opened with a mouse click or with Ctrl+O (see "Keyboard
  support (current)").
- We do **not** auto-dismiss error notifications.
