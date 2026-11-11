# Developer Handoff — RFC 017 part 3: split the large UI files and the large test files (job J8c)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 017](../../accepted/017-source-structure-conformance.md) (Design item 3). Review of part 2: `.git-exclude/reviewed/rfc-017-part-2-review-2026-11-11.md`. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; no change a user could notice. **A pure move: no view code is rewritten, no function is split, no style or layout changes.** (RFC 027 and RFC 021 will rewrite these views later; this job only makes those later reviews small.)

**Workflow:** work on `main`: `git pull --ff-only` first, **one commit per step**, gates chained with `&&` before every push, wait for each commit's CI before the next push, never a red push, no tags, no force-push. This is the last part of RFC 017; do not touch anything else.

## 0. Baseline

On the base commit: gates and `cargo test --workspace --all-features --locked -- --list 2>/dev/null | grep ': test$' | sort` into `evidence/tests-before.txt` (263). Line counts of the four files below. **Build and keep the base release binary** (`cp target/release/logolig <scratch>/base/logolig`) for the visual comparison in §3.

## 1. Steps (each its own commit)

1. **`crates/app/src/ui/preview_panel.rs` (570 lines)** keeps `view` and the `mod` declarations; the rest moves into `ui/preview_panel/`: `pickers.rs` (`view_as_picker`, `surface_picker`, `picker_button`, `picker_button_optional`, `loading_placeholder`, `context_message_key`, `background_message_key`), `checker.rs` (the checker image: constants, `checker_handle`, `checker_view`), `mockups.rs` (`render_context`, `browser_tab_view`, `smartphone_view` and the four colour helpers `chrome_bg_for`, `tab_face_for`, `text_color_for`, `wallpaper_for`). Adjust only if a function's real dependencies say otherwise, and say why.
2. **`crates/app/src/ui/advanced_drawer.rs` (660 lines)**: `view` (about 290 lines) **stays whole in `advanced_drawer.rs`** (splitting a view function is a rewrite, and RFC 027 will rewrite it); the rest moves into `ui/advanced_drawer/`: `svg.rs` (`svg_subsection`, `algorithm_row`, `vtracer_preset_row`), `manifest.rs` (`subsection`, `web_manifest_body`, `labeled_input`), `choices.rs` (`LocalizedAlgorithm`, `LocalizedPreset`, `algorithm_message_key`, `preset_message_key`), `styles.rs` (`reset_button_style`, `secondary_drawer_button_style`, `extras_header_style`). If `advanced_drawer.rs` ends above 300 lines, that single reason is accepted (the unsplit `view`); state it.
3. **`crates/core/tests/exporter.rs` (697 lines)** into top-level integration test files, **one file per theme, each its own test crate so test names stay identical**: `exporter_disk.rs` (the default-set, SVG/WebP/ICO/HTML/apple-touch/failure tests), `exporter_monochrome.rs`, `exporter_in_memory.rs`, `exporter_transparency.rs`, `exporter_microsoft_logos.rs`. `fresh_tmp_dir` is used by several: move it **unchanged** into `crates/core/tests/fixtures.rs` as `pub fn` (that file is the shared helper module and already has `#![allow(dead_code)]`, justified) and `mod fixtures;` in each file that uses it. Delete `exporter.rs` when empty. Each new file at most 300 lines.
4. **`crates/app/src/app/tests.rs` (600 lines)**: split by theme into sibling files declared in `app.rs` the way `toast_hold_tests` is (`#[cfg(test)] mod <theme>_tests;`), for example input/ingest and advisories, settings, result and download, toast lifetime, shortcuts; the existing `toast_hold_tests.rs` stays. Shared helpers (state builders, `press`, …) go to **one** `app/test_support.rs` (`#[cfg(test)]`), not copied. Test **bodies unchanged**; their module path changes (`app::tests::x` → `app::<theme>_tests::x`), which is unavoidable and is the only name change allowed. Each new file at most 300 lines. Delete `tests.rs` when empty.

