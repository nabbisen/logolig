# RFC 017 — Source structure conformance (tests, file size, dead code)

**Status.** Proposed
**Priority.** P2 (enabler: should land before RFC 011, 012, 016 touch `app.rs`)
**Tracks.** Technical debt against `project-instructions-rust.md` and the development instructions.
**Touches.** `crates/app/src/app.rs` and its UI modules, `crates/core/src/services/flatten.rs`, `crates/app/src/task_queue.rs`, `crates/app/tests/app_state.rs`, `crates/core/tests/exporter.rs`, working-tree hygiene.

## Summary

Behaviour-preserving refactoring that brings the code in line with the project's own rules. No user-visible change.

## Evidence

Verified (listing and `wc -l`, 2026-10-21).

Rule: tests are separate files (`some_mod.rs` + `some_mod/tests.rs`); `#[test]` modules inside `some_mod.rs` are "Bad". Files over ~300 lines should be considered for splitting, over ~500 strongly; the same applies to `tests/` files.

| Item | Finding |
|---|---|
| Inline test modules | `core/src/services/flatten.rs`, `app/src/task_queue.rs`, `app/src/app.rs` (9 tests) |
| Files over 500 lines | `app/src/app.rs` 1476, `core/tests/exporter.rs` 697, `app/src/ui/advanced_drawer.rs` 660, `app/src/ui/preview_panel.rs` 565 |
| Blanket lint allowances | `#[allow(dead_code)]` on `enum Message`, on `ui::accessibility::{label, marker}`, and `#![allow(dead_code)]` in `result.rs` |
| Vestigial code | `AppState::advanced_open` (never set to true), `Message::NoOp`, `Message::AdvancedToggled`, `Message::LocaleChanged` (duplicate of `LocalePicked`), `busy`, `ExportPlan::overrides`/`SizeOverride` (no readers), `PreviewContext::label` and `ThemeMode::next` (check usage) |
| Dead branch (found in the RFC 009 review) | `ResultAssetKind::has_visual_thumbnail` lists `Svg`, but `build_thumbnail` returns `None` for SVG |
| Path comparison (found in the RFC 009 review) | `exporter::ensure_unique_paths` compares `to_string_lossy().to_lowercase()` without normalising separators; normalise via path components and add a backslash-path test when core is next hardened |
| Stale test scaffolding | `crates/app/tests/app_state.rs` contains one test and a "Step 1 / Step 2" comment; the integration-test crate cannot see app types |
| Working tree | untracked empty directories `crates/logolig-app/`, `crates/logolig-core/`, `crates/logolig-i18n/` (leftovers of an earlier layout) and empty `.agents/` |
| Doc comments | blank line splitting doc comments in `export_plan.rs` (two places) and `exporter.rs` header |

The prior review (2026-07-09) already named thin app-level tests; the current gates are green, so none of this is blocking.

## Design

1. **Move inline tests** to `flatten/tests.rs`, `task_queue/tests.rs`, `app/tests.rs` (declared with `#[cfg(test)] mod tests;`). No test changes.
2. **Split `app.rs`** by responsibility, keeping `AppState` and `Message` as the single public surface:
   `app.rs` (run, wiring), `app/state.rs`, `app/message.rs`, `app/update.rs` (dispatch) with handlers grouped in `app/update/{input,convert,download,settings,manifest,nav}.rs`, `app/toast.rs`, `app/persist.rs`. Pure move; no signature changes.
3. **Split** `advanced_drawer.rs` (sections: sizes, svg, extras, manifest) and `preview_panel.rs` (tab mock, phone mock, controls); split `core/tests/exporter.rs` by theme (default set, mono, transparency, Microsoft logos, in-memory vs disk).
4. **Dead code.** Remove the vestigial items above after confirming no readers; replace blanket `allow(dead_code)` with item-level allowances only where justified. Keep `ExportPlan::overrides` only if an RFC claims it (owner decision Q1); removing it changes the persisted JSON shape, which `serde(default)` tolerates on load.
5. **Tests.** Move `tests/app_state.rs` content into `core/tests/export_plan.rs` (it only tests core types) and delete the stale file.
6. **Hygiene.** Delete the empty directories locally; they are not tracked.
7. Each step is its own commit so reviews stay mechanical.

## Open question

- **Q1** `ExportPlan::overrides: Vec<SizeOverride>` is persisted API surface with no feature behind it. Remove, or keep as a reserved extension point?

## Acceptance criteria

- `cargo fmt --check`, `cargo clippy ... -D warnings`, `cargo test --workspace --all-features` unchanged (same 147 tests, same names).
- No `.rs` file over 500 lines (target: over 300 only with a stated reason).
- No inline `#[cfg(test)] mod tests { .. }` in `src/`.
- `git diff --stat` shows moves, not rewrites (review with `git diff -M`).

## Risk

Medium merge risk, low behaviour risk. Do not run concurrently with feature work in `app.rs`.
