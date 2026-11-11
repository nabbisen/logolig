# Developer Handoff — RFC 035: the headless UI test level (job JH)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (**read the updated `project-instructions-gui-common.md`, in particular "GUI Evidence and Automation Rules" and "Test Optimization Rules"**; Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 035](../../accepted/035-headless-ui-test-level.md). On any conflict, stop and send a clarification request. Previous job: J11 (RFC 011), reviewed first.

Owner philosophy: clean, safe and secure, robust design. **No product behaviour changes in this job.** Production code changes only if the spike shows `widget::Id`s are needed, and then only those, listed.

**Workflow:** `main`; `git pull --ff-only` first; one commit per step; gates chained with `&&` before every push; wait for each commit's CI before the next push; never a red push; no tags, no force-push.

## 0. Evidence plan (the rules require it before work starts)

| Question | Level | Where it is answered |
|---|---|---|
| Does `iced_test` build and run in `cargo test` without a GPU, locally and on the CI runner? | headless | local run; the first pushed commit's CI |
| Does the headless level see today's RFC 026 defect ("← Back" not fully visible, Result screen, default plan, 1280 × 720)? | headless | a local, uncommitted, red test; its output pasted in the review request |
| Which selector reaches the controls; is any production `Id` needed? | headless | spike notes |
| How far do the simulator's bounds differ from the real window's? | **real UI, once** | one native-Wayland capture (below) compared with the simulator's numbers |
| What does the dependency cost (lock, audit, release binary)? | recorded state | `git diff Cargo.lock`, `cargo audit`, `cargo tree -e normal` before/after |
| Do the properties P1 (no horizontal overflow) and P3 (hit area ≥ 24 px) hold today across the matrix? | headless | the matrix test; failures are findings, not committed red |
| Can the properties fail (no vacuous pass)? | headless | the two self-tests (design 6) |

