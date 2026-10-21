# Developer Handoff — RFC 019, Phase 1: keyboard verification and honest documentation

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/`.

Governing RFC: [RFC 019](../../accepted/019-keyboard-accessibility.md) (Accepted, phased). This handoff covers **Phase 1: steps 1 and 2** (verify at run time, then make the documentation exact). Its section "Decisions and phasing" is binding. Steps 3–4 (shortcuts, focus states) are **not** part of this task. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled. A documentation promise that the software does not keep misleads exactly the users who depend on it.

Dependencies: none. May run in parallel with the other handoffs. Its documentation edits are limited to keyboard and focus statements, so conflicts with other handoffs' docs edits should be trivial; rebase if needed.

## 1. Purpose

1. Establish, by execution, what a keyboard-only user can and cannot do in the running application today.
2. Make every keyboard/focus statement in the documentation match that evidence.

## 2. Background (what is known, and how well)

- **Verified in dependency source** (`iced_widget` 0.14.2): `Button::operate` registers no focusable; `Focusable` is implemented only by `text_input` and `text_editor`. Checkbox, toggler and radio were not found among the implementers either. In iced, Tab traversal is not automatic: an application has to run a focus operation (`focus_next` / `focus_previous`) from its own keyboard handling, and this application has none (verified by `grep`: the only keyboard-related subscription is the window/runtime events one in `app.rs::subscription`, which handles file hover/drop and resize).
- **Not verified by execution:** every consequence for the user. That is this task.
- snora 0.25.2 offers only `snora::keyboard::dismiss_on_escape`; `functional-spec.md` records "ESC key: Not implemented".
- Documents that promise keyboard behaviour (from the architect's grep; re-run it yourself, do not trust this list): `docs/src/ui-a11y.md` (section "Keyboard parity", the sentence about focus order and the focus ring, and "What we do not do: we do not suppress iced's focus ring"), `docs/src/functional-spec.md` §14 ("Keyboard alternative path…"), and the transition table (ESC row). README has no explicit keyboard promise but check.
- Environment: the maintainer's session is niri (Wayland). The GUI testing rules are in `.git-exclude/rules/project-instructions-gui-common.md` (`niri msg`, `xdotool` available; resize to 1280×720; focus the window before typing; screenshots with `niri msg action screenshot-window --id ... --path <relative path>`, not ImageMagick `import`). `xdotool` drives X11 windows only. To test with it, run the application through the X11 backend (for example `WINIT_UNIX_BACKEND=x11`, or unset `WAYLAND_DISPLAY`) under XWayland, if available. Whether a way to inject key events into a native Wayland window exists on this machine (for example `wtype` or `ydotool`) is **unknown**; check and report.

## 3. Change scope (you may touch)

Documentation only, and only keyboard/focus statements:
- `docs/src/ui-a11y.md`
- `docs/src/functional-spec.md` (§14 and the transition-table ESC row; anything else that your grep finds and the evidence contradicts)
- `README.md` only if the grep finds a keyboard/focus statement that the evidence contradicts (list it in the review request either way)
- A new evidence file in the review-request directory (§8); optionally a copy of the final test matrix as `docs/src/keyboard-support.md` **only if** the architect asks for it later (do not create it now).

## 4. Non-change scope (you must not touch)

- Any code under `crates/`, `Cargo.toml`, `Cargo.lock`, CI files, packaging, RFC files.
- Do not implement shortcuts, focus handling, focus styles, or ESC handling. Do not "fix" anything you find; report it.
- Do not rewrite documentation outside keyboard/focus statements (other drifts are RFC 018's).
- Do not claim support for a platform you did not test.

## 5. Required work

### Step 1 — verify by execution

Build the application from the base commit (`cargo build --release -p logolig-app`; baseline notes in §7) and drive it as a keyboard-only user would. For **each screen** (Home/Empty, Home/Result with a converted image, Customize with the Advanced section both collapsed and expanded, Settings) and **each key** (Tab, Shift+Tab, Enter, Space, Escape, arrow keys, Ctrl+O, Ctrl+S, Ctrl+W, Alt+F4, and typing letters), record what happens: nothing / focus moves (and whether any visible indicator shows where) / action activates / text is entered / something else. Also record:

1. Can a user reach and activate the primary path with the keyboard only: choose a file (the whole drop-zone card and its "Choose file…" chip), wait for conversion, activate "Download all (ZIP)", use "Back", and switch between Home / Customize / Settings? State yes/no per action.
2. In the Customize page: do text inputs receive keyboard focus only through mouse clicks, or also through Tab? Does Enter in the custom-size input add the size (`PngSizeAddRequested`)? Does Tab move between the manifest text fields when the manifest option is on?
3. Is there **any** visible focus or hover/pressed indicator on the drop-zone card, nav items, download buttons? (The drop-zone button style ignores the widget status; verify visually and say so.)
4. What the native file dialog does with the keyboard is the operating system's business; note it but do not count it as application support.
5. Window close shortcuts and other OS-level behaviour: record, but separate them from application behaviour.

Platforms: Linux Wayland (native) is required where input can be injected; otherwise Linux through the X11 backend; Windows and macOS only if a machine is available. For each platform not tested, write "not tested" with the reason; do not infer. If Wayland input injection is impossible and only X11 was exercised, say that the Wayland result is **inferred from identical application code and unverified**.

Produce a table (platform × screen × key → observed behaviour) plus screenshots where a visible indicator exists or is absent (for example a before/after pair around pressing Tab on the Empty screen). Keep raw notes of how each key was sent (command lines).

### Step 2 — make the documentation exact

1. Run `grep -rniE "keyboard|focus|\bTab\b|reachable|ESC|escape|shortcut|accessib" README.md docs/src/*.md` and list every hit that makes a keyboard/focus claim (include the file and line, and the claim verbatim).
   Also list, **without editing them** (code is out of scope), source comments that make the same claim; the architect found two: `crates/app/src/ui/drop_zone.rs:6` ("Keyboard-accessible alternative path") and `crates/app/src/ui/preview_panel.rs:134` ("buttons as keyboard-accessible"). The architect's grep for real key handling (`keyboard`, `focus_next`, `focus_previous`, `key_press`, `on_key`) found no code, only those comments; re-check.
2. For each claim, mark it **confirmed**, **contradicted** or **not testable**, citing your evidence.
3. Edit only the contradicted or unsupported claims. Replace them with statements that are exactly what the evidence shows. Required structure in `ui-a11y.md`: one section "Keyboard support (current)" with two short lists: **Supported today** and **Not supported yet (tracked in RFC 019)**; keep the tone factual and short, no apology, no promises of dates. State the tested platforms and the version. The existing principle "We do not rely on drag-and-drop for any operation" must be re-checked against the evidence (clicking is still needed) and worded truthfully (the file chooser is an alternative to drag-and-drop, but it is **mouse-activated**).
4. Do not remove the design intent: keep a short "Goal" sentence that the project aims for full keyboard parity, pointing at RFC 019.
5. Keep ABDD wording consistent with the rest of the documentation; English only.

## 6. Acceptance criteria

- The test matrix exists, with every cell filled or marked "not tested/inferred" with a reason; the primary-path questions 1–3 of step 1 are answered yes/no with evidence.
- Every keyboard/focus claim found by the grep is classified; every contradicted one is corrected; the review request lists each documentation edit as before/after text.
- No claim in the documentation is stronger than the evidence.
- No file outside §3 changed (the reviewer will run `git diff --stat` against the base).

## 7. Required validation

Documentation only, so Rust gates are not required for the diff. Record the base commit and the build command you used, and state: "Rust gates not run; no Rust files changed", or run them and report if you prefer. Check that all Markdown relative links you touched still resolve (a simple script or manual check; say which).

## 8. Prohibited shortcuts

- No inferring behaviour from source or from this handoff's background; the point is execution. Mark inferences clearly.
- No documenting an aspiration as a fact, and no deleting a promise without recording that it was contradicted.
- No code changes, even trivial ones.
- No platform claims without a test.

## 9. Delivery

Work on branch `rfc-019-keyboard-docs`, small commits (match the repository's short imperative subject style). Push the **branch**. Do not merge, tag or release; do not change RFC status.

## 10. Required review request

In `.git-exclude/review-request/rfc-019-phase-1-keyboard/` with a `README.md` entry point, containing everything required by section 7 of the mid-capability operating instructions (adapted), plus: the full test matrix; how each key was sent; screenshots; the claims table (claim, location, verdict, evidence); the documentation diff summary with before/after text; unverified platforms; findings beyond the RFC (for example anything that behaves unexpectedly); and open questions for the architect. Report to the owner only the path of the `README.md`.
