# RFC 019 — Keyboard accessibility

**Status.** Proposed
**Priority.** P1 for documenting the truth; P2 for the implementation spike
**Tracks.** ABDD commitment "every mouse operation is reachable by keyboard" versus the capabilities of iced 0.14.
**Touches.** `crates/app` (event subscription, focus state, UI styles), `docs/src/ui-a11y.md`, README, manual test checklist.

## Summary

The documentation promises keyboard parity and a preserved focus ring. Dependency source shows that iced 0.14 buttons cannot receive keyboard focus, so a keyboard-only user cannot reach the primary path (choose file, navigate, download). This RFC verifies the claim at run time, then fixes the documentation immediately and designs a keyboard model.

## Evidence

- Verified (dependency source, `iced_widget` 0.14.2): `Button::operate` only calls `operation.container(..)` and traverses its child; it does not register a focusable. A search finds `Focusable` implemented only by `text_input` and `text_editor`. Checkbox, toggler and radio were not found among the implementers either.
- Verified (project source): the drop zone, the nav items, the download buttons, "Back", "Choose file…" and the toggles are all `button`/`checkbox`-family widgets. `drop_zone_button_style` returns `Style::default()` and ignores `Status`, so there is also no visible hover/pressed/focus state for the largest control.
- Verified (snora 0.25.2): its only keyboard helper is `keyboard::dismiss_on_escape`; no focus helpers.
- **Not verified at run time**: actual behaviour of Tab / Enter / Space on the running application, per platform. The owner noted this should be verified soon; step 1 does it.
- Documentation claiming parity: `ui-a11y.md` "Keyboard parity", `functional-spec.md` §14, README.
- Known backlog item: ESC-to-back (never wired).

## Design

**Step 1 — verify (time-boxed, no design commitment).** On Linux (Wayland and X11) and Windows, record what Tab, Shift+Tab, Enter, Space, Esc do on each screen, including text inputs on the Customize page. Output: a table committed as evidence. Use `niri msg` / `xdotool` as the GUI rules suggest.

**Step 2 — make the docs honest.** Replace the parity claim by what step 1 proves, per the dev instructions ("document current platform limitations honestly").

**Step 3 — keyboard model (D1).** Options:

| Option | Description | Cost | Assessment |
|---|---|---|---|
| A | Global shortcuts only: Ctrl/Cmd+O choose file, Esc back/close, Ctrl/Cmd+S download all, 1/2/3 or Alt+1..3 switch Home/Customize/Settings; shown in an in-app shortcut list | small | Delivers a complete keyboard path without focus traversal; discoverable via the list; recommended first |
| B | App-level focus model: `focused: Option<FocusId>`, Tab/Shift+Tab cycle a declared order, Enter/Space activate, visible focus ring in styles | medium-large | Closest to expectation; requires every interactive element to be registered; screen-reader semantics still absent |
| C | Wait for upstream iced focusable buttons | unknown | Leaves users blocked; acceptable only with A as interim |
| D | Document only | none | Not compatible with the owner's inclusive-design priority |

Recommendation: **A now, B as a follow-up RFC if the verification shows Tab traversal is the main gap, C tracked.**

**Step 4 — visible states.** Give every custom button style distinct hover/pressed/disabled/focused appearance (non-colour cue: border thickness or underline), starting with the drop-zone card.

## Out of scope

Screen-reader support (no accessibility tree in iced 0.14 was found in this review; confirm in step 1), high-contrast presets (tracked with the Snora Design System decision DEC-004).

## Tests

Mostly manual (iced has no mature GUI test harness, DEC-009). Deterministic parts are unit-testable: a pure `key → Message` mapping function with table tests; `update()` transitions for shortcuts; style functions return different border widths for different `Status` values.

## Acceptance criteria

A keyboard-only user can: choose a file, wait, download all, go back, and switch pages, on the platforms tested; `ui-a11y.md` states exactly what is supported.

## Risk

Medium for option B; low for A. Option A interacts with text inputs: shortcuts must not fire while a text field has focus.
