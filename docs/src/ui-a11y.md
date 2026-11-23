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

## Keyboard support

**Goal.** Full keyboard parity: everything the mouse can do, the keyboard can do. A
keyboard-only user can choose a file, convert it, download one file or the ZIP, go
back, change every setting and switch pages; the focus is always visible; nobody is
trapped ([RFC 019](../../rfcs/done/019-keyboard-accessibility.md),
[RFC 027](../../rfcs/done/027-keyboard-focus-model.md)).

**How it is checked.** Automated tests drive the real key path (the key event, the
application's subscription, the widget operations that follow) on the real view, on
every page and window size the layout tests use, in both languages.

Real-window runs, on Linux under the niri compositor, with an English interface and
the light theme:

- *X11 backend (XWayland), keys and pointer by `xdotool`:* Tab and Shift+Tab walk the
  stops of the Customize page one by one, the ring moves with every press, and the
  page scrolls to keep a stop below the fold in view.
- *Native Wayland, keys only by `wtype` (no pointer):* Tab 1 to 4 put a ring on each
  stop in turn, the fourth on the drop-zone card; **Enter on the focused drop-zone
  card opens exactly one file chooser**, and Esc closes it and leaves the application
  on the start screen; Ctrl+2 goes to Customize; Tab four times reaches the first
  checkbox and **Space ticks it**; Shift+Tab moves the ring back; Ctrl+3 goes to
  Settings.

Run again on the release candidate, in real windows as above:

- *Native Wayland:* the sequence above again, and in addition the focused active navigation
  item keeps its whole accent bar and a whole ring; on the Settings page the arrow keys on
  the theme group change the theme at once (↓ makes the page dark, ↑ back); a PNG chosen in
  the file chooser by keyboard shows the Result screen; Ctrl+S pressed twice opens exactly
  one save window.
- *X11 backend (XWayland), keys and pointer:* Tab out of the custom-size field keeps its
  text and puts the ring on the next stop; Ctrl+1 in the focused field goes Home and
  types nothing (and Ctrl+2 back shows the field unchanged); Esc in the field leaves it,
  and Tab then starts at the first stop.
- *Native Wayland, the disclosure controls:* Tab to the "Advanced settings" header and
  Space opens the section, Space closes it, and the ring stays on the header each time;
  after opening, the next Tab goes on from the header (not back to the start). On the Result
  screen the ring stays on the preview toggle through Space (open) and Space (close); Tab
  after it is checked by the automated tests only.

Not run in a real window: the Japanese interface (the automated layout tests cover it).

Windows and macOS have not been tested (on macOS the command key is Cmd instead of
Ctrl; that is the intended mapping and is not verified). Non-Latin keyboard layouts are
covered by unit tests of the key mapping only, not by a run on such a layout.

**Moving: Tab and Shift+Tab**

- Tab moves the focus to the next control, Shift+Tab to the previous one. Every
  control is a stop: the navigation items, the drop-zone card and "View results →",
  "Back", every card's "Save", the preview toggle and "Download all (ZIP)", every
  checkbox, every group of choices (one stop for the whole group), every text field,
  "+ Add custom size", the "Advanced settings" header, "Reset to defaults" and the
  close control of a message.
- The order is the order on the screen: the side navigation first (on a narrow
  window the bottom navigation comes last), then the page from top to bottom and,
  within a row, from left to right. A page's fixed top and bottom parts (the
  "Back" row, "Download all", "Reset to defaults") come before and after what
  scrolls.
- **There is no wrap.** After the last stop the focus leaves (nothing is focused) and
  the next Tab starts again at the first one; Shift+Tab at the first stop does the
  same the other way round. The same happens after a control that had the focus goes
  away (Back, a page change) or after Esc in a text field: nothing is focused and
  the next Tab starts at the first stop.
- The focused control is drawn with a **ring**: a 2 px outline in the text colour
  (black on the light theme, white on the dark one), 2 px outside the control. It
  is separate from the marks of selection, so a checked box is never confused with
  a focused one.
- **The focused control is always in view.** When it lies in a part that scrolls,
  the page scrolls by the least amount that shows it, with room for its ring.
- Ctrl, Alt or Cmd with Tab is not Tab and does nothing here.
- A text field passes Tab on: the focus leaves the field and the field keeps its
  text. Esc inside the field leaves it first.

**Operating: Enter, Space, the arrow keys**

- On a focused button, card or navigation item, **Enter or Space does exactly what a
  click does**, once.
- On a checkbox, Enter or Space toggles it.
- On a group of choices (language, theme, "View as", "Surface", SVG conversion,
  resize algorithm) the arrow keys move **and choose**: ↑ ↓ ← → change the choice at
  once, wrapping at the ends. Enter and Space on the group change nothing, as a click
  on the chosen option does not. A group that is disabled (the Surface group while
  the transparency checker is shown) is not a stop and takes no key.
