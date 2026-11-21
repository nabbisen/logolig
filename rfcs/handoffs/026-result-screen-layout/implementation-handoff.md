# Developer Handoff — RFC 026: Back and Download always reachable; the Settings page scrolls (job J9)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, 2018+ module style, tests in separate files, English only; relative paths with `cd`). Governing RFC: [RFC 026](../../done/026-result-screen-layout-small-windows.md), **sections "Amendment 2026-11-12" and "Decisions" (D1–D6 are binding)**. Findings source: `.git-exclude/review-request/rfc-035-headless-ui-test/README.md` §5 and `evidence/findings-today.txt`. **Starts after J13b is reviewed.**

Owner philosophy: clean, safe and secure, robust design; users must not be confused; less is more. **Layout only: no message, state, text, icon or style changes.**

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit **0 vulnerabilities, 12 allowed warnings** (the baseline since the `chrono` advisory)); CI waited for before the next push; no tags, force-push or history rewriting.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Are "Back" and "Download all (ZIP)" fully visible, with something to press under them, on every Result combination (7 and 16 cards × 7 sizes × both languages × Light/Dark by the harness's rules)? | headless | P2 entries added **first, shown red**, green after the fix |
| Does the Settings page no longer squeeze anything? | headless | the 26 `KNOWN` entries of `overflow.rs` deleted; P1b green on the whole matrix without them |
| Is everything else unchanged? | headless, machine comparison | snapshots of **Home, Converting, Customize** (Light and Dark, 1280 × 720 and 360 × 640) taken before and after with the same temporary test and the CPU renderer: **byte-identical** (`cmp`); Result and Settings are expected to differ |
| Do the changed screens look right? | headless snapshots, looked at only where `cmp` says they differ | the worst case (Result, 16 cards, 360 × 640, Japanese, Light) after the change, plus one baseline (Result, 7 cards, 1280 × 720, English, Light) before and after; describe every difference; also Settings at 800 × 600 before/after |
| Does the real window agree? | real UI, **one** capture | the Result screen at the size the app opens at (native Wayland, via `scripts/gui-session.sh`), after the change: Back and Download all both visible; compare their pixel rectangles with the harness's `bounds()` as in JH §2 (state the difference) |

The theme proof (RFC 035) still holds for Result; if a new difference between Light and Dark appears, the proof test will say so.

## 1. Work (commits as you see fit, one logical step each; tests first)

1. **P2 entries first (red):** add "Back" and "Download all (ZIP)" for the Result cases to `reachable.rs` (the structure is there: `controls()` already lists them; they are currently excluded because they fail). Show them failing on today's tree (paste the output).
2. **Result view** (`ui/result_view.rs`): per D1/D2 build `column![header, middle, footer]` with `header` = the Back row (today's `action_row` content, moved), `middle` = the `scrollable` of everything else (width and height fill), `footer` = the "Download all" container (centred, today's button). Nothing else about the view changes; the `action_row` and its `Space` go. Keep the doc comments truthful.
3. **Settings page** (`ui/settings_page.rs`): wrap the body in `scrollable(...)` `width(Length::Fill)` `height(Length::Fill)` as `customize_page.rs` does; no other change.
4. Delete the 26 Settings entries of `overflow.rs::KNOWN`; the harness must be green with an empty list (if other entries remain they are named in the review request).
5. The unchanged-pages proof (see §0): temporary uncommitted test as in J13a; patch kept in the review-request folder; no golden images committed.
6. Update the comments in `reachable.rs` ("RFC 026 adds its entry with its fix") to the present state.

## 2. Required evidence (review request: `.git-exclude/review-request/rfc-026-result-layout/README.md`; report only its path)

§0 table filled in; the red output of step 1 and the green output after; the `cmp` results; the pictures that differ with your description; the real capture and the pixel comparison; gates and CI by commit; the test-list diff (named additions, none removed); harness time and the `test` job time on CI against the previous run.

## 3. Acceptance criteria

P2 asserts Back and Download all on all Result combinations and passes; `overflow.rs::KNOWN` has no Settings entries; Home, Converting and Customize are byte-identical; no message, state, text, style or size change; gates and CI green at every pushed commit.

## 4. Prohibited shortcuts

No threshold change, no `#[ignore]`, no new `KNOWN` entries, no change to button sizes or styles (RFC 034), no keyboard/focus code (RFC 027), no new `#[allow]`, no tags, force-push or history rewriting.
