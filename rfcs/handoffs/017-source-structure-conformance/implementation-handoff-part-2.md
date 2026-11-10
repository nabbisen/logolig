# Developer Handoff — RFC 017 part 2: split `app.rs` (job J8b)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 017](../../accepted/017-source-structure-conformance.md) (Design item 2 and the amended decision). Review of part 1: `.git-exclude/reviewed/rfc-017-part-1-review-2026-11-11.md`. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; no change a user could notice. **Behaviour-preserving: a pure move plus mechanical extraction.**

**Workflow (owner decision 2026-10-21): no topic branches, no concurrent jobs.** Work on `main`: `git pull --ff-only` first, **one commit per step**, gates chained with `&&` before every push, never a red push, no tags, no force-push. After a push, wait for that commit's CI to finish before the next push (part 1 had a run cancelled by two quick pushes; avoid it).

**Position:** J8b in `ROADMAP.md`, after J8a (reviewed). J8c (the other large files, and the split of `app/tests.rs`) follows after this job's review; **do not start it.**

## 0. Baseline

On the base commit: gates, and `cargo test --workspace --all-features --locked -- --list 2>/dev/null | grep ': test$' | sort` into `evidence/tests-before.txt` (263 expected). `crates/app/src/app.rs` is 1 428 lines today.

## 1. Step 0 (two small commits, before any move): retire dead code and one deprecation, decided in the part 1 review

1. **Retire the four ICO messages** `IncludeIcoToggled`, `IcoSizeInputChanged`, `IcoSizeAddRequested`, `IcoSizeRemoveRequested`: the variants, their `update` arms, the four item-level `#[allow(dead_code)]` and the comment about them, and `AppState::ico_size_input` (field, `Default`). `rg` first: nothing outside the arms and tests may use them; if a test sends one, remove that test and **name it** in the review request; if anything else reads them, stop and report. Keep every i18n key and every core item (`ExportPlan::ico_sizes`, `MessageKey`) as they are: public API of the published crates and RFC 016's business. Fix any document sentence that your removal makes false (grep `docs/`).
2. **Deprecate `ThemeMode::next`** in `crates/core/src/domain/theme_mode.rs` with `#[deprecated(since = "1.28.0", note = "no caller in the application; it will be removed in the next major version")]`; its test `theme_mode_cycles` gets an **item-level** `#[allow(deprecated)]` with a one-line reason (the test pins the behaviour of a published method until it is removed).

## 2. Target layout (Steps 1 to 5; names may differ only if you explain why, files stay small)

`app.rs` keeps the module doc, the `mod` declarations, the public re-exports and the **runtime wiring** only: `run`, `window_title`, `theme`, `subscription`, `view`. Everything else moves:

| New file | Receives |
|---|---|
| `app/state.rs` | `Screen`, `AppState`, `NavPage`, `impl AppState`, `impl Default for AppState`, `resolve_theme`, `is_mobile` |
| `app/message.rs` | `enum Message` |
| `app/update.rs` | `fn update` as a **dispatch only**: one short line per message that calls a handler (see §3) |
| `app/update/*.rs` | the handlers, grouped by responsibility, for example `input.rs` (file drop/pick, ingest, SVG analysis, preview build), `convert.rs` (conversion result, preview context/background, result preview toggle), `download.rs` (the Download* messages), `settings.rs` (export-plan toggles, sizes, algorithm, preset, reset, keep transparency, locale, theme), `manifest.rs` (web manifest and Microsoft logos and monochrome toggles), `nav.rs` (navigation, window resize, shortcuts) and `toast.rs` (toast tick, attention, dismiss) |
| `app/toast.rs` | `push_error_toast`, `push_success_toast`, `push_long_success_toast`, `push_warning_toast`, `next_id`, and the advisories `push_transparency_warning`, `push_non_square_warning`, `push_svg_parts_warning`, `push_jpeg_input_warning`, `is_non_square` |
| `app/persist.rs` | `snapshot_persisted`, `persist_settings` |

`parse_size` and `SizeParseError` go with the size handlers in `app/update/settings.rs`; `start_ingest`, `fail` and `handle_shortcut` go with their handler groups. Existing `app/tests.rs` and `app/toast_hold_tests.rs` stay where they are (J8c handles `tests.rs`).

**Size targets:** every new file at most 300 lines (hard limit 500). If a file exceeds 300, split it by responsibility rather than explain it.

**Visibility:** nothing becomes `pub` that was private. Items used across the new files become `pub(super)` or `pub(crate)` (the narrowest that compiles). `crate::app::{AppState, Message, NavPage, Screen, is_mobile, resolve_theme}` must keep working for every importer in `crates/app/src` **without editing any file under `ui/`, `shell.rs`, `task_queue*`, `result.rs`, `window*`, `shortcuts*`, `toast_time*`** (re-export them from `app.rs`). The tests use `use super::*`; keep them compiling without editing them (add `#[cfg(test)] use` lines in `app.rs` for private items they reach, or widen to `pub(super)`), and report every such line.