- Choosing the language with the arrow keys therefore changes the language at once.
- On the "Advanced settings" header (Customize) and the preview toggle (Result screen), Enter
  or Space opens or closes the section below it and **the focus stays on the control**, so the
  ring does not disappear and Tab goes on from there: into the section that just opened, or
  to the control after it when the section closed.

**Global shortcuts**

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
Alt on top of Ctrl make a different combination that is not bound. Ctrl+O is a
shortcut of the Home page only: on Customize and Settings there is no "Choose
file…" control, so it does nothing there.

**Text fields**

- While a text field has focus, the shortcut keys still work and never type into
  the field: Ctrl+1 / 2 / 3 and F1 change the page. The field keeps its own keys
  (typing, Backspace, the arrows, Enter, Ctrl+C / X / V / A). Enter in the
  custom-size field adds the size.
- The operating system's file chooser and save dialogs can be operated with the
  keyboard once they are open.

**Known behaviour**

- A mouse click on the "Advanced settings" header or the preview toggle leaves the focus ring
  on that control, as Space does (the application cannot tell the two presses apart); the ring
  goes at the next click elsewhere or at the next Tab.
- After Enter on a navigation item that changes the page, nothing is focused (the
  page has a different number of controls) and the next Tab starts at the first
  stop, so a keyboard user passes the three navigation items again to reach the page.
- When a conversion takes a very long time (a 50-second test image), shortcuts
  pressed meanwhile were not acted on at once: Ctrl+O and Ctrl+S opened nothing,
  and Ctrl+2 took effect only after the conversion had finished. Typical logos
  convert in well under a second.
- Arrow and paging keys do not scroll a page (Tab does, by moving the focus).
- The drop-zone card shows no hover feedback; the navigation items and the download
  buttons do.
- Window-close shortcuts are not handled by the application; they depend on the
  window manager.

**Not covered**

- Screen readers: iced 0.14 exposes no accessibility tree, so no screen reader reads
  the interface.

## Status, not just color

State is communicated by a **shape** in addition to color, so users with low
vision or color-blindness do not lose information. Each cue has one meaning:

- Active page (navigation): a filled item, a **bold label** and a thin
  **accent bar** on the edge facing the content (the right edge of the side
  bar, the top edge of the bottom bar).
- Chosen option of a single choice (language, theme, View as, Surface, SVG
  conversion, resize algorithm): a **filled dot in a ring** (a radio group).
- Checked option: a **box with a tick** (a checkbox). The marks are drawn by the
  interface engine in the accent blue on the light theme (5.2:1 and 4.5:1 against
  the page, measured) and in white on the dark theme (13.8:1); the outlines of
  unchecked boxes and unselected options are the text colour (10:1 or more).
- Busy: the **Loader** icon before the message.
- Success: the **Check** icon on the Result headline.

The marks are icons and drawn shapes from the bundled icon font, never text
characters, so they do not depend on the fonts installed. They live in
`crate::ui::choice` (choices, checkboxes) and `crate::ui::cues` (busy, success).
Errors appear as toasts (next section); there is no text marker for them.

## Contrast

The rule (RFC 034): in **Light and Dark**, every text reaches **7:1** (WCAG AAA) against the
surface it is drawn on, secondary text included; icons and the borders and marks of controls
reach **3:1** (WCAG 1.4.11). This is a measurement of the 1.29.0 interface, not a promise about
any other release; it was made on the headless renderer (CPU, bundled fonts), not in a real
window, and says nothing about Windows or macOS beyond that the colours are the same.

- **One place defines the two themes** (`ui/theme.rs`): iced's Light and Dark with the accent
  moved (Light: `#3C47B8`, white text 7.5:1; Dark: `#8A93F7`, black text 7.6:1), the success,
  warning and danger colours moved so that toast text reads at 7:1, and **every tier's text
  lifted to 7:1 against its own fill**. Secondary text is a **solid** colour derived against
  the page and the card it sits on (it used to be the body colour at 60 % alpha, 5.0 to 5.7:1);
  on the dark theme the card leaves little room, so there it is close to the body text and
  the hierarchy is carried by size and weight.
- **Borders and marks** of buttons and text fields are derived to 3:1 against the page and a
  card (they were 1.2 to 1.8:1). The placeholder of a text field is secondary text (7:1).
- **Exempt, and why:** the edges of cards and containers that only frame other content (the
  controls inside carry their own borders); the preview's mock-up colours (an illustration of
  a browser tab and a phone home screen, not interface text; they are measured as drawn and
  pass); a disabled control; the focus ring (drawn by the interface engine at 4.58:1 or more
  against what is behind it, checked in the ring tests).
