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
reachable by keyboard. This is **not met yet**; the work is tracked in
[RFC 019](../../rfcs/accepted/019-keyboard-accessibility.md).

The lists below were verified by running Logolig 1.26.2 (release build) on
Linux under the niri compositor: through the X11 backend (XWayland) on every
screen, and on native Wayland on the Home screen only (the other screens need a
mouse to reach). Windows and macOS have not been tested.

**Supported today**

- Text fields (custom PNG size, web manifest name, short name and colours)
  accept typed text once they have been focused with a mouse click. Enter in
  the custom-size field adds the size.
- The operating system's file chooser and save dialogs can be operated with the
  keyboard once they are open. They are opened with a mouse click.

**Not supported yet (tracked in RFC 019)**

- Tab and Shift+Tab move no focus: no control can be reached, including text
  fields, and Tab does not move between text fields.
- Enter and Space activate nothing: not the drop-zone card or "Choose file…",
  the navigation items, the download buttons, "Back", the toggles, the sliders,
  or "Advanced settings".
- As a result, the main path (choose a file, download, go back, switch between
  Home, Customize and Settings) needs a mouse.
- There are no keyboard shortcuts (for example Ctrl+O, Ctrl+S), Esc does
  nothing, and arrow and paging keys do not scroll.
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
  alternative. It is opened with a mouse click today (see "Keyboard support
  (current)").
- We do **not** auto-dismiss error notifications.