## 3. How to extract the handlers (mechanical, provable)

For each `update` arm whose body is more than three lines, create one function with the arm's variables as parameters:

```rust
Message::PngSizeAddRequested => settings::png_size_add_requested(state),
```

```rust
pub(super) fn png_size_add_requested(state: &mut AppState) -> Task<Message> {
    /* the arm body, byte for byte, one indentation level less */
}
```

Rules: the function body is **the arm's body unchanged** (same statements, same order, same comments); parameters carry the pattern bindings (`size`, `path`, `opt`, …) with the same names and types as bound in the arm; or-patterns and guards stay in the dispatch (the dispatch calls one function per pattern); arms of one to three lines (`Task::none()`, one assignment) stay inline in the dispatch. Handler names say what happens (`png_size_add_requested`), not "handle_x". No logic moves between arms, no early return is added or removed, no `?` or `let else` is introduced, nothing is "simplified" on the way. If an arm cannot be extracted without changing its body (for example it borrows two bindings in a way a function boundary breaks), **stop and report that arm**; do not rewrite it.

## 4. Steps (each its own commit, in this order)

1. Move `Message` to `app/message.rs`.
2. Move state types and `resolve_theme`/`is_mobile` to `app/state.rs`.
3. Move `persist.rs` and `toast.rs` helpers.
4. Extract the handlers and turn `update` into the dispatch (one commit per handler file is fine and preferred; if the intermediate state compiles and passes the gates, push each).
5. Final tidy: remove imports that became unused in each file (`clippy -D warnings` enforces it), update the module doc of `app.rs` to describe the layout in one short table.

## 5. Required evidence

- **Test list before and after identical** except the ICO tests you removed in Step 0 (named). Count = 263 − removed.
- **Move proof by script, per step**, as in part 1: for every moved item, the stripped non-blank lines of the new location equal those of the old, in order. For Step 4, per extracted arm: the stripped lines of the arm body equal the stripped lines of the function body (report the number of arms extracted and the number checked; all must match). Attach the script and its output.
- **The dispatch is total:** `update` has no `_ =>` catch-all arm (there is none today either; the compiler enforces exhaustiveness); the number of `Message` variants equals the number of dispatched patterns minus the ICO variants removed.
- Line counts per file in `crates/app/src` (before/after table); no new file over 300 lines (or each exception explained).
- `git diff --stat` and `git show --stat` of the steps; no file outside `crates/app/src/app*`, `crates/core/src/domain/theme_mode.rs`, `crates/core/tests/export_plan.rs` (only if the `theme_mode_cycles` test lives there), and docs fixed because of Step 0 may change. Say so explicitly.
- Gates chained on every pushed commit: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked` (the four existing doc warnings are expected; no new one).
- CI per commit by name (wait for each).
- **Application run (required, the dispatch was rewritten):** one release build of the head, isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME` (never start the binary without them), a **pinned `settings.json` copied with the copy asserted**, X11 or Wayland as the steps need (state which; **measure every pixel criterion on the backend where you use it, before the run**): Ctrl+O + a PNG → Result screen; Esc dismisses the toast, Esc returns to the start screen (input/convert/toast/nav paths); Customize: toggle "Keep transparency" and one PNG preset checkbox and add a custom size, then Settings: switch the language; `settings.json` after each step shows exactly the expected change and nothing else (settings path); one toast waits under the pointer (X11) or, if you cannot place the pointer, say so. The Download paths go through the portal save dialog: cover them by the existing unit tests and say so; run them in the application only if the dialog can be driven within the GUI rules. One run; stop at the first anomaly.
- Review request: `.git-exclude/review-request/rfc-017-part-2/README.md`; report only its path.

## 6. Acceptance criteria

`app.rs` is wiring only and under 300 lines; no file in `crates/app/src/app/` over 300 lines (exceptions explained; `tests.rs` is J8c's); every extracted handler body equals its old arm; the test list differs only by named removed tests; gates and CI green at every pushed commit; the application run shows unchanged behaviour; no change to a file outside the allowed list.

## 7. Prohibited shortcuts

No behaviour change hidden in a move; no rewriting of a handler to make the extraction easier; no `pub` that was private; no new `#[allow]` (except the one named in Step 0.2); no edit of `ui/`, `shell.rs` or other modules to make imports work (re-export instead); no deletion or weakening of tests except the named ICO ones; no split of `advanced_drawer.rs`, `preview_panel.rs` or `tests.rs` (J8c); no tags, force-push or history rewriting.