- **How it is checked:** `ui_layout_tests/contrast.rs` finds every text piece of every screen
  in both themes in the CPU-rendered picture and measures its ink against its ground (1,100
  pieces, toasts of all three intents included); `ui_layout_tests/contrast_inventory.rs`
  checks the colours the helpers and the themes give, including the states a resting screen
  does not show (hover, pressed, the drop-zone hover tint), and fails if a colour helper is
  not in the list. The table is what the inventory measures (ratio before → after, 1.28.0 →
  this release):

| Pair (drawn on) | need | Light before → after | Dark before → after |
|---|---|---|---|
| body text on the page | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| body text on a card | 7:1 | 15.58 → **15.58** | 7.45 → **7.45** |
| secondary text on the page | 7:1 | 5.74 → **9.64** | 5.00 → **10.54** |
| secondary text on a card | 7:1 | 5.22 → **7.15** | 3.83 → **7.14** |
| app name on the page | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| section label on a card | 7:1 | 15.58 → **15.58** | 7.45 → **7.45** |
| file name on the page | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| page title on the page | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| group heading on the page | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| drop-zone headline on the card | 7:1 | 15.58 → **15.58** | 7.45 → **7.45** |
| drop-zone headline on the hover tint | 7:1 | 11.14 → **18.81** | 5.01 → **7.44** |
| drop-zone secondary text on the hover tint | 7:1 | 4.60 → **8.64** | 2.84 → **7.13** |
| drop-zone secondary text on the card | 7:1 | 5.22 → **7.15** | 3.83 → **7.14** |
| inactive navigation label on the sidebar | 7:1 | 5.22 → **8.51** | 3.83 → **8.79** |
| inactive navigation label on the hover fill | 7:1 | 4.93 → **7.25** | 3.30 → **7.10** |
| active navigation label on its fill | 7:1 | 13.28 → **13.28** | 6.02 → **7.11** |
| text on the hover fill of an auxiliary button | 7:1 | 13.28 → **13.28** | 6.02 → **7.11** |
| placeholder of a text field | 7:1 | 2.94 → **9.64** | 4.01 → **10.54** |
| text on the primary button, rest | 7:1 | 4.43 → **7.54** | 4.43 → **7.60** |
| text on the primary button, hovered | 7:1 | 3.58 → **8.17** | 3.58 → **9.21** |
| text on the primary button, pressed | 7:1 | 4.43 → **8.91** | 4.43 → **10.81** |
| tier background.base: its text on its fill (toasts, default widgets) | 7:1 | 21.00 → **21.00** | 11.00 → **11.00** |
| tier background.weak: its text on its fill (toasts, default widgets) | 7:1 | 15.58 → **15.58** | 7.45 → **7.45** |
| tier background.strong: its text on its fill (toasts, default widgets) | 7:1 | 13.28 → **13.28** | 6.02 → **7.11** |
| tier primary.base: its text on its fill (toasts, default widgets) | 7:1 | 4.43 → **7.21** | 4.43 → **7.11** |
| tier success.base: its text on its fill (toasts, default widgets) | 7:1 | 6.61 → **7.10** | 6.91 → **7.30** |
| tier warning.base: its text on its fill (toasts, default widgets) | 7:1 | 6.05 → **7.15** | 8.12 → **8.12** |
| tier danger.base: its text on its fill (toasts, default widgets) | 7:1 | 4.83 → **7.10** | 4.83 → **7.16** |
| border of a button or text field, against the page | 3:1 | 1.58 → **4.21** | 1.83 → **4.61** |
| border of a button or text field, against a card | 3:1 | 1.17 → **3.13** | 1.24 → **3.12** |
| border of the reset button, against the page | 3:1 | 1.93 → **7.16** | 2.04 → **4.60** |
| border of the reset button, against a card | 3:1 | 1.43 → **5.31** | 1.38 → **3.11** |
| the active page's bar, against the page | 3:1 | 4.61 → **7.54** | 3.70 → **4.99** |
| the active page's bar, against the sidebar | 3:1 | 3.42 → **5.59** | 2.51 → **3.38** |
| the primary button's fill, rest, against the page | 3:1 | 4.61 → **7.54** | 2.99 → **4.99** |
| the primary button's fill, hovered, against the page | 3:1 | 4.61 → **8.17** | 2.99 → **6.05** |
| the primary button's fill, pressed, against the page | 3:1 | 4.61 → **8.91** | 2.99 → **7.10** |
| the drop-zone accent border against the page | 3:1 | 3.73 → **7.54** | 3.70 → **4.99** |

Captions (11 to 13 px text, such as the sizes on a result card and the lines under a field) carry
nothing that is not also said by a larger label next to them; they are secondary text at 7:1 too.

## Labels, not glyphs

Every interactive widget has a meaningful text label even when its
visual representation is short. The labels are the texts of the controls
themselves (translated through `logolig-i18n`). iced 0.14 exposes no
accessibility tree, so no screen reader reads them yet. (An earlier module
of label constants was never used and was removed in 1.28.0.)

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
