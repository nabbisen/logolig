# Developer Handoff — RFC 010: Non-square raster sources in square outputs

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and the project rules in `.git-exclude/rules/` (Rust 2024, Rust 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden).

Governing RFC: [RFC 010](../../accepted/010-non-square-raster-sources.md) (Accepted). The RFC defines **what and why**; this handoff defines **how to implement and prove it**. If you find a conflict between the two, stop and send a clarification request; do not resolve it yourself.

Owner philosophy to keep in mind while deciding local details: a clean, safe, robust design; users must not be confused or misled.

Dependencies: none. Independent of the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) and 004 (different files). Do not wait for them.

## 1. Purpose

Square outputs (favicon PNGs, ICO frames, `apple-touch-icon.png`, monochrome variants, and the preview rasters) must never distort a non-square PNG/WebP/JPEG source. They must be contain-fitted on a transparent square canvas, as SVG sources and the Microsoft app logos already are. The user is told about it once per loaded image.

## 2. Background (facts, verified by source reading; the first task proves the defect by execution)

- `crates/core/src/services/exporter.rs::render_at_size`, raster branch: `resize::resize(src, size, size, plan.algorithm)` with no aspect handling.
- `crates/core/src/services/preview.rs::render_at`: same direct `resize(.., size, size, ..)` for PNG/WebP/JPEG.
- `crates/core/src/services/canvas.rs::contain_on_transparent_canvas(src, target_w, target_h, algorithm)` already exists and is used only by `render_to_contain_canvas` (Microsoft logos).
- `services/rasterize_svg.rs::rasterize` already letterboxes non-square SVGs. Do not change it.
- `keep_transparency = false` flattens **after** rendering (`flatten_to_white` at the end of `render_at_size`). Keep that order; the padding then becomes white.
- `SourceAsset::intrinsic_size: Option<(u32, u32)>` is parsed from headers at ingest.
- Existing tests: `crates/core/tests/exporter.rs` has `microsoft_wide_logo_preserves_aspect_ratio_with_transparent_padding` (uses a **square** source, so it does not cover this defect). `tests/fixtures.rs` has only square fixtures.

## 3. Change scope (you may touch)

Core (`crates/core`):
- `src/services/canvas.rs`: add one public helper (name fixed here): `pub fn contain_square(src: &Rgba8, size: u32, algorithm: ResizeAlgorithm) -> Result<Rgba8, AppError>`. Behaviour: if `src.width == src.height`, return exactly `resize::resize(src, size, size, algorithm)` (no extra canvas, no extra copy, byte-identical to today); otherwise `contain_on_transparent_canvas(src, size, size, algorithm)`. This additive public function is approved by the architect.
- `src/services/exporter.rs::render_at_size`: raster branch calls `canvas::contain_square`.
- `src/services/preview.rs::render_at`: PNG/WebP/JPEG branches call `canvas::contain_square`.
- `tests/` (see §6) and `Cargo.toml` `[[test]]` registration for each new test file (**`autotests = false` is set; an unregistered test file is silently not run**; verify with `cargo test -p logolig --test <name>`).
- `tests/fixtures.rs`: add helpers for non-square fixtures (see §6). Do not change existing helpers.

i18n and app:
- `crates/core/src/message_key.rs`: add `ToastNonSquareInputTitle`, `ToastNonSquareInputBody` next to `ToastJpegInput*`.
- `crates/i18n/src/dictionary.rs` (struct fields + `lookup` arms) and `crates/i18n/locales/en.toml`, `ja.toml`: the two new keys. `crates/i18n/tests/i18n.rs`: extend the existing all-keys/parity tests so they include the new keys.
- `crates/app/src/app.rs`: in the `IngestCompleted(Ok(asset))` handler push the advisory toast (see §5). Add a small private helper (for example `fn is_non_square(size: Option<(u32, u32)>) -> bool`) in the same file.
- App tests: add to the existing `#[cfg(test)] mod tests` in `app.rs`. (That module is a known rule violation scheduled for RFC 017; do **not** restructure it here.)

Docs:
- `docs/src/export-spec.md` ("Quality strategy"): add the contain-fit rule for every source kind.
- `docs/src/functional-spec.md` §10 toast table: add the non-square advisory row.

