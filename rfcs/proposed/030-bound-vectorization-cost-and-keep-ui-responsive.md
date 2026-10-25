# RFC 030 — Bound the cost of vectorization and keep the UI responsive during conversion

**Status.** Proposed
**Priority.** P1 (a default-settings hang on a common input; architect recommends 1.27.0, owner decides scope)
**Tracks.** Found by the RFC 019 phase 2 implementer (shortcuts not served during a long conversion) and quantified by the architect.
**Touches.** `crates/app/src/task_queue.rs` (`convert_task`, `build_preview_task`), `crates/core/src/services/vectorize.rs` and `exporter.rs` (input size cap), `docs/src/export-spec.md`, tests.

## Summary

Converting a **photographic or noisy** raster with the default settings (`include_svg` and `vectorize_on_raster` are both on) runs the vtracer tracer on the full-size image. Its cost grows faster than the pixel count and it runs as synchronous CPU work inside an `async` block on a tokio worker. Result: the application shows "Importing…" for seconds to minutes, the generated `favicon.svg` can be tens of megabytes, and while it runs the keyboard shortcuts, drag-and-drop events and window events are not served (the mouse still works). Flat logos, the intended input, are unaffected.

## Evidence

- **Measured by the architect (release build, `vectorize::vectorize` with `VtracerPreset::Default`, synthetic inputs, one machine, `.git-exclude/tmp/vec-time/`):**

| Input | 256 px | 512 px | 1024 px | 1400 px |
|---|---:|---:|---:|---:|
| flat two-colour logo | 4 ms | 13 ms | 49 ms | 93 ms |
| gradient with noise | 76 ms | 551 ms | 3.7 s | 10.2 s |
| pure noise | 93 ms | 752 ms | 4.1 s | 13.1 s (SVG 24 MB) |

  From 1024 to 1400 px (1.9 × the pixels) the time grows 2.8 to 3.2 ×: superlinear. A 12-megapixel photograph (4000 × 3000) is therefore expected to take minutes (extrapolation, not measured). These are synthetic worst-case-like images, not real photographs; a real photograph is somewhere between "gradient with noise" and "noise".
- **Reported by the RFC 019 phase 2 implementer (debug build, 1400 × 1400 noise PNG, about 50 s):** during the conversion Ctrl+O, Ctrl+S and Ctrl+2 were not acted on until it ended, a click on the sidebar worked, the process had 46 threads with one tokio worker at 100 % CPU. The implementer's explanation (a blocked worker starving the subscription stream) is a **hypothesis**, not verified; the same would affect drag-and-drop and resize events and toast expiry.
- **Verified (source):** `task_queue.rs::convert_task` is `Task::perform(async move { run_convert(&asset, &plan) }, ...)`: `run_convert` is synchronous CPU work. `build_preview_task` likewise.
- RFC 011 (stale results) recorded the same suspicion as an open question (Q1).
- Default settings: `ExportPlan::default()` has `include_svg = true`, `vectorize_on_raster = true`.

## Design

1. **Run CPU-bound work on a blocking thread.** `convert_task` and `build_preview_task` use `tokio::task::spawn_blocking` (the application already depends on `tokio` through iced; no new dependency), so the executor keeps serving events. The effect on shortcut starvation is verified by a repeatable measurement (see Tests); if the hypothesis is wrong the RFC is amended with the real cause.
2. **Bound the tracing cost.** Before vectorizing a raster, scale it down so that its longest side is at most **`VECTORIZE_MAX_SIDE` (proposal: 512 px)**, with the project's own resize (Lanczos3). Rationale: the SVG is resolution independent and every raster output is rendered from the source, not from the traced SVG; the favicon sizes are at most 512 px by default (1024 at most by the setting range); the cost at 512 px is under a second even for noise (table above) and the SVG stays near 3 MB at worst. Flat logos lose nothing visible. Decision D1 on the number.
3. **Say it when it matters.** If the source was scaled down for tracing and/or the resulting SVG exceeds a size (proposal: 2 MB), the existing advisory mechanism shows one toast: "This image looks like a photograph, so favicon.svg is only an approximation. Turn off SVG output in Customize if you do not need it." (texts decided with the handoff). No new setting.
4. **Out of scope** (separate RFC if needed): a cancel button, a determinate progress indicator, size limits on input files (RFC 014), skipping the SVG automatically for photos.

## Alternatives considered

- Only step 1: the UI stays responsive but a photo still takes minutes and produces a huge SVG. Rejected as insufficient.
- Only step 2: bounded cost but a long conversion would still starve events for up to a second; acceptable alone, but step 1 is the robust fix.
- Disable vectorization for raster sources by default: changes a product decision (default output set); owner decision, not proposed here.
- Time-box the tracer (abort after N seconds): vtracer has no cancellation hook; would need a thread and an abandonment policy. Rejected as complex.

## Tests (design-derived)

- Core: vectorizing a 4000 × 3000 noise image finishes within 3 s in a release test run, with an SVG under the size bound (a `#[ignore]`-free test with a smaller image if CI time requires it: assert on the downscale decision and the output size instead of wall time).
- Core: a flat logo at 1024 px produces the same SVG structure before and after (golden hash of the traced SVG for a fixture below the cap, which must be unchanged because the cap does not apply to it).
- App: a regression test or measurement that a 20-second blocking task does not delay delivery of a keyboard event: the architect's proposal is a scripted GUI measurement (Wayland, `wtype`): start a conversion of a heavy image, send Cmd+2, record the time until the page changes; required result: under 1 s while converting. Evidence by state, one run.
- Docs: `export-spec.md` states the cap and that `favicon.svg` of a photograph is an approximation.

## Decisions requested

- **D1** Cap 512 px (architect's proposal), or another number?
- **D2** Add as job **J4b** to release 1.27.0? Recommended: the defect is on the default path for a common input (photos and JPEGs are accepted formats, and the educational JPEG toast invites them), a 1.27.0 user dropping a photograph can see the application appear to hang for minutes. Size: M (a spike-sized evidence step first, then a small change).

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| Downscaling changes `favicon.svg` of large flat logos | Low | Only above the cap; golden test below it; the SVG is an approximation of the raster anyway |
| `spawn_blocking` does not remove the starvation | Medium | Measure first (the shortcut test); amend the RFC with the real cause |
| Panics inside `spawn_blocking` | Low | The join error maps to an `AppError` instead of a silent loss |
