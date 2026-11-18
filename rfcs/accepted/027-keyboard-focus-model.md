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

## Requirement found by RFC 019 phase 2 (architect, 2026-10-25)

A focused text field takes some shortcut keys before the application sees them: with the custom-size field focused, **Ctrl+1 types a "1"** instead of going Home, Ctrl+O does nothing, while Ctrl+3 and F1 still navigate (observed on X11). The focus model must therefore **know when a text field owns the keyboard** and define, per key, whether a global shortcut or the field wins (a shortcut such as Ctrl+digit must never insert text). Esc inside a field (leave the field) is to be observed and specified in phase 0.

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
- **Phase 0 (handoff to be written when the job is scheduled, J13 in `ROADMAP.md`; starts on snora 0.57 after RFC 032):** evidence only; confirms the limits listed in "Evidence", prototypes Tab over the nav items and the drop-zone card, checks text-input focus hand-over and which key events reach the application when an input has focus. The design is frozen only after the architect reviews that evidence. Handoffs: not yet written.

## Update 2026-11-09 — snora 0.57 changes the starting point (architect)

snora's notice of 2026-11-08 (0.57.0) and its guides 0.54–0.57 contradict the "Not verified" bullet and the premise of D1:

- **Since 0.55 snora's own controls are keyboard-reachable** (tabs, crumbs, menus, notices, chips, `design::button`, toast ×, the side bar) once `update` routes Tab (`snora::keyboard::traverse`); Enter/Space press; a focus ring is drawn (3:1); overlays block the keys behind them; focus returns to the opener after a modal and moves to a neighbour after a removal (0.56); a click presses without focusing (0.56).
- **Opt-in shortcuts (0.57):** `AppLayout::shortcuts` with `Digits::SideBar` gives Ctrl+1..9 for the side bar and Alt+↑ for crumbs; `AppLayout::escape` routes Escape per overlay. These overlap RFC 019 phase 2 (our Ctrl+1/2/3 and Esc) and must not be bound twice.
- **What snora does not cover:** checkboxes, the slider, radio-like groups, text inputs and the drop-zone card; plain `iced::widget::button` is not focusable ("aren't affected"). Our UI is built almost entirely from plain iced buttons, so **the upgrade alone gives our screens no keyboard reach** (RFC 032, evidence).
- F6 zone navigation (0.35+) is the older layer; whether it is still needed once Tab works across the page is a phase 0 question.

**Consequence for D1.** D1 chose option A because no released engine could make buttons focusable. That fact has changed. D1 is therefore **reopened until phase 0 reports**: the spike (J13, on snora 0.57 after RFC 032) compares **Option A** (app-owned model, as above) with a new **Option D: adopt snora's engine** for everything it covers (our nav as snora's side bar, our buttons as `design::button`, `traverse`, `shortcuts`, `escape`) and keep an app-owned model only for what it lacks (checkboxes, slider, radio groups, text inputs, the drop-zone card), including how the two focus owners hand over to each other. The spike must also answer: does adopting snora's controls change our look (RFC 021's cues, the sidebar, button styles) and cost the custom styling we have; are the Ctrl+digit shortcuts in a focused text field still safe; what does a hybrid cost against A. D2 (indicator visibility) and D3 (one model, four reviewed steps) stand; D2 may be satisfied by snora's ring on its controls and must then be consistent with the ring we draw on the rest. The design is frozen only after the architect reviews the spike.

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| View and target list drift apart | Medium | Single data source and signature-enforced helpers; tests |
| Interaction with text inputs (focus hand-over) | Medium | Phase 0 spike before design is frozen |
| Large diff in the UI files | Medium | Do RFC 017 (split `app.rs`, UI modules) first |
| No screen-reader support remains | Known | Documented honestly; follow-up RFC when iced allows |

## Decision after the phase 0 spike (architect, 2026-11-15)

Spike: `.git-exclude/review-request/rfc-027-focus-spike/` (reviewed in `.git-exclude/reviewed/rfc-027-focus-spike-review-2026-11-15.md`). The architect also trial-built the application on snora 0.62 in a scratch copy: it compiles with no code change and every test passes (RFC 036).

