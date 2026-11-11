# Developer Handoff — RFC 011: conversion job identity and "a chooser is already open" (job J11)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 011](../../accepted/011-conversion-job-identity.md) (Accepted, owner authorization 2026-11-09, release 1.28.0). Review of the last job: `.git-exclude/reviewed/rfc-017-part-3-review-2026-11-12.md`. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **The visible behaviour of the single-file case does not change.**

**Workflow:** work on `main`: `git pull --ff-only` first, **one commit per step**, gates chained with `&&` before every push, wait for each commit's CI before the next push, never a red push, no tags, no force-push.

**Position:** J11 in `ROADMAP.md`. The code now lives in `crates/app/src/app/` (`message.rs`, `state.rs`, `update.rs`, `update/{input,convert,…}.rs`) and `crates/app/src/task_queue.rs`; the RFC's file names (`app.rs`) are out of date, the design is not.

## 0. Baseline

Gates; `cargo test --workspace --all-features --locked -- --list 2>/dev/null | grep ': test$' | sort` into `evidence/tests-before.txt` (263). After the job, the list differs only by the tests you add (named) and by tests whose **message construction** you had to adapt (their **names stay**; list every adapted test).

## 1. Decisions (binding; the RFC left the policy open and the architect decides here)

1. **Input policy while a job runs: latest intent wins.** A new drop or a new pick supersedes the running job; the stale results are ignored. (The alternative, ignoring drops while converting, makes a mis-drop uncorrectable and is rejected.)
2. **Only the current job may change state or raise a toast.** A result, a success **or an error**, of a superseded job is dropped silently: no state change, no toast, no screen change.
3. **One file chooser at a time.** While a chooser opened by `PickFileRequested` is open, a further `PickFileRequested` (button, Ctrl+O, or any other route that sends the message) does nothing. The flag is cleared when the chooser's result arrives (`FilePicked(Some)` or `FilePicked(None)`). A **drop** while the chooser is open is still accepted (latest intent wins); the chooser stays open and its result, when it arrives, starts its own job as the latest intent.
4. **Out of scope:** the two Save dialogs (download one, download all) are not changed; if you see the same double-open there, **report it, do not fix it here**. RFC 031 (a dialog that cannot open) is a later job; do not change what happens when a dialog fails.

## 2. Implementation (steps, each its own commit)

1. **State:** `AppState::job_id: u64` (monotonic, starts at 0; documented as "id of the newest job; results with another id are stale") and `AppState::chooser_open: bool` (documented). Both are plain state, not persisted.
2. **Message payloads and tasks:** the id travels with each asynchronous result: `IngestCompleted(u64, Result<…>)`, `PreviewBuilt(u64, Result<…>)`, `ConvertCompleted(u64, Result<…>)`, `SvgAnalyzed(u64, PathBuf, SvgReport)` (keep the path; the id is added). `ingest_task(path, job)`, `build_preview_task(asset, algorithm, job)`, `convert_task(asset, plan, job)`, `analyze_svg_task(asset, job)` capture the id at creation and return it. **Every** caller passes `state.job_id`: besides `ingest_completed` there is at least one other caller of `build_preview_task` (the resize-algorithm change handler): find all of them with `rg` and say which. A preview rebuild after a settings change belongs to the **current** job (same id).
3. **New job:** `start_ingest` increments `state.job_id` **before** creating the ingest task and passes the new value. Nothing else increments it.
4. **Stale results:** every handler of the four messages first compares the carried id with `state.job_id`; a different id returns `Task::none()` immediately, **before any other work** (before `fail`, before any toast, before `state.screen` is touched). The existing source-path/algorithm checks in `preview_built` and `svg_analyzed` stay (they still guard the settings-change rebuilds); the id check is in front of them. Keep the handler bodies otherwise unchanged; the guard is the only new code in each.
5. **Chooser flag:** `pick_file_requested` returns `Task::none()` if `state.chooser_open`, otherwise sets it and starts the picker. `FilePicked(None)` and `FilePicked(Some(path))` clear it (the `Some` arm is the or-pattern with `FileDropped`; a **drop must not clear** a flag that belongs to an open chooser, so give the chooser result its own handling or clear the flag in a way that a drop does not trigger; show your solution and test it).
6. **`fail()`:** unchanged in behaviour; it is now only reached for the current job. Add no new branch.
7. **Docs:** one short passage in `docs/src/functional-spec.md` where the input flow is described (§ on dropping/choosing files): choosing or dropping a new file while one is being converted replaces it; only one file chooser can be open at a time. Nothing else in the documentation changes (RFC 018 syncs the rest).

## 3. Tests (write them first; show them failing before the change where they can fail)

All as `update()` tests with hand-built messages, in a new test file `crates/app/src/app/job_tests.rs` (declared like the other test files; shared helpers from `test_support.rs`):

