# RFC 022 — Verify the vtracer "Sharp" preset

**Status.** Implemented (1.29.0) (the spike found Sharp wrong; Sharp is 45 and the labels say only what is measured). Earlier: Accepted (architect, under the owner's delegation of 2026-11-22 "review them carefully with my fundamental philosophy"; release 1.29.0; decisions in the section "Decisions 2026-11-22")
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

## Decisions 2026-11-22 (architect)

Spike only (evidence), scheduled as job H2 after the readable-interface work; no behaviour change and no relabelling without evidence. Since RFC 021 the UI names the preset "Sharp (logos, icons)": the name is a promise to the user, so the spike's conclusion decides whether the name, the parameters or neither change (a name change is a user-text change in both languages and needs the owner's confirmation of the wording).

## Result of the H2 spike and decisions (architect, 2026-11-26)

Spike: `.git-exclude/review-request/rfc-022-sharp-spike/` (reviewed in `.git-exclude/reviewed/rfc-022-sharp-spike-review-2026-11-26.md`). Measured through the application's own code path on 25 fixtures (hard-edged and anti-aliased regular polygons, an arrow, a circle, a colour field), with `Sharp` byte-identical to `corner_threshold` 80 and `Default` to 60 on all 25: vtracer's `corner_threshold` is the **minimum** turn angle that counts as a corner, so a **higher** value smooths more; **`Sharp` (80) is never sharper than `Default` (60)**, identical on 90° and acute corners, **rounder** on obtuse ones (pentagon 126° traced against 112°) and larger (51 against 27 segments). 45 keeps corners at least as well as 60 on all 22 corner fixtures, with no more segments than `Default` on 21 of 22; 30 and 15 start to cut shallow corners. On a smooth gradient all three presets flatten the picture to one colour: "gradients" is unsupported for `PhotoRich`, whose measurable gain is a few percent on noisy colour images.

**Decisions:** (1) **`Sharp` becomes `corner_threshold` 45** (option b): one value, the lowest without the cutting artefact, the value `PhotoRich` already uses; it changes the SVG output of users who chose Sharp (corners of 100–135° become sharper, fewer segments), so a release-notes line; no golden test pins Sharp. (2) The labels claim **only what is measured**: "crisp edges" stays for Sharp (now true); "gradients" leaves PhotoRich's label, which says "a little more color detail". (3) **`Default` is not changed** (it also rounds 120° corners; changing it changes everybody's output: a separate decision, not now, recorded here). Job H2b.

