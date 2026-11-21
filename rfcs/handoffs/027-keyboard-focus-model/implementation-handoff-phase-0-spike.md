# Developer Handoff — RFC 027 phase 0: the keyboard focus spike (job J13, evidence only; includes the shared GUI session script, former job JS)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 027](../../done/027-keyboard-focus-model.md), including the **update of 2026-11-09** (D1 is reopened: Option A, app-owned model, against Option D, snora's engine, and a hybrid). Previous review: `.git-exclude/reviewed/rfc-031-notice-no-stack-review-2026-11-15.md`. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **The owner's priority for this release is that a keyboard-only user can do everything**; this spike decides how, so the evidence must be honest, including "it does not work".

**This job changes no product code.** The prototypes live as patch files in your review-request folder and are reverted. The only committed file is the shared session script of Step 0.

**Workflow:** `main`; `git pull --ff-only` first; `git log origin/main..` before any push (shared checkout); gates chained for the Step 0 commit; CI waited for; no tags, force-push or history rewriting.

## 0. Evidence plan

Check each question at the cheapest level. Most of this spike can be answered **headlessly** with the harness of RFC 035 (`iced_test`, `UserInterface`: send key events, read event statuses and messages, take a snapshot); a real window is for what needs a real keyboard stack or human eyes.

| # | Question | Level |
|---|---|---|
| Q1 | What exactly does snora 0.57's keyboard engine cover, and how is it wired into `update`/`view`? (`snora::keyboard::traverse`, `Shortcuts`, `Escape`, `AppLayout::shortcuts`/`escape`, the side bar, `design::button`; where focus state lives; how the app can set or read it) | source reading plus a minimal headless prototype |
| Q2 | Are our plain `iced::widget::button`s really unreachable by `traverse`? What happens to a snora-routed Tab when the page holds only plain buttons? | headless: send Tab events, record the focus result |
| Q3 | If the nav (side bar and the mobile bottom bar) and the buttons were snora's (`design::button`), what changes in **look** (our RFC 021 cues, sizes, button styles) and in **layout properties** (hit areas, the RFC 035 properties)? | headless snapshots before/after plus the harness's P1–P3 on the prototype |
| Q4 | **Text-field key ownership.** With a text input focused (the custom-size field), for each of Tab, Shift+Tab, Esc, Enter, Space, Ctrl+1/2/3, Ctrl+O, Ctrl+S, F1, F6, arrows: does the key reach the application, as Captured or Ignored, and what does the field do (type, ignore)? | headless (`UserInterface::update` with key events) **and one real confirmation**: Ctrl+1 and Ctrl+O in the field on native Wayland (`wtype`) must agree with the headless answer, or the headless method is not trusted for keys and you say so |
| Q5 | Can an **app-owned focus model (Option A)** drive plain iced buttons, checkboxes, the slider, radio-like groups and the drop-zone card? Prototype on **two pages only**: Home (nav, drop-zone card, "Choose file…") and the PNG-size block of Customize (preset checkboxes, custom field, Add, slider): a `FocusTarget` enum, an ordered list per page, Tab/Shift+Tab, Enter/Space to the same messages as the mouse, `focus_ring` styling, hand-over to and from the text input. | headless: scripted key events and asserted messages; one snapshot of the ring |
| Q6 | **Hand-over between two owners (the hybrid):** if snora's engine owns its controls and our model the rest, can Tab pass from one to the other without a trap or a double focus? How is "who owns focus now" represented? Is a clean single-owner design possible? | prototype of the simplest hybrid on Home, headless |
| Q7 | Do the opt-in shortcuts (`Digits::SideBar` Ctrl+1..9, Alt+↑, `escape`) collide with ours (`shortcuts.rs`: Ctrl+1/2/3, Ctrl+O, Ctrl+S, Esc, F1)? Which side should own each key? | source plus headless |
| Q8 | **Testability:** can "every interactive control is reachable by Tab, in this order, and activates its message" be a headless property (like P1–P3 of RFC 035) for each option? What would the test look like, and how large is it? | prototype of one such test |
| Q9 | Newer snora: the registry already holds **0.59 and 0.62** (the project pins 0.57). Do their guides (`.git-exclude/upstream/snora/receive/` if present, else the crate READMEs and sources) change the answer to Q1–Q2 (for example plain buttons becoming focusable)? Report only; **do not upgrade**. | reading |
| Q10 | Real look of the ring: one capture of the best prototype's focus ring in Light and in Dark. | real UI or snapshot (state which); worst case plus nothing else |

Real-window rules (GUI evidence rules): unlocked session, isolated `XDG_CONFIG_HOME`/`XDG_STATE_HOME` with a pinned `settings.json`, one instance, window by pid, focus check before every keystroke, assert after each step, stop at the first anomaly, analyse after the run, no repeats without asking. The owner's real `~/.config/logolig/settings.json` mtime before and after.

## 1. Step 0 — the shared GUI session script (the former job JS; one small commit)

Create `scripts/gui-session.sh` from `.git-exclude/review-request/rfc-031-dialog-spike/evidence/session-lib.sh` (itself extracted from the J11 run). It is a library to be `source`d, with separate, documented functions: isolation (`iso_setup`: scratch XDG dirs, pinned `settings.json` written and asserted), `launch` (binary keeps its real name `logolig`; optional `DBUS_SESSION_BUS_ADDRESS`), window selection by pid, `focus_app`, `new_windows`/`count_new_windows`, `capture` (absolute path), `teardown` (`pkill -x logolig`). A header comment states the rules it enforces and a 10-line usage example. Add a git-ignored fixtures folder note only if you actually need one (do not create empty structure). `bash -n` and a dry run that sources it and calls the pure helpers are the gate (no `shellcheck` here). Commit and push on its own; CI must stay green. The spike's real runs then **source this file**.

## 2. Step 1 — the spike (no commit)

Work uncommitted in the checkout, save each prototype as a patch file under `.git-exclude/review-request/rfc-027-focus-spike/evidence/`, and revert; `git status` must be clean at the end (apart from that folder). Where a prototype needs a new test file, the patch contains it.

## 3. Deliverable: `.git-exclude/review-request/rfc-027-focus-spike/README.md` (report only its path)

Per question: the answer, the evidence (pasted output, snapshot paths), what you could not do and why. Then:

1. **A comparison table of A, D and the hybrid** (for each: what it covers, what it costs in lines and touched files measured from your prototypes, look changes, text-field behaviour, shortcut collisions, headless testability, long-term risk with snora releases).
2. **Your recommendation** with reasons and a **frozen-design proposal** the architect can turn into the RFC's design and into the four step handoffs (Home, Result, Customize, Settings): the target list per page, the ordering rule, the activation map, the indicator, focus on screen change, the key-ownership table of Q4, what to do about Q7's collisions.
3. **An honest list of what stays unfixed** (for example screen readers) and of risks.
4. Anything the spike suggests changing in RFC 021 (selection cues) or RFC 034 (sizes) so that the ring and the cues fit together.

## 4. Acceptance criteria

Step 0 pushed with green CI; all ten questions answered (a stated "cannot be done here" with the reason counts); nothing in `crates/`, `Cargo.toml` or `Cargo.lock` changed at the end; the real runs follow the rules; every claim has an artefact.

## 5. Prohibited shortcuts

No snora upgrade or version change; no committed prototype; no change to `shortcuts.rs` behaviour; no timing heuristics; no "works" claim from reading alone where a headless run is possible; no repeats of a real run without asking.
