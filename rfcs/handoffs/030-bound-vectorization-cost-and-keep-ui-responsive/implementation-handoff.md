# Developer Handoff — RFC 030: Bound vectorization cost and keep the UI responsive (job J4b)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`). **GUI evidence rules: `.git-exclude/rules/project-instructions-gui-common.md`, revised 2026-10-25 and authoritative**: fixed window size only at launch (never resize or float a running window; state the size the window opened with), screenshots with an **absolute** `--path`, `wtype` only after confirming niri's focused window is the instance under test (by `pid` for a Wayland window), isolate `XDG_CONFIG_HOME` **and** `XDG_STATE_HOME`, binary named `logolig`, one instance.

Governing RFC: [RFC 030](../../accepted/030-bound-vectorization-cost-and-keep-ui-responsive.md) (Accepted; owner authorized it into release 1.27.0 on 2026-10-25), including its "Binding details fixed with the handoff". On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision): no topic branches, no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first (the checkout is shared with the architect), small commits, **chain every gate with `&&` and push only when all pass**, no force-push, no tags. Job J4b in `ROADMAP.md`; prerequisite: J4 reviewed (it is).

## 1. Purpose

With the default settings a photographic or noisy raster makes the tracer run for seconds to minutes on full-size pixels, produces a `favicon.svg` of tens of megabytes, and blocks a tokio worker so that other events (keyboard shortcuts, drag and drop) are not served meanwhile. Fix both, in this order: **measure and confirm the cause, then bound the cost and move the work off the executor, then tell the user when the SVG is large.**

## 2. Background (verified facts)

- Architect's measurement (release build, `vectorize::vectorize`, `VtracerPreset::Default`, synthetic images, one machine; program kept in `.git-exclude/tmp/vec-time/`, untracked): flat two-colour logo 4 ms / 13 ms / 49 ms / 93 ms at 256 / 512 / 1024 / 1400 px; gradient with noise 76 ms / 551 ms / 3.7 s / 10.2 s; pure noise 93 ms / 752 ms / 4.1 s / 13.1 s with a **24 MB SVG** at 1400 px. Superlinear growth.
- RFC 019 phase 2 report (debug build, 1400 × 1400 noise PNG, about 50 s): while converting, Cmd+O, Cmd+S and Cmd+2 were not acted on until the end (a sidebar click worked); 46 threads, one tokio worker at 100 %. **The explanation (a blocked worker starves the subscription stream) is a hypothesis, not verified.**
- Code: `crates/app/src/task_queue.rs::convert_task` and `build_preview_task` are `Task::perform(async move { <synchronous CPU work> }, ...)`. `crates/core/src/services/exporter.rs::run_in_memory` calls `vectorize::vectorize(src, plan.vtracer_preset)` (line about 114) on the full decoded raster. `ExportPlan::default()` has `include_svg = true` and `vectorize_on_raster = true`.
- `logolig-app` does **not** depend on `tokio` directly today; the workspace declares `tokio = "1"` and iced is built with its `tokio` feature, so tokio is already in the tree.
- Toast lifetimes come from `toast_time` (RFC 029); advisory toasts are pushed with `push_warning_toast`.

## 3. Step 1 — measure (evidence only; stop if the hypothesis fails)

