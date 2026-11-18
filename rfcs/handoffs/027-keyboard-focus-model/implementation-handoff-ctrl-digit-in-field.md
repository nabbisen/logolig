# Developer Handoff — RFC 027: a command key must never type into a text field (job J13b)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; read the GUI Evidence and Test Optimization Rules again; relative paths with `cd`). Governing RFC: [RFC 027](../../accepted/027-keyboard-focus-model.md), section "Decision after the phase 0 spike" (key ownership). Evidence: the spike's Q4 (`.git-exclude/review-request/rfc-027-focus-spike/`, `Q4-headless-output.txt`, `q4-real-*`). **Starts after J13a is reviewed.**

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **Defect (reproduced on the real stack, X11):** with the custom-size field focused, **Ctrl+1 types a "1"** into it and does not go Home. iced's `text_input` inserts the event's `text` whatever modifier is held (only Ctrl+C/X/V/A are special), and the field then captures the event, so our subscription (which sees only uncaptured events) never gets the shortcut.

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before pushes; gates chained as in J13a; CI waited for; no tags/force-push.

## Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Which `text` do Ctrl+1/2/3, Ctrl+O, Ctrl+S, F1 carry on the **native Wayland** stack (the spike measured X11 only)? | real UI, one script (sources `scripts/gui-session.sh`) | a probe that logs key events (a scratch test binary or `iced` example outside the repository, or the app with a temporary log line kept as an uncommitted patch); state the backend; no repeats |
| After the fix, does Ctrl+1/2/3 navigate while the field is focused, and does the field still take typing, Backspace, ←/→, Esc, Enter and Ctrl+C/X/V/A? | headless (`UserInterface` with key events carrying the **measured** `text`) | tests |
| Real confirmation of the fix | real UI, one run per backend where the pointer allows (X11 for the click into the field; Wayland if the field can be focused another way — say) | the J13 spike's Q4 script is the model |

## Work

1. **Step 0 (spike inside the job, no commit):** verify the spike's proposed approach — receive command-modified key events whatever their status (`iced::event::listen_raw`) and discard a `PngSizeInputChanged` produced while the command modifier is held — or find a cleaner way (for example intercepting before the field: a wrapper that filters the event, if iced allows it). Report in the review request what you tried and why the chosen way is the cleanest; the architect confirms before step 1 if the chosen way is not the one in the spike.
2. **Step 1:** implement; keep `shortcuts.rs` as the one owner of shortcuts; nothing may be handled twice (a key that navigates must not also be processed by another subscription arm).
3. **Tests (logic and headless):** each command shortcut with the measured `text` while the field is focused → the shortcut's message and the field's text unchanged; Ctrl+C/X/V/A still reach the field; plain typing and Esc/Enter/Backspace/arrows unchanged.
4. Docs: `ui-a11y.md` and the shortcut text only if they now say something false (RFC 018 syncs the rest).

## Required evidence, acceptance, prohibited

Review request `.git-exclude/review-request/rfc-027-ctrl-digit/README.md` (report only its path): the table above filled in, failing-before tests, gates and CI by commit. Acceptance: no command key types into the field on the tested backends; typing behaviour unchanged; nothing handled twice. Prohibited: any timing heuristic; changing which keys are shortcuts; enabling snora's `Digits`/`escape`/`focus_zone`; new `#[allow]`.

## Architect's decision on step 0 (2026-11-18; step 1 may start)

Reviewed in `.git-exclude/reviewed/rfc-027-ctrl-digit-step0-review-2026-11-18.md`.

1. **The wrapper approach is confirmed** (remove the cause, the text on a command chord, instead of reacting to its effect; `listen_raw` plus discarding messages is rejected for the reasons you gave). **One change: wrap once, at the root of the view, not per field.** `command_safe` (rename it `without_command_text` or similar, your choice) wraps the whole `Element` that `view` returns (`app.rs`/`shell.rs`), so every text input now and in future is covered and **nothing has to be remembered**; the five per-field wrap sites and the "a field added later must be wrapped" limit disappear. The widget stays what the prototype shows (stateless, delegates `children`, `diff`, `size`, `size_hint`, `layout`, `operate`, `update` with the stripped event, `mouse_interaction`, `draw`, `overlay`); the same rule (command held, Alt not held; Ctrl+Alt is left alone for AltGr). The event is only changed for **keyboard `KeyPressed` events that carry text**; every other event passes unchanged.
2. **Tests (all headless, with the measured `text`, preceded by the `ModifiersChanged` the window sends):** as in step 0, plus: (a) a **completeness test** that builds each page state (Home, Customize with Advanced open and every option on, Settings, Result) through the real `view`, focuses **each** text input found in the tree and checks that a Ctrl+digit with text is ignored by it (so a future field cannot regress); (b) a test that the strip does not apply to a **non-command** chord (plain `1`, Shift+1 → "!") and to Ctrl+Alt; (c) a test that a command chord's event still reaches the subscription filter as `Ignored` (the path the application uses).
3. **`iced` `advanced` feature** in the workspace `Cargo.toml` is accepted (compile-time API surface only; show `cargo tree -e normal` unchanged and `Cargo.lock` unchanged).
4. **Real confirmation: agreed as proposed, with a stricter script:** one X11 run (the pointer is needed to focus the field), **assert after every step by a second signal before the next step** (the page changed: compare a crop of the page title with the Customize baseline; the field has the typed text: crop compare), abort at the first anomaly; if the first click fails again, find out why with state checks (window geometry, pointer position, `xdotool getmouselocation`) before any second run, and ask me before a third. Native Wayland cannot focus the field without a pointer: stated in the review request, the Wayland measurement of the `text` values you already made stands.
5. Everything else of this handoff stands (docs only if something says something false; no `Digits`/`escape`/`focus_zone`; no timing heuristics; no new `#[allow]`).