Captures that remain: **exactly one** real capture (fidelity), plus `Simulator::snapshot` PNGs of the single worst-case combination and one baseline for the review request. Nothing else is photographed. Real-window rules apply to the one capture: unlocked session (`loginctl show-session <id> -p LockedHint`; if locked, stop and report), isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME` with a pinned `settings.json` (Light, English), one instance, window chosen by pid, `niri msg action screenshot-window --id … --path <absolute>`, state the backend and the window size obtained, assert after the step, stop at the first anomaly. Pixel positions are measured on **this** capture with a script run **after** the capture.

## 1. Step 1 — Spike (uncommitted working tree; report; commit only what is listed in step 2)

1. Add `iced_test = "0.14"` as a dev-dependency (workspace table plus `crates/app/Cargo.toml`). Write a throw-away test that builds the Result view (7 cards, default plan, English, Light) with `Simulator::with_size`, `window_size` set the same, and finds the "← Back" control. Run it on the CPU backend (`ICED_TEST_BACKEND=tiny-skia`; if the variable is not the right switch, find out how the renderer is chosen and say so). Report: does it run with no GPU (try with `WGPU_BACKEND`/no adapter, or in a container if available)?
2. Assert `visible_bounds()` of the Back control equals `bounds()`. On today's code this should be **red** (the RFC 026 defect). Paste the output. If the harness reports it visible, say so and investigate before going on (the simulator's metrics may be too generous: that is the fidelity question).
3. How are controls found? `iced_selector` accepts text and ids; the application's labels are localized strings from the i18n crate: use the string the i18n crate returns for the test language, not a literal. List every place where a control cannot be found without an `Id`; propose the smallest set of `Id`s.
4. The fidelity capture (one real window, native Wayland, Result screen, default plan, 1280 × 720 as obtained at launch): measure the pixel rectangles of "Download all" and "← Back"; compare with the simulator's `bounds()` for the same state and size. Report the difference in pixels per rectangle and the cause you can name (font, line height). Which font does the simulator use, which the real window? (The app sets only the Lucide font; the simulator defaults to Fira Sans unless `Settings::default_font` is given.)
5. Cost: `git diff --stat Cargo.lock`, the new crates by name and version, `cargo audit` (vulnerabilities and warnings, compared with 0 and 11), `cargo tree -e normal -p logolig-app` before and after (**must be identical**).

**Stop rule:** if (1) or (2) fails, or the release tree changes, **stop**, do not build the harness, and send the findings to the architect (fallback: nested compositor).

## 2. Step 2 — The harness (one commit, or two if the matrix needs its own)

1. Files (declared like the existing test files, `#[cfg(test)]`): `crates/app/src/ui_layout_tests.rs` (declares the submodules) with `ui_layout_tests/{support,overflow,hit_area,reachable}.rs`. Use these names unless the module layout forces a change, and say why.
2. `support.rs`: `fn view_at(state: &AppState, size: Size, theme: &Theme) -> Simulator<…>`; helpers to build states for the screens (reuse `test_support.rs`; add to it, do not copy); `fn matrix()` yielding every (size, screen, language, theme); a function that collects the bounds of all interactive and text widgets (a structural walk through `Simulator`/`iced_selector` operations, as the spike found).
3. **Matrix (binding):** sizes 1280 × 720, 1024 × 600, 800 × 600, **767 × 480, 768 × 480**, 640 × 480, 360 × 640; screens Home, Converting, Result (default plan **and** every option on), Customize (Advanced open), Settings; every language of the i18n crate; Light and Dark. Report the run time. Over about 30 s: report, and the architect decides how to trim; do not trim on your own.
4. **P1 `overflow.rs`:** no text or interactive widget has `bounds().x + width` beyond the window width.
5. **P3 `hit_area.rs`:** every button, checkbox, radio and slider has bounds ≥ 24 × 24 logical px. If controls fail today, **do not commit a red test**: list the failing combinations (screen, size, control, measured) in the review request as findings for RFC 034 and J9/J10, and commit the property with the **set of known exceptions as an explicit, named, commented list** that those jobs shrink to empty (a test that names each exception is honest; a loosened threshold is not).
6. **P2 `reachable.rs`:** the function and its list of controls (`Back`, `Download all (ZIP)`, `Choose file…`, navigation items). Only entries that hold today are asserted; entries that fail today (expected: Back at 1280 × 720 Result) are **not** committed; they are listed as findings. RFC 026 adds its entry with its fix.
7. **Self-tests (so no property passes vacuously):** a test-only view with one widget wider than the window must be reported by P1; a test-only 10 × 10 button must be reported by P3; a test-only clipped widget must make P2's check fail.
8. Docs: a short passage in the existing contributor/testing document under `docs/` (find it; if none exists, add three paragraphs to the nearest testing text, not a new file tree): the three test levels, where the headless tests live, how to run them, the fidelity limit. Nothing else.
9. `Cargo.lock` change: only what `iced_test` brings. Do **not** run a general `cargo update`.

## 3. Required evidence (review request: `.git-exclude/review-request/rfc-035-headless-ui-test/README.md`; report only its path)

- Spike answers (a) to (e) with the pasted output.
- The red RFC 026 reproduction and, separately, the fidelity table (pixel rectangles real vs simulated).
- Dependency cost: new crates, audit before/after, `cargo tree -e normal` before/after.
- The matrix: how many combinations, run time, results per property; the findings list (failing combinations) with exact numbers.
- The worst-case snapshot (largest text is not a setting yet: the narrowest window, the longest-labelled language, Result screen with 16 cards) and one baseline, as PNGs in the review-request folder, each with the question it answers.
- Test list diff: only your named additions (count before: 263).
- Gates chained on every pushed commit: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked` (four existing doc warnings expected). CI per commit by name.
- Real configuration untouched: the owner's `~/.config/logolig/settings.json` modification time before and after the one capture.

## 4. Acceptance criteria

The spike's answers are reported; the release dependency tree is unchanged; the matrix runs in every `cargo test` and in CI; P1, P2, P3 exist with self-tests that prove they can fail; every failure found today is listed as a finding (none committed red); gates and CI green on every pushed commit; production code unchanged except any listed `Id`s; one real capture only.

## 5. Prohibited shortcuts

No golden-image files committed; no thresholds loosened to make a property pass; no `#[ignore]`; no matrix dimension dropped without the architect's decision; no `cargo update`; no new `#[allow]`; no layout change; no tags, force-push or history rewriting.
