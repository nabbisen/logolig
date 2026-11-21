# Developer Handoff — RFC 027 J14a: Tab is routed, focus stays in view, one scroll area on Customize

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, 2018+ module style, tests in separate files, English only; relative paths with `cd`). Governing text: [RFC 027](../../done/027-keyboard-focus-model.md), **"Decision after the phase 0 spike" (D′), "Rule from J13b", "Requirement 2026-11-19 (focus must stay in view)" and "Decisions after the J10 build"**. Evidence you already produced: `.git-exclude/review-request/rfc-021-selection-cues-build/README.md` §4 and `evidence/focus-into-view.txt`. Review: `.git-exclude/reviewed/rfc-021-selection-cues-build-review-2026-11-19.md`.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **A keyboard user must never lose sight of the focus and must never be trapped.**

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit 0 / 12); CI waited for each; no tags, force-push or history rewriting. **Scratch hygiene:** `/tmp` is a shared 30 GB tmpfs; delete your own large scratch files (render dumps, build copies) when done.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does a plain Tab / Shift+Tab (key event `Ignored`) reach `snora::keyboard::traverse` and move focus through all Tab stops in layout order, on every page, including out of a focused text field? | headless (key events through the real `shell::view` and the real subscription filter `runtime_event`) | tests; Customize widest: 21 stops |
| Is the focused control always fully inside the visible part of its scroll area after every Tab and Shift+Tab? | headless property over the matrix (every page state × the sizes) | the property, red first on today's tree (your numbers: 11/21, 2/21, 12/21 outside), green after |
| Does Customize have one scroll area, and is the ring complete on all four sides at every control? | headless: structure walk (one `Scrollable` on Customize) and the ring-sides assertion you wrote | tests; snapshots of Customize before/after only where `cmp` differs |
| Do the Tab stops equal the interactive areas of the RFC 035 scan, apart from a named list? | headless | a completeness test with a **named, commented list of controls not yet migrated** (J14b empties it; a stale entry fails) |
| Does it work in a real window? | real UI, **one run**, native Wayland (`wtype`: the keyboard is what is tested), via `scripts/gui-session.sh` | below |

## 1. Work (tests first; one commit per step)

1. **Route Tab.** In `runtime_event` (`app.rs`) an `Ignored` `KeyPressed` of `Named::Tab` (any modifiers) becomes a new message (for example `Message::TabPressed(Modifiers)`); `update` returns `snora::keyboard::traverse(key, modifiers)` (it answers only plain Tab and Shift+Tab; Ctrl/Alt/Cmd+Tab stay untouched) or `Task::none()`. `shortcuts.rs` remains the one owner of shortcuts; nothing is handled twice. A Tab pressed while a text field is focused must leave the field (measured: the field passes Tab); Esc inside the field leaves it first (unchanged). A Tab at the end of the stops leaves the group and the next Tab starts again (snora's rule, adopted: no wrap).
2. **One scroll area on Customize.** Remove the nested scrollable (`advanced_drawer.rs` inside `customize_page.rs`) so the page scrolls as one; keep the "Reset to defaults" button where it is visually; keep `RING_ROOM` only if the ring is still clipped without it (the ring-sides test decides). Layout properties (P1–P3, theme proof) stay green; snapshots of the **other pages** byte-identical.
3. **Focus into view.** Decision rule: implement an iced operation run after the focus move (chain it after `traverse`'s task) that finds the focused control's bounds and the enclosing scrollable and scrolls just enough (`scrollable::scroll_to` / the absolute-offset form) so the control and its ring (2 px outside) are fully visible; at most **about 120 lines including its tests**. If iced's `Operation` API cannot give you both the focused bounds and the enclosing scrollable in one traversal, or if it needs more than that, **stop and report with what you found** (the architect then raises it with the snora team); do not invent a heuristic. No timers.
4. **The properties** (in the RFC 035 harness style, new files under `ui_layout_tests/`): Tab order equals layout order (the stops' bounds, read top-to-bottom, left-to-right, are monotonic within a page, except where a scroll area legitimately changes the order: say what you found); **after every Tab and Shift+Tab the focused control is fully visible** (the property above, for every Customize/Result/Settings/Home state of the matrix); completeness against the hover scan with the named not-yet-migrated list (nav items, drop-zone card, "View results →", Save buttons, preview toggle, Back, Download all, "Add custom size", the two disclosure headers, "Reset to defaults": confirm the list by running the scan). A property that cannot fail is no property: a self-test with a control placed below the fold must fail it.
5. **Docs:** nothing about keyboard support changes yet (the Settings note stays; J14b rewrites `ui-a11y.md` from the finished work).

## 2. The one real run

Native Wayland, `wtype`, `scripts/gui-session.sh` (its functions, not a copy); unlocked session; isolated `XDG_CONFIG_HOME`/`XDG_STATE_HOME` with the pinned settings of your earlier run (all options on); one instance by pid; focus check before every keystroke; **assert each step by a second signal before the next** (crop compare against references) and abort at the first anomaly. Steps: open Customize (Ctrl+2); Tab once → the ring appears on the first stop and nothing else changed; Tab until a stop below the fold (count the presses from the headless list) → it is visible with its ring; Shift+Tab back one → still visible; Tab into the custom-size field, type `12`, Tab out → focus left the field and the field kept `12`. State what could not be seen. Owner's real `settings.json` mtime before and after. One run; ask before a second.

## 3. Required evidence (review request: `.git-exclude/review-request/rfc-027-j14a-engine/README.md`; report only its path)

§0 table filled in; the red and green outputs of the focus-into-view property; the lists of stops per page (before/after); the named not-yet-migrated list; the picture(s) where `cmp` differs; the real-run log and analysis; gates and CI by commit; the test-list diff (named additions, none removed); what you decided under step 3's rule and why.

## 4. Acceptance criteria

Tab and Shift+Tab move through every Tab stop of every page in layout order; the focused control is fully visible after every move on the whole matrix; Customize has one scroll area; the ring is complete on all four sides; Esc/Tab behaviour in the field is as before; every earlier test passes; gates and CI green on each pushed commit; nothing else changes.

## 5. Prohibited shortcuts

No timer or heuristic for scrolling; no keyboard code that duplicates snora's (`traverse` is the one engine); no change to `shortcuts.rs` bindings; no enabling of snora's `Digits`, `escape` or `focus_zone`; no new `#[allow]`; no new package or `design` feature; no migration of buttons to `snora::control` (that is J14b); no threshold change, `#[ignore]` or unnamed exception; no tags, force-push or history rewriting.
