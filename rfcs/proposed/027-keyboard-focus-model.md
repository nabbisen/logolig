# RFC 027 — Keyboard focus model

**Status.** Proposed
**Priority.** P1 (owner priority, 2026-10-21: "enhance keyboard UI/UX")
**Tracks.** Option B of RFC 019: Tab traversal, activation and visible focus for every interactive control.
**Touches.** `crates/app` (state, keyboard subscription, all `ui/*` views and styles), `docs/src/ui-a11y.md`, tests, a manual test checklist. Builds on RFC 019 phase 2 (global shortcuts) and interacts with RFC 021 (selection indicators) and RFC 026 (result layout).

## Summary

RFC 019 phase 1 proved by execution that a keyboard-only user can do nothing today: no key moves focus or activates a control; checkboxes, the slider, the "Advanced settings" header, nav items, buttons and the drop-zone card are unreachable; text inputs take focus only by mouse click. The owner keeps the "accessible" claim and asked for better keyboard UI/UX, so the claim has to become true. This RFC designs the focus model that makes every control reachable, operable and visibly focused.

## Evidence

- Verified (dependency source, iced 0.14.2): `Button::operate` registers no focusable; `Focusable` is implemented only by `text_input` and `text_editor`. Iced does not move focus by itself on Tab; an application must run a focus operation from its own key handling.
- Verified (RFC 019 phase 1, execution): the matrix of keys by screen; nothing responds; Enter inside a mouse-focused custom-size field adds the size; Tab inside a field does not leave it; no keyboard focus indicator exists; the drop-zone card has no hover feedback.
- Verified (source): the only keyboard-related code is the runtime-event subscription for file hover/drop/resize.
- snora 0.25.2 offers only `keyboard::dismiss_on_escape`.
- Not verified: whether newer iced or snora releases add focusable buttons (the registry holds snora up to 0.52; iced 0.14 is pinned by snora 0.25). Phase 0 below checks this before building anything.

## Goals and non-goals

Goals: (1) Tab / Shift+Tab visit every interactive control in a predictable order on every screen; (2) Enter and Space activate the focused control (Space toggles checkboxes; arrows operate the slider and picker groups); (3) the focused control is always visibly marked, **not by colour alone**; (4) focus is never lost or trapped; (5) behaviour is identical for the mouse and keyboard paths (same messages).

Non-goals: screen-reader support (iced 0.14 exposes no accessibility tree; confirm in phase 0), changing visual design beyond the focus indicator, global shortcuts (RFC 019 phase 2).

## Design options

| Option | Description | Assessment |
|---|---|---|
| A | **App-owned focus model.** `AppState.focus: Option<FocusTarget>` where `FocusTarget` is a typed enum of every focusable control; one function per screen returns its **ordered list of targets**; Tab/Shift+Tab move through that list; Enter/Space map the focused target to the existing `Message`; views read `state.focus` to draw the indicator. Text inputs are focused through iced's `focus` operation when their target becomes current, and leave focus on Tab. | Works with iced 0.14 as it is; no per-widget fork; typed, testable (pure functions); risk: the list and the view must not drift apart |
| B | Fork or wrap iced widgets to make buttons `Focusable` and use iced's own `focus_next` | Closest to upstream behaviour but means maintaining widget code; high cost |
| C | Wait for upstream | Leaves users blocked; tracked in phase 0 only |

**Recommendation: A.** To prevent drift, the view builders take the target as an argument and compare it with `state.focus` (a single helper `focus_ring(target, state)` draws the indicator), and a test asserts that every target in a screen's list is rendered exactly once (the test iterates the list; rendering cannot be asserted without a GUI harness, so the list is built from the same data the view uses).

## Proposal

1. **Phase 0 (spike, evidence only):** confirm the iced/snora limits; prototype Tab over the nav items and the drop-zone card; verify that the iced `focus`/`unfocus` operations let a text input hand focus back to the app model; verify that key events reach the application when a text input has focus (iced marks them captured or ignored; Tab is expected to be ignored by the input).
2. **Targets and order** (reading order, top-left to bottom-right; nav first, then the page): Home/Empty: nav items, drop-zone card, "Choose file…" is the card itself, history "View results"; Home/Result: nav, Back, each card's Save button, Preview toggle, Download all; Customize: nav, PNG preset checkboxes, custom size input and Add, SVG slider, Keep transparency, Advanced header, then the Advanced controls when expanded, Reset; Settings: nav, language group, theme group. Focus moves only among controls that are currently shown.
3. **Activation:** Enter and Space on buttons and cards; Space on checkboxes; Left/Right (or Up/Down) on the slider and within a radio-like group (RFC 021); Enter in a text input submits (already works).
4. **Indicator:** a 2 px outline in the theme's primary colour plus a shape cue (offset outline, not a fill change), visible in light, dark and high-contrast; it must also appear for the drop-zone card, which has no hover state today. The indicator is drawn only when the keyboard is in use (after a key press) and hides on mouse movement, to avoid visual noise for mouse users ("less is more").
5. **Focus on screen change:** after a navigation the focus lands on the first control of the new page (the page heading is not focusable); after Back, on the history card or the drop-zone card; after a toast appears, focus is unchanged (toasts are dismissible with Esc from RFC 019 phase 2).
6. **No traps:** Esc leaves a text input and returns to the model; there is always a next target.
7. **Documentation:** `ui-a11y.md` "Keyboard support (current)" is updated from the evidence of the finished work, never before.

## Tests

- Pure unit tests: ordering for each screen and state (for example Advanced collapsed vs expanded; Result with and without monochrome cards); wrap-around at both ends; Shift+Tab reverse; skipping of hidden targets; activation mapping (target → message) equals the mouse mapping.
- Update tests for focus after navigation and after Back.
- Manual matrix with screenshots on Linux Wayland and X11 (the RFC 019 harness), Windows and macOS if available; each cell either shows behaviour or says "not tested".
- A regression guard: a test that fails when a new interactive control is added without a target (for example a list of all `Message` variants emitted by views is not possible; instead require every `button`/`checkbox` helper in `ui/` to accept a `FocusTarget`, enforced by the helper's signature).

## Acceptance criteria

A keyboard-only user can choose a file (through the OS dialog), convert, download one file and the ZIP, go back, change every setting, and switch pages; the focus is always visible; the documented matrix proves it on the tested platforms.

## Decisions requested

- **D1** Option A as recommended.
- **D2** Indicator that appears only after keyboard use (recommended) versus always visible.
- **D3** Scope: all screens in one release, or Home first and Customize/Settings next? The architect recommends all screens in one job series (it is one model), delivered in this order: Home, Result, Customize, Settings, each reviewed.

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| View and target list drift apart | Medium | Single data source and signature-enforced helpers; tests |
| Interaction with text inputs (focus hand-over) | Medium | Phase 0 spike before design is frozen |
| Large diff in the UI files | Medium | Do RFC 017 (split `app.rs`, UI modules) first |
| No screen-reader support remains | Known | Documented honestly; follow-up RFC when iced allows |
