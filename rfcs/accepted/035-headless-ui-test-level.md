# RFC 035 — A headless UI test level (layout, size, language and theme matrix without a window)

**Status.** Accepted (architect's decision, 2026-11-12, implementing the owner's "Test Optimization Rules" of the same date in `.git-exclude/rules/project-instructions-gui-common.md`; the owner may veto before the job starts)
**Priority.** P1 for the test process (it unblocks RFC 026 and RFC 021 and shortens every later UI job); no product behaviour changes.
**Tracks.** Level 2 of the owner's test levels ("headless UI") does not exist in this project: layout, clipping, hit-area and size/language/theme combinations are checked only with real windows, one capture at a time.
**Touches.** `Cargo.toml` and `crates/app/Cargo.toml` (one dev-dependency), new test files in `crates/app/src/` (`*_tests.rs` style), `Cargo.lock`, `docs/` (a short testing note), CI (no new job expected). No production code, except possibly a few `widget::Id`s if the spike shows text selectors are not enough.

## Summary

The owner's rules say: check each property at the cheapest level that can prove it, and cover every combination there; layout properties (nothing overflows, every label fully visible, minimum hit area, contrast) belong to a **headless UI** level that runs on every CI run across every text size, language, theme and window size; real windows only for what needs human eyes.

iced 0.14 ships exactly this level as **`iced_test`** (a `Simulator`: build the view for a given window size, find widgets, read their `bounds()` and `visible_bounds()`, click, press keys, take a PNG `snapshot`, all without a window or compositor). The application's view is a pure function of `AppState` (the window size lives in `AppState::window_size`), so a test can build the view of any screen at any size.

This also **removes the small-window blocker** of RFC 026 (Back button clipped at 1280 × 720) and RFC 021 (mobile bottom navigation below 768 px): both become headless tests at the sizes in question, with one worst-case real capture left for human eyes. The nested compositor the architect asked for on 2026-11-12 is then **not needed** (it stays the fallback if the spike below fails).

## Design

1. **Dependency.** `iced_test = "0.14"` as a **dev-dependency only** (workspace-pinned like the other iced crates). It must not enter the release binary: `cargo tree -e normal -p logolig-app` is identical before and after. MSRV: `iced_test` declares 1.88; the project's 1.90 gate holds. New crates in the lock are listed in the review request; `cargo audit` must stay at 0 vulnerabilities and the 11 allowed warnings must not grow without a named reason.
2. **Harness.** One support module builds `(AppState, Message-free view, window size, theme) → Simulator`. The state is constructed the way `test_support.rs` already builds states; the size is set **both** in `AppState::window_size` (the app's own breakpoint logic reads it) **and** in `Simulator::with_size`. The renderer is pinned to the CPU backend for determinism (`ICED_TEST_BACKEND=tiny-skia` or the equivalent set inside the test setup, whatever the spike shows works on a machine without a GPU, such as the CI runner).
3. **Properties (initial set, each a function over the matrix).**
   - **P1 No horizontal overflow:** no text or interactive widget extends beyond the window width.
   - **P2 Primary controls reachable without scrolling:** a named list (the list grows by RFC: "Back", "Download all (ZIP)", "Choose file…", the navigation items) is fully visible (`visible_bounds() == Some(bounds())`) at scroll offset 0. Today this list is empty or small on purpose; RFC 026 and RFC 021 add their entries together with their fixes.
   - **P3 Minimum hit area:** every button, checkbox, radio and slider has bounds of at least 24 × 24 logical pixels (WCAG 2.2 AA minimum). RFC 034 raises the numbers (44 and 32) when it lands.
   - **P4 Labels are not cut:** reported by the spike as possible or not (a `Text` target inside a scrollable is legitimately clipped; the harness must tell the two apart or P4 is dropped and recorded as unreachable at this level).
   - Contrast (RFC 034) is a pure function over palette pairs and needs no simulator.
4. **Matrix.** Window sizes: 1280 × 720, 1024 × 600, 800 × 600, **767 × 480 and 768 × 480 (the mobile breakpoint, both sides)**, 640 × 480, 360 × 640. Screens: Home, Converting, Result (default plan, and a plan with every option on: 16 cards), Customize (with "Advanced settings" open), Settings. Languages: every language the i18n crate ships. Themes: Light and Dark. The matrix size is whatever runs in a few seconds; if it does not, the dev reports the cost and the architect trims it (the rule is "every combination at the cheapest level", so trim by speed, not by dropping dimensions without asking).
5. **Fidelity is a stated limit, not an assumption.** The simulator measures text with its own default font (Fira Sans) unless told otherwise, while the real window uses the system font; heights can differ. The job measures the difference **once** against one real capture (Result screen, 7 cards, 1280 × 720, native Wayland) and records it. Findings from the headless level are then "layout facts at the simulator's metrics"; anything that depends on the real font stays a real-window check.
6. **Captures.** `Simulator::snapshot` PNGs are an evidence tool for review (the worst-case combination plus one baseline, per the rules), written to the review-request folder, never committed as golden files unless a later RFC says so (golden images are brittle across font and renderer versions).
7. **A capture that shows a problem the matrix missed gets a new property or a new matrix entry** (the owner's rule); the review request says so when it happens.

## Phases

- **Phase 0 (spike, no commit unless it passes):** answer five questions with evidence: (a) it builds and runs in `cargo test` on the CI image without a GPU; (b) it **sees the RFC 026 defect** on today's code (Result screen, default plan, 1280 × 720: "← Back" is not fully visible; run locally, red, not committed); (c) which selector reaches the controls (text, `widget::Id`, or a structural walk), and whether production code needs `Id`s (list exactly where); (d) the fidelity difference (design 5); (e) the lock and audit cost (design 1). If (a) or (b) fails, stop and report: the fallback is the nested compositor.
- **Phase 1:** the harness, P1 and P3 over the full matrix (they must pass today or the failures are listed as findings for J9/J10, never committed red), P2 with the entries that hold today, a short testing note in the docs.

## Not in scope

Golden-image comparison; testing colours by eye; clicking through whole user flows (logic and state tests already cover what a click does); changing any layout (RFC 026, 021, 034 do that, with the harness in place).

## Tests and evidence

The harness is itself tested: a deliberately oversized widget in a test-only view must be reported by P1, and a tiny button by P3 (so the properties cannot pass vacuously). Evidence plan, levels and captures are in the handoff.

## Risk

- The simulator's fonts differ from the real ones (design 5): mitigated by measuring and stating it. - `iced_test` is young: pinned to 0.14.0 with the rest of iced, upgraded with iced. - Test time grows: the matrix is sized by measurement. - A property that cannot be expressed headlessly stays a real-window check, said openly.
