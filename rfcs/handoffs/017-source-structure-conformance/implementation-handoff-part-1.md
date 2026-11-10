# Developer Handoff — RFC 017 part 1: tests to their own files, vestigial code, stale scaffolding (job J8a)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 017](../../accepted/017-source-structure-conformance.md), **including the decision amended on 2026-11-10** (read it first: the core crate is published, so nothing public in it is removed in 1.x). The RFC says what and why; this handoff says how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; no change a user could notice. **This job is behaviour-preserving.**

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** Work on `main`: `git pull --ff-only` first, **one commit per step below** (short imperative subject), gates chained with `&&` before every push, never a red push, no tags, no force-push.

**Position in the schedule:** J8a in `ROADMAP.md`, after J7e. J8b (the `app.rs` split) follows only after the architect reviews this job; **do not start it and do not split `app.rs` into files here.**

## 0. Baseline first

On the base commit: run the gates, and record `cargo test --workspace --all-features --locked -- --list 2>/dev/null | grep ': test$' | sort` into `evidence/tests-before.txt` (261 tests expected). After each step you compare the list again; the only allowed differences are the ones this handoff names.

## 1. Steps (each its own commit)

**Step 1 — move the three inline test modules (pure move, no change to any test).**
- `crates/core/src/services/flatten.rs` → tests into `crates/core/src/services/flatten/tests.rs`;
- `crates/app/src/task_queue.rs` → `crates/app/src/task_queue/tests.rs` (the directory already holds `zip_tests.rs` and `blocking_tests.rs`; keep them);
- `crates/app/src/app.rs` → `crates/app/src/app/tests.rs` (`app/toast_hold_tests.rs` already exists; keep it).
In each parent keep exactly `#[cfg(test)] mod tests;`. The module is still called `tests`, so every test keeps its full path and name. Test bodies stay byte-for-byte the same except the unavoidable removal of one indentation level; show that with `git diff -M --stat` and a whitespace-insensitive diff (`git diff -M -w`) that shows moved lines only. Test list before and after: **identical**.

**Step 2 — remove vestigial code in the application crate (a binary: no public API).** For every item, first show with `rg` that nothing outside tests reads it; if something does, stop and report that item instead of removing it.
1. `AppState::advanced_open`: the field, its `Default` entry, the two assignments in `update` (`= false`), and the stale comment "vestigial but harmless"; also the mention in the module doc of `crates/core/src/domain/persisted_settings.rs` (comment only).
2. `Message::NoOp`, `Message::AdvancedToggled` and their `update` arms; the test that sends `AdvancedToggled` is removed with it (**named in the review request as a removed test**) unless it tests something else too, in which case adjust only the part that no longer exists.
3. `Message::LocaleChanged`: **first compare its handler with `LocalePicked` line by line**; remove `LocaleChanged` only if it is an exact duplicate and nothing sends it. If they differ, stop and report the difference.
4. `AppState::busy`: written in `start_ingest`/`fail` and the ingest/convert completion arms, read only by one test? If no production code reads it, remove the field, its writes and the assertion; **report the test change**. If a view or handler reads it, keep it and say where.
5. `ResultAssetKind::has_visual_thumbnail` lists `Svg`, but `build_thumbnail` returns `None` for SVG. Remove `Svg` from the `matches!` (the result is identical); keep the comment "SVG: no raster thumbnail" where it is true.
6. Blanket allowances: remove `#[allow(dead_code)]` from `enum Message` and `#![allow(dead_code)]` from `crates/app/src/result.rs`; build; for each warning delete the item if it is dead, or keep it with an item-level `#[allow(dead_code)]` and a one-line reason if it is needed (for example, by a test or a documented near-term use). List every item you kept and why. `ui/accessibility.rs`: remove the unused `label` constants (delete the module if every constant is unused); for `marker` **do not change any constant or its meaning** (RFC 021 changes the cues in a later job); if some constants are unused, keep an item-level allowance with a reason. `crates/core/tests/fixtures.rs` (`#![allow(dead_code)]` for a helper module shared by test crates) is justified: leave it and say so.

