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
