# Developer Handoff — RFC 021: selection-state indicators on snora's form controls (job J10; phase 0 look study, then the build)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, 2018+ module style, tests in separate files, English only; relative paths with `cd`). Governing texts: [RFC 021](../../accepted/021-selection-state-indicators.md) (**principles, Q1 decision, the amendments of 2026-11-15 and 2026-11-18**) and [RFC 027](../../accepted/027-keyboard-focus-model.md) (amendment 2026-11-18). **Starts after J13c (snora 0.65) is reviewed.** Reviews to read: `.git-exclude/reviewed/rfc-027-focus-spike-review-2026-11-15.md`, `…rfc-026-result-layout-review-2026-11-18.md`.

Owner philosophy: clean, safe and secure, robust design; users must not be confused; less is more.

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit 0 / 12); CI waited for; no tags, force-push or history rewriting.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Phase 0: does `snora::checkbox` / `snora::radio_group` (engine path) satisfy RFC 021's principles on our five places, in both themes and both languages? | headless snapshots (`iced_test`, CPU renderer, hermetic fonts) | pictures of the prototype **and** of today's UI, same state, same size; only the worst case plus one baseline per place; your description of every difference |
| Layout properties (RFC 035): hit areas ≥ 24 px, no overflow, nothing squeezed | headless matrix | the existing harness on the prototype (P1, P1b, P2, P3); the 26 `hit_area.rs::KNOWN` entries for checkboxes shrink as rows are replaced |
| One message per action: a click, and Enter/Space (checkbox) or an arrow (radio group), each publish the message the mouse path published before | logic/headless | tests per control kind: the message equals today's; no double message |
| Keyboard reach of the new controls | headless | Tab lands on each checkbox and on each group once; the arrows move and select |
| The look of the real window | real UI, **one** capture at the end of the build (worst case) | native Wayland via `scripts/gui-session.sh` |

## 1. Phase 0 — the look study (no commit; stop and report)

Prototype in the working tree (patches in your review-request folder, then revert) the five places, using `snora::checkbox` and `snora::radio_group` on the **engine path** (do not enable snora's `design` feature) with `LayoutDirection::Ltr`:

1. **Settings page:** language (3 options) and theme (3 options) as `radio_group`.
2. **Result screen preview panel:** "View as" and "Surface" as `radio_group` (look at `ui/preview_panel/pickers.rs`; they may be horizontal today: say what the group's orientation options are and what the narrowest window does).
3. **Customize, PNG block:** the six preset checkboxes and "Keep transparency" as `snora::checkbox` (label inside the target); the custom-size field and Add stay as they are.
4. **Customize, Advanced:** the option checkboxes (apple-touch, snippet, SVG, vectorize, manifest, Microsoft logos, monochrome) as `snora::checkbox`; the resize-algorithm and trace-preset choices as `radio_group` (count the options and show whether they fit at 360 × 640; if a group is too long, say so).
5. **The SVG quality control:** three labelled choices as `radio_group` (RFC 034 D5; the labels: reuse the current slider's end labels and the middle one, say which strings).

Judge each against RFC 021 principles 1–4 and the amendments: the selected cue is shape plus accent (not colour alone, not a font glyph), the accent means selection only, the focus ring (neutral, 2 px, offset) does not merge with it, contrast in both themes, Japanese strings (longest) do not squeeze, the group is one Tab stop. Also report: how the engine path **sizes** the controls (ems of our text: what pixel sizes result at our 13 px body?), how a long label wraps, whether `snora::checkbox` accepts our existing label strings unchanged, what `radio_group` needs for our value types, and any limitation you find. **Do not change `Cargo.toml` features.**

Deliverable of phase 0: `.git-exclude/review-request/rfc-021-selection-cues/README.md` with the pictures (before / prototype, per place), the properties' results, the table of principles per place, **a recommendation** (adopt as is / adopt with these changes / do not adopt a given control and why), and the list of things you could not decide. **Stop there; the architect reviews and then confirms the build (step 2).** Nothing committed in phase 0.

## 2. Step 2 — the build (after the architect's confirmation; commits as the review says)

Scope (RFC 021): the selection cues of the five places as confirmed; the **navigation cue** (Q1: filled background, **bold label and a thin accent bar on the edge facing the content**, desktop side navigation and mobile bottom navigation, built on today's buttons); `✓` reserved for success/ready, `⚠` for warnings/errors, busy by a lucide `Loader` instead of the `⏳` text glyph (`ui/accessibility.rs` `marker`; an explicit `SELECTED`-style helper is no longer needed where snora's controls draw the cue; remove what becomes unused); docs (`ui-a11y.md`, README, `functional-spec.md`, development instructions) describe the shipped design, the `▣` mentions go; RFC 018 syncs the rest.

Tests: one meaning per cue (helpers return distinct elements); messages equal to today's; headless P1–P3 on the whole matrix with the `KNOWN` list shrunk to what remains; Tab/arrow reach of every new control; the old font-glyph markers are gone (a source test that the selected cue uses no text glyph).

## 3. Acceptance criteria

Phase 0: the report above, nothing committed. Build: no glyph carries two meanings; no state cue depends on font fallback; the five places use snora's controls (or the confirmed exceptions); every new control is a Tab stop; messages unchanged; layout properties green; docs and UI agree; gates and CI green on every pushed commit; one real capture of the worst case.

## 4. Prohibited shortcuts

No `design` feature, no new package; no change to messages or state; no keyboard code of our own for these controls (snora's is the one owner; `shortcuts.rs` unchanged); no threshold change, no `#[ignore]`, no new `KNOWN` entry; no `snora::select` or `snora::slider` (decided against for now, RFC 027 amendment); no new `#[allow]`; no tags, force-push or history rewriting.