## 4. Non-change scope (you must not touch)

- `rasterize_svg.rs`, `vectorize.rs` (favicon.svg from rasters), `resize.rs`, `ResizeAlgorithm`, `flatten.rs`, Microsoft logo path (`render_to_contain_canvas`).
- `ExportPlan`, `PersistedSettings`, any serialized format. **No new setting.**
- Square-source output: must stay **byte-identical**.
- No new dependency; no `unsafe`; no change to `Cargo.toml` other than `[[test]]` entries.
- No unrelated refactoring (especially do not split `app.rs` or `exporter.rs`; RFC 017 owns that).
- Do not edit RFC files and do not move them between folders; the architect does that.

## 5. Required behaviour

1. **Contain-fit** for all square raster outputs: favicon PNG sizes, ICO frames, apple-touch icon, `mono/` PNG and ICO, and preview rasters (16 and 120). Content is scaled to fit, centred, never cropped or stretched. Padding is transparent; with `keep_transparency = false` the padding is flattened to opaque white together with everything else.
2. **Extreme aspect ratios** must not panic or error: the fitted side is clamped to at least 1 px (already true in `contain_on_transparent_canvas`; prove it by test).
3. **Advisory toast.** On `IngestCompleted(Ok(asset))`, if `asset.intrinsic_size == Some((w, h))` and the sides differ by more than 2 % (`abs(w - h) * 100 > 2 * max(w, h)`), push **one** transient warning toast (use `push_warning_toast`). Applies to every source kind (SVG included, because padding occurs for SVG too). Not shown when `intrinsic_size` is `None`, when the image is square or within the 2 % tolerance, nor again on `ShowLastResultRequested`. The 2 % tolerance avoids false alarms from the `ceil()` rounding in `parse_svg_size`.
4. **Texts** (architect-defined; do not reword):
   - en title: `Your logo is not square`
   - en body: `It is shown in full, with empty space around it, so the icon may look small. A square version of your logo gives a larger icon.`
   - ja title: `ロゴが正方形ではありません`
   - ja body: `ロゴ全体を表示するため、アイコンの周りに余白が付きます。正方形のロゴ画像を用意すると、アイコンを大きく表示できます。`
   Keys use the existing naming (`toast_non_square_input_title`, `toast_non_square_input_body`). The ja text has not had a native review; say so in the review request.

## 6. Required tests (design-derived; write the first one before any production change)