Keep every `use` that a moved block needs; remove none that another block still needs; `clippy -D warnings` enforces unused imports. Nothing private becomes `pub`; use `pub(super)` (or the narrowest that compiles). **Callers must not change:** `crate::ui::advanced_drawer::view` and `crate::ui::preview_panel::view` keep their paths; `customize_page.rs`, `result_view.rs`, `ui.rs` are not edited.

## 2. Proofs (scripts and outputs attached; as in part 1 and part 2)

- **Moved code (steps 1 and 2, and the test bodies of 3 and 4):** for every moved item, its stripped non-blank lines appear contiguously and in order in the new file; the only allowed differences are a leading visibility keyword (every such line printed) and added `use` lines. Report the item count and "all found".
- **Test list:** for steps 1, 2 and 3 **identical** to the baseline. For step 4 compare **names without module path** (`sed 's/^.*:://'`-style normalisation, say exactly how): the multisets are identical (263), and give the old-path → new-path table of all moved tests. The final list has 263 tests.
- **No view or style edit:** `git diff` of steps 1 and 2 contains no line of function **body** that is not a moved line (the script above proves it); no `pub` added; `rg` for new `#[allow]`: none.
- Line counts per new file; none over 300 except the stated one.

## 3. Evidence of unchanged behaviour (required: this job moves view code)

Release builds named `logolig`: the **base** binary (§0) and the **head**; isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME` (never start the binary without them); a **pinned** `settings.json` copied with the copy asserted (stop if it failed); one instance; windows not resized by you; **measure every pixel criterion on the backend where you use it, before the run**; the owner's session must be unlocked (check `loginctl show-session <id> -p LockedHint`; if it is locked, **stop and report, do not try to work around it, do not repeat**).

1. **Visual comparison by machine (native Wayland, `wtype`; as in J7d §6.1):** with the same input (`packaging/windows/Assets/Square150x150Logo.png`, now the new logo art if J7f has landed; base and head use the same file in one run) capture with both binaries: Result with the preview panel **collapsed**; Result with it **expanded** in each of its three contexts (browser tab, phone, checker); Customize with the advanced extras **expanded** (SVG section, algorithm, preset, web manifest enabled with its fields); Settings; Home. Light and Dark where practical. Compare each pair by bytes (or pixel-difference count with the bounding box). Expected: **all identical**. Any difference: describe it in words and stop the job's push; the architect decides.
2. **The J8b list (it was not run in part 2):** Ctrl+O + a PNG → Result; Esc dismisses the toast (still Result); Esc returns to the start screen; Customize: toggle "Keep transparency", one PNG preset checkbox, add a custom size; Settings: switch the language; after each step `settings.json` changed **exactly the expected leaf and nothing else** (start-up, conversion, Esc and navigation change nothing). On X11 only (pointer needed): a toast waits under the pointer past its lifetime. Downloads need the portal save dialog: cover them by the unit tests and say so. One run of each set; stop at the first anomaly; if a set has to be repeated, stop and ask.
3. Real configuration: the owner's `~/.config/logolig/settings.json` modification time must be unchanged by your work (record it before and after).

## 4. Other evidence

- Gates chained on every pushed commit: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked` (the four existing doc warnings expected).
- CI per commit by name. Review request: `.git-exclude/review-request/rfc-017-part-3/README.md`; report only its path.

## 5. Acceptance criteria

No `.rs` file in `crates/` over 500 lines; every new file at most 300 (the one stated exception allowed); moved code equals old code; names as stated; gates and CI green at every pushed commit; the visual comparison identical (or its differences explained and approved); the J8b settings run clean; nothing outside the files above changed.

## 6. Prohibited shortcuts

No rewriting or splitting of a view function; no style, layout or text change; no `pub` that was private; no new `#[allow]`; no edit of `customize_page.rs`, `result_view.rs`, `ui.rs` or other callers; no test body edit; no deleted or weakened test; no `rfcs/` or `ROADMAP.md` edit; no tags, force-push or history rewriting.