- **D1 is decided again: Option D′.** Every interactive element becomes a `snora::control` (snora ≥ 0.59; the project moves to 0.62 by RFC 036). iced's own focus is the **single owner**; there is no application-owned focus model, no target list to keep in step, no click reconciliation. Reasons: it keeps our look (our styles pass through unchanged); it gives one owner (no hybrid with two sources of truth); the Tab order is the layout order; the ring contrast, Enter/Space and a ≥ 24 px target are guaranteed by the dependency; and the completeness check (every hover-scan area has a focus target) is automatic and headless. Option A stays the **documented fallback** if a later snora release removes `control` (the spike's patch and its tests carry over); Option D (snora's design buttons and rail) and the hybrid are **rejected** (look change, two new packages, two owners).
- **What snora still lacks, and the answer.** The checkbox, the slider and the two pick lists are not focus targets in any snora version. Decided: **checkboxes become a `control` row** (mark plus label; Enter/Space toggle; one hit area for the whole row); the **SVG quality slider becomes three labelled choices** (RFC 034 D5, brought forward into this release because a keyboard user must be able to change every setting); the **two pick lists (resize algorithm, trace preset) become radio-like groups of `control` rows** in the style RFC 021 defines. These are visible changes and are decided here so that the look is designed once with RFC 021.
- **The ring (D2 refined by Q10).** A **neutral, high-contrast ring, 2 px, offset 2 px outside the control** (snora's, text colour in both themes); the **accent colour is for selection only** (RFC 021 amended). The ring never moves the layout and is not shown for a click (snora: a click presses without focusing).
- **Order rule.** Layout order, navigation first; no wrap: focus leaves the last control and the next Tab starts again (snora's rule, adopted so there is one behaviour).
- **Key ownership (spike Q4).** The text field owns printable text, Space, Enter, Backspace, ←/→, Esc (to leave it) and Ctrl+C/X/V/A; it passes Tab, Shift+Tab, ↑/↓, F1, F6. The application owns every other command-modified key **even while a field is focused**. This **fails today** on the real stack: Ctrl+1 types "1" into the custom-size field and does not navigate (reproduced on X11). It is fixed by its own small job (J13b), because it is a defect independent of the model. `shortcuts.rs` stays the one owner of shortcuts; snora's `Digits`, `escape` and `focus_zone` are **not enabled**.
- **Ctrl+O on Customize and Settings** does nothing, by design: those pages have no "Choose file…" control, and the shortcut list is the visible documentation. The shortcut list text will say "(on the Home page)"; no other behaviour change (decided; the keyboard rule "a shortcut does what the visible control of the page does").
- **Delivery (D3 stands):** four reviewed page steps (Home, Result, Customize, Settings), each migrating that page's buttons and rows to `control`, adding the headless Tab-order/Enter-equals-click/completeness tests, and nothing is released unless the previous step is complete. The Customize step carries the slider and pick-list replacements. They run **after** J9 (RFC 026) and J10 (RFC 021), which settle layout and cues first.
- **Tests to commit with each step (from the spike):** Tab order equals the expected list; Enter/Space produce exactly the message of the click for every control; completeness against the hover scan (RFC 035); a ring snapshot per theme at the headless level; one real key sequence per release (Tab, Enter, Esc, Ctrl+1 in the field), with the number of tool calls stated.
- **Not solved, said openly:** screen readers (iced 0.14 exposes no accessibility tree).

## Rule from J13b (architect, 2026-11-18): a command chord carries no text

`ui/command_keys.rs::without_command_text` wraps the root of the view: **every widget below it sees a command chord (Ctrl, or Cmd on macOS, without Alt) with `text: None`.** Consequence for all present and future widgets: **read the key, never the text, to recognise a command chord**. Ctrl+Alt (AltGr) is not a command chord and keeps its text. The doc comment of the module is completed with this sentence in the next commit that touches the file (the first RFC 027 page step); it is not worth a CI round by itself.

## Amendment 2026-11-18 (snora 0.65): the form controls come from snora

The decision of 2026-11-15 said that checkboxes become hand-built `control` rows, the slider becomes three labelled choices and the two pick lists become radio-like groups of `control` rows. snora 0.65.0 (RFC-134) now ships exactly these controls with the keyboard built in: **`snora::checkbox`, `snora::toggler`, `snora::radio_group` (one Tab stop for the group; ↑ ↓ ← → move and select, wrapping), `snora::select`, `snora::slider`**, on the engine path (coloured by our iced theme, sized in ems of our text, ringed, a 24 px target, a click and a key publish the same message). Decided:

- **Use snora's controls instead of building our own:** `snora::checkbox` for every checkbox row, `snora::radio_group` for every single-choice group (language, theme, "View as", "Surface", the quality choices, the resize-algorithm and trace-preset choices when short enough), `snora::toggler` only where a switch is the right metaphor (none planned). **`snora::select` and `snora::slider` are not adopted** for now: the slider stays replaced by three labelled choices (RFC 034 D5 is about clarity, not only about the keyboard), and the select has a cost (it is opened by the application and closed through the engine's one `on_close_menus` message; its arrow keys work only inside `snora::render`) that three- or five-option groups do not need. Revisit if a list grows.
- The checkbox, radio and selection **look** is settled by RFC 021's job J10 (which builds on these controls), so the cues are designed once; this RFC's page steps (J14) then migrate the remaining buttons, cards and rows to `snora::control` and add the Tab-order, Enter-equals-click and completeness tests; they no longer rebuild form controls.
- Everything else in the decisions of 2026-11-15 stands (D′, neutral ring, no wrap, key ownership, `shortcuts.rs` the one owner).

## Requirement 2026-11-19 (from the J10 phase 0 study): focus must stay in view

After RFC 021 the Customize page holds 22 Tab stops and scrolls; iced's `scrollable` does not scroll a focused child into view by itself. A keyboard user tabbing into a control that is below the fold would see no focus at all, which defeats the model. **Acceptance addition for every page step of J14:** after each Tab (and Shift+Tab) the focused control lies fully inside the visible part of its scroll area; if the engine does not do this (the J10 build reports the numbers; snora 0.62's `control` may or may not), the application scrolls it into view by an iced operation (`scrollable::scroll_to` or the `snora` equivalent) after every focus move it can observe, or by asking snora for it. This is a **headless property** (RFC 035 style): "for every Tab stop, `visible_bounds == bounds` after the Tab", over the page states and sizes of the matrix. If neither iced nor snora can report a focus change to the application, that is raised with the snora team before J14's Customize step starts.
