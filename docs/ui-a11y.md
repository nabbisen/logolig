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

## Keyboard parity

Every operation reachable by mouse must also be reachable by
keyboard:

- Drag-and-drop has a sibling **"Choose file…"** button (`Message::PickFileRequested`).
- Toggling theme is a button, not a click-on-icon-only widget.
- Opening / closing advanced settings is a footer button. The bottom
  sheet itself can be dismissed by clicking the explicit "Close"
  button inside, in addition to clicking outside (which routes to
  `Message::CloseModals`).
- Error toasts have an explicit close button (× glyph). They never
  rely solely on auto-dismiss.

Focus order matches reading order. iced's default focus ring is
preserved; we do not suppress it.

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
`snora::Toast` notifications anchored at the bottom-end of the
window, so the user's loaded image, preview, and progress remain
visible. The user dismisses an error explicitly when they have read
it (`Message::DismissToast(id)`). Successes use the default
transient lifetime (4 s) — long enough to read, short enough not to
stack.

## Detail kept hidden by default

Per §5.3, advanced settings are **not** shown on the main screen.
They live in a `BottomSheet` that opens via a footer button, can be
dismissed by clicking outside (single sink:
`AppLayout::on_close_modals`), and never blocks access to the
underlying body content visually.

## What we do not do

- We do **not** suppress iced's focus ring.
- We do **not** rely on hover-only interactions.
- We do **not** use color as the only indicator of state.
- We do **not** require drag-and-drop for any operation.
- We do **not** auto-dismiss error notifications.