**Step 3 — core crate: deprecate, never remove (public API of the published `logolig` 1.27.0).**
1. `ExportPlan::overrides` and `SizeOverride`: add `#[deprecated(since = "1.28.0", note = "...")]` (the note says it has no effect and is removed in the next major version). Make `overrides` skip serialisation when empty (`#[serde(skip_serializing_if = "Vec::is_empty")]`) so `settings.json` no longer carries `"overrides": []`; deserialisation stays tolerant (the struct has `#[serde(default)]`).
2. `ThemeMode::next` and `PreviewContext::label`: check callers in `crates/app` and `crates/core`; if there are none, deprecate each the same way; if there are callers, leave them alone.
3. Inside the workspace nothing may use a deprecated item, so no warning appears; the unavoidable uses (the `Default` impl that builds `overrides`, the `pub use` re-exports in `domain.rs` and `lib.rs`) get an **item-level** `#[allow(deprecated)]` with a one-line reason, and nothing broader.
4. Tests (in `crates/core/tests/persisted_settings.rs` or `export_plan.rs`, per the rules): (a) a `settings.json` text **with** an `"overrides": [{"size": 32, "source_path": "x.png"}]` entry in `export_plan` loads, and every other value is as written; (b) `PersistedSettings::default()` serialised to JSON contains **no** `overrides` key; (c) the same text loaded and saved again has no `overrides` key. Write them first and show them failing/passing as appropriate (only (b) and (c) fail before the change).
5. `cargo doc --workspace --no-deps` must not warn about the deprecated items.

**Step 4 — stale test scaffolding.** `crates/app/tests/app_state.rs` tests only core types and carries "Step 1 / Step 2" comments. Move its tests into `crates/core/tests/export_plan.rs` (or `persisted_settings.rs` if that is where they belong; check for duplicates and for the `overrides` assertion, which becomes `#[allow(deprecated)]`-free by deleting that single assertion, **reported**), delete the file and the empty `crates/app/tests/` directory. Test names stay; their crate changes.

**Step 5 — doc comments.** Remove the blank line that splits a doc comment in two places in `crates/core/src/domain/export_plan.rs` and at the header of `crates/core/src/services/exporter.rs` (and any other you meet with `rg -n -B1 '^\s*$' ` on a `///` block you touch; do not hunt beyond these).

**Step 6 — hygiene (not a commit).** The empty untracked directories `crates/logolig-app/`, `crates/logolig-core/`, `crates/logolig-i18n/`: check with `find <dir> -empty` that they are empty, then delete them locally. **Leave `.agents/` alone** (it may belong to the owner's tooling).

## 2. Not in this job

No file split beyond the three test files (J8b/J8c). No behaviour change; **in particular not** the `exporter::ensure_unique_paths` path comparison (a behaviour fix, tracked in RFC 017's table for a later hardening). No change to `AppxManifest.xml`, `rfcs/`, `ROADMAP.md`, CI, dependencies, i18n texts.

## 3. Required evidence

- `evidence/tests-before.txt`, `evidence/tests-after.txt` and their `diff`: the only differences are (a) tests removed with removed code (named, with the reason) and (b) the new tests of Step 3.4 (named). Final count = 261 − removed + added; report the real number and show the arithmetic.
- For Step 1: `git diff -M --stat` and `git diff -M -w` evidence that tests moved unchanged; line counts of the three parent files before and after (`app.rs` is expected to fall by about 600 lines).
- `rg` evidence per removed item (Step 2: no reader outside tests) and the list of items kept under item-level allowances.
- Gates chained on the final tree: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked` (no warnings from your changes).
- CI on the last pushed commit: the five jobs by name.
- **A short run of the application is required**, because `update` and persistence were touched: one release build, isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME` (never start the binary without them), native Wayland; with a **pinned** `settings.json` (copy it and **assert that the copy succeeded**; stop if not) that contains an old-style `"overrides": [...]` entry: the application starts, Ctrl+O + a PNG reach the Result screen (the criteria of the J7e smoke check), a language switch in Settings makes the application rewrite `settings.json`, and the rewritten file has **no** `overrides` key and keeps the other values. One run; stop at the first anomaly.
- Review request: `.git-exclude/review-request/rfc-017-part-1/README.md`; report only its path.

## 4. Acceptance criteria

Same behaviour; the three inline test modules are gone from `src/`; the listed vestigial items are gone from the application crate or kept with a stated reason; public items of `logolig` are deprecated, not removed; old settings files load and new ones do not carry `overrides`; test list differs only as stated; all gates and CI green; nothing outside the scope above changed.

## 5. Prohibited shortcuts

No removal of a public item of `logolig` or `logolig-i18n`; no weakening or deleting of a test other than the named ones; no `#[allow]` broader than the item; no behaviour change hidden in a "move"; no split of `app.rs`; no `-A warnings`; no tags, force-push or history rewriting.
