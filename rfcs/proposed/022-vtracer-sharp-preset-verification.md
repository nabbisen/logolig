# RFC 022 — Verify the vtracer "Sharp" preset

**Status.** Proposed
**Priority.** P3 (spike first; no behaviour change until evidence exists)
**Tracks.** Possible mismatch between a preset's name and its effect. Raised by RFC 018 D2.
**Touches.** `crates/core/src/services/vectorize.rs`, `domain/vtracer_preset.rs`, Customize page labels, docs.

## Summary

The "Sharp" preset is documented as the logo-quality choice and, in older documents, as the default. Its only change from vtracer's defaults is `corner_threshold: 60 → 80`, justified by the comment "no corner rounding". By vtracer's own definition the opposite may be true.

## Evidence

- Verified (code): `config_for(Sharp)` is `Config { corner_threshold: 80, ..Config::default() }`; the comment says "Logo preset: no corner rounding". `PhotoRich` uses 45.
- Verified (dependency docs, vtracer 0.6.5 README): `corner_threshold` is the "Minimum momentary angle (degree) to be considered a corner". A *higher* minimum angle means fewer joints qualify as corners, so more of the outline is smoothed. That would make 80 *rounder* than 60, not sharper, and 45 (PhotoRich) *more* corner-preserving than Sharp.
- Not verified: the visible effect. This is an inference from the parameter definition; vtracer's behaviour with `mode: Spline` may differ in practice. The 1.4.1/1.4.2 notes record that tuning was done by user testing of one parameter at a time, but no evidence artefact is kept.
- Current default is `VtracerPreset::Default` (RFC 018 D2 keeps it), so the default output is unaffected by this question. Only users who move the Customize slider to the "Simple" end are.
- Not verified: how the slider labels ("Simple" ↔ "Detailed") map onto Sharp / Default / PhotoRich in the UI.

## Plan (spike, no product change)

1. Pick fixtures: a geometric logo (the hexagon mark), a text-like logo, a gradient/photo-like image.
2. Render with `corner_threshold` 30, 45, 60, 80, 100 at otherwise default settings; record SVG size, path/node counts and screenshots of the result at 16, 64 and 512 px.
3. Write findings in this RFC; decide whether to (a) keep the preset, (b) change the value, or (c) rename the preset so name and effect agree.
4. If the preset changes, the change is a separate small RFC or an amendment here, with golden tests, because it alters SVG output for those who selected it.

## Acceptance criteria

A short evidence section exists; the preset's name, comment, parameter and documentation agree, or a decision to leave them is recorded with reasons.

## Risk

None for the spike. Any later change affects only users who explicitly chose that preset.
