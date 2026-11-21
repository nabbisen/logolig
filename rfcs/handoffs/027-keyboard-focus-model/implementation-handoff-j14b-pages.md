# Developer Handoff — RFC 027 J14b: every remaining control becomes a keyboard-reachable `snora::control` (five reviewed steps)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, 2018+ module style, tests in separate files, English only; relative paths with `cd`). Governing text: [RFC 027](../../done/027-keyboard-focus-model.md) (D′, "Rule from J13b", "Requirement 2026-11-19", "Decisions after the J10 build", D3: **nothing keyboard-related is released unless every step is done**) and [RFC 021](../../done/021-selection-state-indicators.md) ("Status after the J10 build": the chevrons). Review of J14a: `.git-exclude/reviewed/rfc-027-j14a-engine-review-2026-11-19.md`.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **A keyboard-only user can choose a file, convert, download one file and the ZIP, go back, change every setting and switch pages; the focus is always visible; nobody is ever trapped.**

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit 0 / 12); CI waited for each; no tags, force-push or history rewriting; delete large scratch files when done (`/tmp` is shared).

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does each migrated control look exactly as before at rest? | headless snapshots, machine comparison | the same temporary snapshot test before and after each step (CPU renderer, hermetic fonts), `cmp`: **byte-identical** unless a size changes on purpose (the control's own 24 px minimum target: describe every difference and look at those pictures only) |
| Is each control a Tab stop, in layout order, and does Enter/Space publish **exactly the message the click publishes, once**? | headless | per control, through the real view and the real `runtime_event` path (the J14a `focus_keys.rs`/`tab_stops.rs` machinery): Tab to it, Enter, Space, a click: the same single message |
| Completeness | headless | `NOT_MIGRATED` in `tab_stops.rs` loses the entries of each step in the same commit (a stale entry fails); **it is empty after step 4** and then removed with its machinery kept or reduced to an assertion that every clickable control is a stop |
| Focus is never lost or trapped | headless | after each action that removes the focused control (Back, a conversion finishing, a page change, a dialog's answer): Tab still moves somewhere sensible and the report says where focus is; no state where Tab does nothing on a page that has stops |
| Real window | real UI, two runs at the very end (below) | `scripts/gui-session.sh` |

## 1. How a control is migrated (all steps)

`snora::control(content)` replaces `button(content)`: **same builder shape** (`on_press`, `on_press_maybe`, `padding`, `width`, `height`, `clip`), the **style closure unchanged** (`Fn(&Theme, button::Status) -> button::Style`), an optional `.id(...)` only where a test needs one. Enter and Space press it; Ctrl/Alt/Cmd chords do not (they reach `shortcuts.rs`). Its minimum target is 24 px: a control smaller than that grows to it, which is a **wanted** change, listed in the review request. Do not wrap an iced `checkbox` in a control (they are snora's since J10). Do not touch messages, state or texts.

## 2. Steps (one commit each, tests first; remove each step's `NOT_MIGRATED` entries in the same commit)

1. **Navigation items** (desktop rail and mobile bottom bar, on every page): `ui/sidebar.rs`. The accent bar, bold label and filled background of RFC 021 Q1 must survive the change (the J10 `nav_cue.rs` tests are the guard and stay green).
2. **Home:** the drop-zone card (one Tab stop; Enter/Space = `PickFileRequested`, which `dialog_open` already guards); "View results →". **First add a matrix case that shows "View results →"** (a loaded result and the Home page) so it is in every property.
3. **Result:** each card's Save (`DownloadOneRequested(i)`, 7 or 16 stops in card order), the preview toggle, Back (header), Download all (footer). The preview toggle's `▶`/`▼` become lucide `ChevronRight`/`ChevronDown`; extend the source test for text-glyph state cues to `▶ ▼` (it currently lists `✓ ▣ ⏳ ⚠`).
4. **Customize:** "+ Add custom size", the "Advanced settings" header (chevrons as in step 3), "Reset to defaults". After this step `NOT_MIGRATED` is empty.
5. **Settings page and the end:** Settings itself has only radio groups (already stops); verify. Then: remove the Settings note "Moving between buttons and fields with the Tab key is not available yet" (the i18n key is public: **deprecate it** with `#[deprecated(since = "1.28.0", …)]` and the same item-level `#[allow(deprecated)]` pattern as RFC 021 D4, both locale files keep the string; a test that the note no longer appears); rewrite `docs/src/ui-a11y.md` "Keyboard support" **from the finished work and the evidence, nothing earlier** (what keys do what, the ring, the order, Esc, Ctrl+1/2/3, Ctrl+O on Home only, what is still not covered: screen readers), plus `README.md` and `functional-spec.md` where they state keyboard facts; RFC 018 syncs the rest later.

## 3. Focus after changes (report what you find; fix only a trap)

For each of: Back pressed; a conversion finishing (Converting → Result); a page change by Ctrl+2 or by Enter on a rail item; a file dialog's answer; the toast's close; Esc in a field: say **where the focus is afterwards** (headless: the focused target) and whether Tab then goes somewhere sensible. snora's rule is that a click presses without focusing, focus returns to the opener after a modal and moves to a neighbour after a removal. A state in which a page has stops and Tab does nothing, or focus sits on a control that is no longer there, is a trap: fix it (smallest change) and test it.

## 4. The real runs (end of step 5; two, then stop)

- **Native Wayland, `wtype`** (the keyboard is what is tested; no pointer): Home: Tab ×N → the ring on the first stop; Tab to the drop-zone card, Enter → **exactly one** new window (the chooser; J11 criterion), Esc closes it, the app is on the start screen; Ctrl+2 → Customize; Tab ×k → the ring on a checkbox; Space toggles it (second signal: the box changes); Shift+Tab back one; Ctrl+3 → Settings; Tab → a radio group, ↓ changes the theme (second signal: the page repaints). Assert each step by a second signal; abort at the first anomaly.
- **X11, `xdotool`** (the pointer is needed to focus the field): Customize, Advanced open, click into the custom-size field, type `12`, **Tab out → the field keeps `12`, the ring is on the next stop**; **Ctrl+1 in the focused field goes Home and types nothing**; Esc in the field leaves it.
- Both: owner's real `settings.json` mtime before and after; one instance by pid; focus check before every key; one run each, ask before a repeat.

## 5. Required evidence (review request: `.git-exclude/review-request/rfc-027-j14b-pages/README.md`; report only its path)

§0 table per step; for each step the `cmp` result and every picture that differs; the list of controls migrated with their message and the Enter/Space/click equality; the `NOT_MIGRATED` list shrinking; §3 findings; the two real runs; gates and CI by commit; test-list diff (named additions, none removed).

## 6. Acceptance criteria

Every interactive control of every page is a Tab stop in layout order; Enter/Space equal the click for each; nothing changes at rest except sizes raised to 24 px (listed); no text-glyph state cue remains; the Settings note is gone and the docs say what is true; no trap; real runs as above; gates and CI green at each commit; RFC 021's last item (the chevrons) is done.

## 7. Prohibited shortcuts

No second keyboard engine (snora's `traverse` and `control` are the owners); no change to `shortcuts.rs` bindings; no `Digits`/`escape`/`focus_zone`; no change to messages, state or texts other than deprecating the note's key; no new `#[allow]` beyond the one pattern named; no new package or `design` feature; no threshold change, `#[ignore]` or unnamed exception; no tags, force-push or history rewriting.