Create `crates/core/tests/non_square.rs` (register it in `Cargo.toml`). Do not add to `tests/exporter.rs` (697 lines; the project rules want large test files split, not grown). Add fixtures such as `png_rgba(width, height, [r,g,b,a]) -> Vec<u8>` (build with the `image` crate's PNG encoder, as the existing fixtures do).

| # | Test | Assertion |
|---|---|---|
| T1 | 64×32 opaque red PNG, plan = PNG size 32 only | output 32×32; top row and bottom row alpha 0; centre alpha 255; opaque bounding box ≈ 32 wide × 16 high (±1). **Must fail on the unmodified code** (record that failure as evidence), then pass. |
| T2 | 32×64 mirror of T1 | left/right columns alpha 0; box ≈ 16 × 32 |
| T3 | T1 with `keep_transparency = false` | corner pixel `(255,255,255,255)`; centre red, alpha 255; no pixel has alpha < 255 |
| T4 | Oracle for square sources (PNG 4×4, WebP 8×8, JPEG 8×8; sizes 16 and 32; also `apple-touch-icon.png`) | exporter PNG bytes equal `encode_png::encode(&resize::resize(&decode(src), size, size, alg))` |
| T5 | ICO and apple-touch for a 2:1 source | every ICO frame (decode with the `ico` crate as existing tests do; the workspace `image` dependency also enables `ico` now, so either works, but be consistent) has a transparent corner pixel and an opaque centre; apple-touch is 180×180 with transparent top row |
| T6 | Monochrome with a 2:1 source | `mono/favicon-32.png` padding is transparent and grey values are unaffected |
| T7 | `build_preview` with a 2:1 source | `tab_16` (16×16) and `icon_120` (120×120): top row alpha 0, centre opaque |
| T8 | 1000×1 source to 16 px | no error; exactly one visible row; output 16×16 |
| T9 | Non-square SVG still letterboxes (existing behaviour) | unchanged result; existing `tests/rasterize.rs` passes untouched |

App tests (in the existing `app.rs` test module): non-square `IngestCompleted(Ok)` adds exactly one toast whose title equals the translated `ToastNonSquareInputTitle`; square asset adds none; 101×100 (inside tolerance) adds none; `intrinsic_size = None` adds none; the toast is not duplicated by `ShowLastResultRequested`.

i18n: parity tests include the new keys in `en` and `ja`.

## 7. Required evidence capture (do this before changing production code)

1. On a clean checkout of the base commit, run the gates (§9) and keep the logs.
2. Write a throwaway program or test (not committed) that dumps all default-plan artifacts for the square PNG/WebP/JPEG/SVG fixtures, record the SHA-256 of each file, and repeat it after your change. Put the two tables in the review request. Square sources must match exactly. Include `monochrome = true` and `include_microsoft_app_logos = true` variants.
3. Manual GUI check (rules: `.git-exclude/rules/project-instructions-gui-common.md`; resize to 1280×720; focus before typing): drop a 2:1 PNG, confirm the toast, open the Result screen, and compare `favicon-32.png` before/after (save both images under the review-request directory). Capture screenshots with `niri msg action screenshot-window --id <id> --path <relative path>`; do not use ImageMagick `import`. If the GUI check cannot be run, say so and say what risk remains.

## 8. Acceptance criteria

- T1–T9, the app tests and i18n tests exist and pass; T1 is shown to have failed before the fix.
- Square-source artifacts are byte-identical before/after (table in the review request).
- Docs updated as in §3.
- `docs` and code agree on: contain-fit, 2 % tolerance, once-per-ingest advisory.
- No change outside the change scope (the reviewer will run `git diff --stat` against the base).

## 9. Required validation (report exactly what was and was not run)

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Baseline at commit `9d79f25`: fmt clean, clippy clean, 147 tests passed, 0 failed. `cargo audit` is not part of this task.

## 10. Prohibited shortcuts

- Do not make the test pass by changing expected values after seeing output; derive them from §6.
- Do not special-case sizes (for example only 32 px), or only the exporter and not the preview.
- Do not add a setting "stretch / fit / crop".
- Do not clone the canvas helper; reuse `contain_on_transparent_canvas`.
- Do not hardcode toast strings in Rust.
- Do not touch unrelated warnings or clippy lints elsewhere.

## 11. Security and compatibility constraints

- No new input parsing, no file I/O, no dependency. Persisted settings are unchanged, so old and new settings files stay interchangeable.
- Behaviour change is confined to non-square raster sources (accepted by the owner in RFC 010 D1).
- Public API: one additive public function (`canvas::contain_square`); nothing else may change.

## 12. Known risks

- Rounding: for sources whose aspect ratio is almost, but not exactly, square, `round()` can produce `fitted_w == size` and `fitted_h == size - 1`, which is correct and expected; do not "fix" it.
- The monochrome and ICO paths call `render_at_size` repeatedly; do not cache or restructure.
- Toast stacking: the JPEG advisory and the non-square advisory may both appear for the same file. That is accepted.

## 13. Delivery

Work on branch `rfc-010-non-square`. Commit in small steps (test first, then code, then docs; match the repository's short imperative subject style). Push the **branch**. Do not merge to `main`, tag, or release; the architect merges after review. Do not change any RFC status.

## 14. Required review request

Place it in `.git-exclude/review-request/rfc-010-non-square/` with a `README.md` entry point. It must contain everything required by section 7 of the mid-capability operating instructions: task and RFC identifier; handoff; implementation summary; requirements addressed (map to §5); changed files; implementation decisions; deviations; tests added or changed; commands executed with results; skipped or unexecuted checks (with reasons); known limitations; the pre/post SHA-256 tables; GUI evidence locations; the T1 "fails before" log; unresolved issues; and the requested review focus. Distinguish new defects from pre-existing ones. Report to the owner only the path of the `README.md`.
