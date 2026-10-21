# RFC 027 — Keyboard focus model

**Status.** Accepted, phased (owner delegation, 2026-10-21): phase 0 is an evidence-only spike; the design below is frozen only after its results are reviewed
**Priority.** P1 (owner priority, 2026-10-21: "enhance keyboard UI/UX")
**Tracks.** Option B of RFC 019: Tab traversal, activation and visible focus for every interactive control.
**Touches.** `crates/app` (state, keyboard subscription, all `ui/*` views and styles), `docs/src/ui-a11y.md`, tests, a manual test checklist. Builds on RFC 019 phase 2 (global shortcuts) and interacts with RFC 021 (selection indicators) and RFC 026 (result layout).

## Summary

RFC 019 phase 1 proved by execution that a keyboard-only user can do nothing today: no key moves focus or activates a control; checkboxes, the slider, the "Advanced settings" header, nav items, buttons and the drop-zone card are unreachable; text inputs take focus only by mouse click. The owner keeps the "accessible" claim and asked for better keyboard UI/UX, so the claim has to become true. This RFC designs the focus model that makes every control reachable, operable and visibly focused.

## Evidence

- Verified (dependency source, iced 0.14.2): `Button::operate` registers no focusable; `Focusable` is implemented only by `text_input` and `text_editor`. Iced does not move focus by itself on Tab; an application must run a focus operation from its own key handling.
- Verified (RFC 019 phase 1, execution): the matrix of keys by screen; nothing responds; Enter inside a mouse-focused custom-size field adds the size; Tab inside a field does not leave it; no keyboard focus indicator exists; the drop-zone card has no hover feedback.
- Verified (source): the only keyboard-related code is the runtime-event subscription for file hover/drop/resize.
- snora 0.25.2 (pinned today) offers only `keyboard::dismiss_on_escape`. snora **0.35 and later** add opt-in **F6 zone navigation** (`snora::keyboard::cycle_zones`, `snora::focus::{Cycle, FocusZone, ZonePresence, next_zone}`) for moving between regions (side navigation, body, footer). Per snora's guides (received 2026-10-21) it provides no widget-level focus and no accessibility tree, focus trapping is "staged, not shipped", and keyboard dismissal is application-owned. Using it requires the upgrade of RFC 028.
- Not verified: whether any released iced or snora makes buttons focusable. snora's 0.53 guides say iced 0.14 has no accessible-name API and snora has no accessibility tree; the registry's newer snora releases still sit on iced 0.14. Phase 0 below checks this before building anything.

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

1. **Phase 0 (spike, evidence only):** confirm the iced/snora limits, **and evaluate F6 zones from snora ≥ 0.35 (RFC 028) as the zone-level layer of this model (F6 / Shift+F6 between side navigation, body and footer; Tab and arrows inside a zone)**, which also shortens the Tab order on every page; prototype Tab over the nav items and the drop-zone card; verify that the iced `focus`/`unfocus` operations let a text input hand focus back to the app model; verify that key events reach the application when a text input has focus (iced marks them captured or ignored; Tab is expected to be ignored by the input).
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

## Decisions (2026-10-21; the owner delegated them with: "Review them carefully by yourself with my fundamental philosophy")

- **D1 — option A (app-owned focus model): decided.** Reasons: it works with iced 0.14 as it is; it is typed and therefore testable and robust (no hidden widget state); it avoids maintaining forked widgets (clean); it gives one behaviour for the mouse and the keyboard path because both emit the same `Message`s (users are not surprised). Option B (fork widgets) costs ongoing maintenance for no user-visible gain; option C leaves users blocked.
- **D2 — indicator shown after keyboard use, hidden on mouse use ("focus-visible" behaviour): decided.** Reasons: "less is more" (no permanent outline noise for mouse users) and it is the convention users already know from browsers and desktop toolkits. To avoid the opposite confusion (a keyboard user not knowing where focus is), the indicator appears on the **first** Tab or arrow key press, shows the *current* control immediately, and a visible shortcut list (RFC 019 phase 2) tells users that keyboard operation exists. The indicator is never colour alone (offset outline plus a thicker border).
- **D3 — scope: one model for all screens, delivered and reviewed in four steps (Home, Result, Customize, Settings): decided.** Reason: a half-covered keyboard path is the most confusing outcome (focus enters a screen and cannot leave); but each step is independently reviewable, and no step is released unless the step before it is complete, so the user is never left in a trap.
- **Phase 0 (handoff to be written when the job is scheduled, J10 in `ROADMAP.md`):** evidence only; confirms the limits listed in "Evidence", prototypes Tab over the nav items and the drop-zone card, checks text-input focus hand-over and which key events reach the application when an input has focus. The design is frozen only after the architect reviews that evidence. Handoffs: not yet written.

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| View and target list drift apart | Medium | Single data source and signature-enforced helpers; tests |
| Interaction with text inputs (focus hand-over) | Medium | Phase 0 spike before design is frozen |
| Large diff in the UI files | Medium | Do RFC 017 (split `app.rs`, UI modules) first |
| No screen-reader support remains | Known | Documented honestly; follow-up RFC when iced allows |