1. Build **release** (`cargo build --release -p logolig-app`; copy nothing, run `target/release/logolig` so the config folder name stays `logolig`).
2. Create (outside the repository, in a scratch directory) a heavy fixture you generate yourself: a 1400 × 1400 RGB noise PNG (the architect's 13 s case in release). Do not read or reference any file you did not create.
3. On **native Wayland** with `wtype` (rules above): start the app, choose the fixture through the file dialog, and about **2 s** after the dialog closed press **Ctrl+2** (the Customize page, a visible effect); take **one** capture about 1 s later. Record whether the page changed within that second. Expected before the fix: **no** (starved until the conversion ends, about 13 s). Take one more capture after the conversion ends to show the app is alive. Also note whether the "Importing…" screen was shown the whole time.
4. If the shortcut **is** served during the conversion in the release build, the hypothesis is wrong in release builds: **stop and report** (the cap in step 2 still stands, but the RFC is amended by the architect). Do not continue with `spawn_blocking` on a guess.
5. Core timing in release for the **current** code: reproduce the architect's table for 512 / 1024 / 1400 px noise and the flat logo with your own small program (untracked); record times and SVG sizes. This is the "before" for step 2.

## 4. Step 2 — change scope (after step 1 confirms the cause)

Core (`crates/core`):
- `src/services/vectorize.rs`: `pub const VECTORIZE_MAX_SIDE: u32 = 512;` a pure `pub fn trace_dimensions(width: u32, height: u32, max_side: u32) -> (u32, u32)` (uniform scale, aspect ratio kept, never below 1, unchanged when the longest side is already within the cap); `pub fn vectorize_with_cap(rgba, preset, max_side)`; `vectorize` calls it with the constant. When scaling is needed use `crate::services::resize::resize(src, w, h, algorithm)` with the **plan's** algorithm: give `vectorize_with_cap` an `algorithm: ResizeAlgorithm` parameter and pass `plan.algorithm` from `exporter::run_in_memory` (adapt the one call site; keep `vectorize::vectorize(rgba, preset)` available with `Lanczos3` for existing callers and tests).
- `src/services/exporter.rs`: only the vectorize call site (pass the algorithm).
- Tests in `crates/core/tests/vectorize_cap.rs` (new; **register in `crates/core/Cargo.toml` `[[test]]`, `autotests = false`**).

App (`crates/app`):
- `src/task_queue.rs`: `convert_task` and `build_preview_task` run their CPU work through `tokio::task::spawn_blocking` (a join error maps to `AppError::export("conversion task failed: ...")`, never a silent loss, never a panic). Add `tokio = { workspace = true, features = ["rt"] }` to `crates/app/Cargo.toml` (pre-approved, see the RFC); verify that `Cargo.lock` gains **no new package** and report the diff. Use the smallest feature set that compiles.
- `src/app.rs`: on `ConvertCompleted(Ok(assets))`, if the item `favicon.svg` exists and `bytes.len() > SVG_ADVISORY_BYTES` (a constant, 2 MB = 2 * 1024 * 1024) push **one** warning toast (`ToastSvgLargeTitle` / `ToastSvgLargeBody`); never for other sizes; never repeated by `ShowLastResultRequested`/`EditCancelled`. Tests in the existing inline `mod tests`.
- i18n: two keys in `message_key.rs`, `dictionary.rs`, `en.toml`, `ja.toml`, `i18n/tests/i18n.rs` (add to the existing lists). Texts exactly as in the RFC ("Say it when it matters"); the ja text is unreviewed, say so.
- `docs/src/export-spec.md`: one paragraph: for tracing, a raster longer than 512 px is scaled down; the raster outputs are rendered from the full source and are not affected; `favicon.svg` of a photograph is an approximation and may be large (and the advisory). `docs/src/functional-spec.md` §10: one toast row.

## 5. Non-change scope (you must not touch)

The raster pipeline (`resize`, `canvas`, `flatten`, PNG/ICO outputs), `ExportPlan` and persisted settings (**no new setting**), preset parameters in `config_for`, the `design`-related or any other dependency, `Cargo.lock` beyond what step 2 allows, ingest, SVG sources (they are copied verbatim, not traced), cancellation or progress UI, input size limits (RFC 014), `rfcs/`, `ROADMAP.md`.

## 6. Required tests (design-derived; write them first)

`vectorize_cap.rs`:
| # | Test | Assertion |
|---|---|---|
| 1 | `trace_dimensions` table | within cap unchanged (512,512,512 → same; 100×50); 1024×512 → 512×256; 512×1024 → 256×512; 4000×3000 → 512×384; 5000×1 → 512×1 (never 0); cap 1 edge; exact aspect within rounding |
| 2 | flat-logo golden | a flat two-colour fixture (below the cap, for example 256 px built in the test) traced with the new code gives **byte-identical** SVG to the output of the **unmodified** code (capture the golden hash first and put it in the test as a constant) |
| 3 | cap applies | with `vectorize_with_cap(img, preset, 64)` a 256-px noise image produces an SVG whose declared width/height are at most 64 and which is smaller than the uncapped trace of the same image (assert on sizes, not on wall time) |
| 4 | aspect | a 2:1 image traced with the cap keeps the declared 2:1 ratio (within 1 px) |
| 5 | exporter integration | `run_in_memory` with a large raster produces a `favicon.svg`; raster outputs (`favicon-32.png`, `favicon.ico`) are **byte-identical** to the same plan with `include_svg = false` (the cap must not touch them), and the existing exporter tests still pass |
| 6 | error path | `vectorize_with_cap` with width 0 or an invalid cap is an `Err`, no panic |

App (inline `mod tests`): the advisory pushes exactly one toast for a `favicon.svg` of 2 MB + 1 byte and none for 2 MB; none when there is no `favicon.svg`; not repeated by `EditCancelled` + `ShowLastResultRequested`; the toast's lifetime follows `toast_time::seconds_for`. i18n: the two keys exist, are non-empty, and `ja` differs from `en`. A test for the `spawn_blocking` error mapping if it can be written without a GUI; otherwise state the limit.

## 7. Required evidence

1. Gates, chained: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit` (expected: 0 vulnerabilities, 12 warnings; no new package in `Cargo.lock`).
2. Step 1 evidence (before) and its repeat **after the fix** with the same script and fixture: Ctrl+2 during the conversion must change the page within about 1 s; one capture; plus the conversion time after the fix in release (expected under 1.5 s for the noise fixture). If the page still does not change after `spawn_blocking`, stop and report.
3. Core timing table after the fix (same sizes as step 1.5) and the SVG sizes: the 1400-px noise `favicon.svg` should be about 3 MB or less.
4. GUI check of the advisory with the same noise fixture: one toast visible, none for a normal flat logo (for example the repository's `packaging/windows/Assets/Square150x150Logo.png`, which must **not** trigger it). One picture at most; say which backend and window size.
5. CI: the run of the final commit and each of the five job conclusions (if a job shows `in_progress` while the run is completed, read the raw job log).

## 8. Acceptance criteria

Step 1 recorded (cause confirmed or the job stopped with a report); the cap and the golden test in place; the CPU work off the executor; shortcuts served during a heavy conversion; the advisory only for a large SVG; docs updated; gates and CI green; no new setting, no new package in the lock.

## 9. Prohibited shortcuts

No wall-clock assertions in committed tests, no change to raster outputs, no new setting, no cap on SVG sources, no skipping step 1, no `spawn_blocking` "because it should work" without the measurement, no weakened tests, no red pushes, no tags or history rewriting.

## 10. Known risks

- If `spawn_blocking` does not remove the starvation, the cause is elsewhere (for example the shortcut stream): stop and report; the architect amends RFC 030.
- Downscaling changes the traced `favicon.svg` of a large flat logo (not below the cap). Accepted by the RFC; the golden test proves nothing changes below it. Record the SVG header (declared size) before and after for a 1024-px flat logo.
- A panic inside the blocking closure surfaces as a join error: map it to an `AppError`.

## 11. Required review request

`.git-exclude/review-request/rfc-030-vectorization-cost/README.md` with everything required by section 7 of the mid-capability operating instructions, plus: the step 1 measurements (shortcut starvation before/after, core timing before/after), the code of the cap and the task wrapper, the lock diff, gates and CI verdicts, GUI evidence with backend and window size, skipped checks, deviations, open questions. Report to the owner only the path of the `README.md`.