1. `IngestCompleted(1, A)`, then `FileDropped(B)` (job becomes 2), then `ConvertCompleted(1, A-assets)` ⇒ `result_assets` unchanged, screen unchanged.
2. Two results in reverse order (job 2's conversion first, then job 1's) ⇒ only job 2's is applied; the final `result_assets` is job 2's.
3. A stale `PreviewBuilt(1, …)` after a new drop of the **same path** ⇒ ignored (`preview_cache` stays `None`).
4. A stale **error** (`IngestCompleted(1, Err)`, `ConvertCompleted(1, Err)`, `PreviewBuilt(1, Err)`) pushes **no toast** and changes no screen.
5. A stale `SvgAnalyzed` ⇒ no advisory toast.
6. The current job's success and error behave exactly as before (existing tests cover the success path; add the error toast for the current job if no existing test does).
7. `PickFileRequested` twice ⇒ the second returns no task (assert on the returned task being the "none" one in the way the existing tests detect it: look at how `shortcuts_tests.rs` counts task units); after `FilePicked(None)` it works again; after `FilePicked(Some(path))` it works again; a `FileDropped` while the flag is set neither clears it nor opens anything.
8. The Ctrl+O shortcut while a chooser is open does nothing (it goes through the same message).
9. `job_id` increments exactly once per `start_ingest`, and not on any other message.
10. **Settings leaves (logic level, replaces the real-window run J8c could not finish):** for each of these messages, build the persisted settings from the state before and after `update()` and assert that **exactly the expected leaf** differs and nothing else: toggling "Keep transparency"; toggling one PNG preset checkbox; adding a custom size; switching the language; picking a theme. Put them in `settings_tests.rs` (extend the existing helpers; look at what `persisted_settings.rs` already compares). If one of the five already has such a test, name it instead of duplicating.

## 4. Required evidence

- Tests: the failing-before list, the final list diff (only your named additions; adapted tests listed), counts.
- Proof of "guard first": for each of the four handlers, show the first statement is the id check (the diff).
- `rg` listing of all callers of the four task constructors, each passing `state.job_id` (or the new value in `start_ingest`).
- Gates chained on every pushed commit: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked` (the four existing doc warnings expected). CI per commit by name.
- **Evidence plan (amended 2026-11-12 to the owner's updated GUI test rules; read `.git-exclude/rules/project-instructions-gui-common.md`, "GUI Evidence and Automation Rules" and "Test Optimization Rules").**

  | Question | Level | Evidence |
  |---|---|---|
  | Are stale results of any of the four messages ignored, errors included? One job at a time? | logic and state | the §3 tests |
  | Does a second `PickFileRequested` (button click, Ctrl+O, any route) open nothing while a chooser is open? | logic and state | §3 tests 7 and 8 (the click and the shortcut send the same message; a double click needs no pointer run) |
  | Do the settings changes still change exactly their own leaf of `settings.json`? (the J8b/J8c "regression-by-use" run, now at the cheapest level) | logic and state | §3 test 10 |
  | Does the **real** chooser open exactly once on a double Ctrl+O, and can it be cancelled and reopened? | **real UI** (only a real portal chooser can show this) | the one run below |
  | Is the owner's real configuration untouched? | recorded state | modification time before and after |

- **The one real-UI run** (native Wayland, `wtype`, because the keyboard is what is tested; the session must be unlocked: `loginctl show-session <id> -p LockedHint`, and if locked stop and report; isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME` with a pinned `settings.json` copied and the copy asserted; one instance, window chosen by pid; the focus check before every keystroke; analyse after the run). Steps, asserting after each: (1) Ctrl+O sent twice in quick succession ⇒ **exactly one** new window (count windows by `niri msg -j windows`, no screenshot needed); (2) close it with Esc ⇒ the app shows the start screen (one capture at most, or a state check); (3) Ctrl+O again ⇒ exactly one new window; (4) choose a PNG ⇒ Result screen (the J7e criterion; one capture). **Write the run as a script and keep it** (path stated in the review request): it will become the project's shared session script (job JS), so isolation, launch, window selection, capture and teardown must be separate functions. Stop at the first anomaly; a repeat is a question to the architect. Not run, by design: the pointer double-click on "Choose file…" and the settings leaf-by-leaf run (both are covered above at the logic level); say so.
- Real configuration: the owner's `~/.config/logolig/settings.json` modification time unchanged (record before and after).
- A real two-file race (drop A, then B while A converts) cannot be injected under the GUI rules here; the tests of §3 are its evidence. Say so.
- Review request: `.git-exclude/review-request/rfc-011-job-identity/README.md`; report only its path.

## 5. Acceptance criteria

All §3 tests pass; the single-file behaviour is unchanged (all earlier tests pass with unchanged names); guard-first in all four handlers; every task constructor receives the current id; one chooser at a time in the application run; gates and CI green at every pushed commit; nothing outside `crates/app/src/app*`, `crates/app/src/task_queue*`, `docs/src/functional-spec.md` changed.

## 6. Prohibited shortcuts

No `Task::abortable`; no change to the Save dialogs; no behaviour change for the single-file flow; no handler body rewritten beyond the guard; no swallowing of the current job's errors; no new `#[allow]`; no `pub` that was private; no tags, force-push or history rewriting.
