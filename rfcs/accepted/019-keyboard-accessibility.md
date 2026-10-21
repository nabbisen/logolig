# RFC 019 — Keyboard accessibility

**Status.** Accepted, phased (owner, 2026-10-21). Phase 1 = run-time verification and honest documentation (steps 1–2); the keyboard model (steps 3–4) is handed off after the evidence exists.
**Priority.** P1 for documenting the truth; P2 for the implementation spike
**Handoffs.** [rfcs/handoffs/019-keyboard-accessibility/](../handoffs/019-keyboard-accessibility/README.md)
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

## Decisions and phasing (2026-10-21, applying the owner's philosophy: clean, safe, robust; users must not be misled)

- **Direction for step 3 (D1): option A, global shortcuts, first.** Reason: it delivers a complete, predictable keyboard path with little code and no per-widget focus machinery, and it is robust (no hidden focus state to get out of sync). Option B (focus model with Tab traversal and a visible focus ring) becomes a follow-up RFC **only if** the phase-1 evidence shows that Tab traversal is the real gap. Option C (wait for upstream) is tracked, not relied on. The exact key bindings are finalised in the step-3 handoff; constraints already fixed: no shortcut fires while a text input has focus; shortcuts are listed in the application (one short discoverable list, in `en` and `ja`); platform modifier is Ctrl, or Cmd on macOS.
- **Phase 1 (handoff 007):** step 1 (verify by execution, on every platform available) and step 2 (make the documentation say exactly what is and is not supported). Documentation edits are limited to keyboard and focus statements. No application code changes.
- **Why step 2 is not optional:** the README, `ui-a11y.md` and `functional-spec.md` currently promise keyboard parity; users with a keyboard-only workflow who rely on that promise are being misled. This is also a prerequisite named in RFC 020 for the first AUR release.
- **Step 1 platform scope:** Linux (Wayland as the maintainer's session; X11 through the winit X11 backend or XWayland) is the minimum. Windows and macOS only if a machine is available; if not, they are reported as "not tested", never inferred.

## Phase 1 results (architect review, 2026-10-21)

Executed on Linux (X11 backend with `xdotool`, native Wayland with `wtype`) against the 1.26.2 release build. **A keyboard-only user can do nothing in the application today.** No key moves focus or activates a control; checkboxes, the slider, the "Advanced settings" header and the nav items are unreachable; text inputs receive focus **only by mouse click** (Tab does not reach them, Tab inside a field does not leave it). Typing works in a mouse-focused field and Enter in the custom-size field adds the size (`on_submit`). The drop-zone card has no hover feedback; nav items and download buttons do. There is no key-handling code in the application (comments only). Evidence limits: native Wayland had no positive control (inferred from identical code and from the X11 result), Windows and macOS were not tested. The documentation (`ui-a11y.md`, `functional-spec.md` §14) was corrected to say exactly this.

**Consequence for the decision:** option A (global shortcuts) delivers the main path (choose file, download all, back, switch pages) but not the Customize page (checkboxes, slider, text fields). The evidence therefore confirms that Tab traversal (option B) is a real gap, as the RFC anticipated.

**Revised direction (architect):**

- **Phase 2 = option A**, as already decided, scoped to the main path plus the in-app shortcut list; no focus machinery.
- **Option B becomes its own RFC** (focus model, visible focus indicators, Customize page), written after phase 2, because it changes every interactive widget and needs its own design review. Until then the documentation states the limitation.
- **README tagline (owner decision):** "A local-first, accessible logo asset generator GUI" is stronger than the evidence for keyboard users. The architect recommends tempering the claim until option A and B land (for example dropping "accessible" from the tagline, or stating what is supported). The tagline is positioning text and belongs to the owner.
- Two source comments still claim keyboard accessibility (`ui/drop_zone.rs:6`, `ui/preview_panel.rs:134`); they are corrected in phase 2.

## Out of scope

Screen-reader support (no accessibility tree in iced 0.14 was found in this review; confirm in step 1), high-contrast presets (tracked with the Snora Design System decision DEC-004).

## Tests

Mostly manual (iced has no mature GUI test harness, DEC-009). Deterministic parts are unit-testable: a pure `key → Message` mapping function with table tests; `update()` transitions for shortcuts; style functions return different border widths for different `Status` values.

## Acceptance criteria

A keyboard-only user can: choose a file, wait, download all, go back, and switch pages, on the platforms tested; `ui-a11y.md` states exactly what is supported.

## Risk

Medium for option B; low for A. Option A interacts with text inputs: shortcuts must not fire while a text field has focus.
