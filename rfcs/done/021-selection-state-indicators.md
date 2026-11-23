# RFC 021 — Selection-state indicators

**Status.** Implemented (1.28.0). Earlier: Accepted (owner authorization 2026-11-09, release 1.28.0)
**Priority.** P2
**Tracks.** UI/UX: one glyph is used for different meanings, and the documented glyph is not the implemented one. Raised by RFC 018 D1.
**Touches.** `crates/app/src/ui/accessibility.rs` (`marker`), `ui/preview_panel.rs`, `ui/settings_page.rs`, `ui/sidebar.rs`, `ui/result_view.rs`, `docs/src/ui-a11y.md`, tests.

## Summary

The product rule is that state is never conveyed by colour alone. The rule is right, but the current marker choice can confuse: the same `✓` means "this option is selected" in pickers and "success / ready" on the Result screen, and the documentation promises a different glyph (`▣`) that does not exist in the code. This RFC defines which cue means what and removes the dependence on font glyphs.

## Evidence

Verified by source reading; **not verified visually on all platforms**.

- `ui/preview_panel.rs` marks the active "View as" / "Surface" button with `marker::READY` (`✓`), i.e. the *Ready* marker is reused for *selected*.
- `ui/settings_page.rs` renders `✓` (or two spaces) for the current language/theme.
- `ui/result_view.rs` shows `✓` as the success mark, and `converting.rs` uses `⏳` (`marker::BUSY`).
- `▣` appears only in a source comment, yet README, `ui-a11y.md`, `functional-spec.md` and the development instructions describe it as the active-state marker.
- All of these are Unicode characters drawn with whatever font the text renderer falls back to. The application bundles the lucide icon font, but not a text font covering these symbols. Whether `✓`, `⏳` and `▣` render everywhere (including minimal Linux installs and Windows) has not been checked.
- Convention: a check mark on a list row means "current choice" in menus, but on a button it reads as "done/OK"; radio-style marks mean "exactly one of these".

## Principles

1. **One meaning per cue.** Success/ready, busy, error, and *selected* each have a distinct cue.
2. **Never colour alone**, and preferably a *shape* difference, not only a different symbol.
3. **Font-independent.** Use bundled assets (lucide icons through snora, already registered at start-up) rather than text glyphs.
4. **Familiar semantics.** Single choice among several (View as, Surface, language, theme) looks like a radio group; independent options keep real checkboxes/toggles.

## Proposal

- Single-choice groups: lucide `Circle` (unselected) and `CircleDot` (selected) before the label, plus the existing filled background for the selected item. Exact icon names are to be confirmed against `snora::lucide`.
- Navigation items (Home / Customize / Settings): the active item gets a filled background, a bold label and a thin accent bar on the edge facing the content (shape cue), instead of a check mark.
- `✓` is reserved for success/ready; `⚠` for warnings/errors; busy keeps a cue that does not depend on an emoji glyph (candidate: lucide `Loader`).
- Replace `marker::READY` reuse with an explicit `SELECTED` helper that returns an icon element, so the compiler shows every call site.
- Documentation (`ui-a11y.md`, README) describes the shipped design, and RFC 018 row 1 closes with it.

## Alternatives

- Keep `✓` and fix the docs: cheapest, but keeps the double meaning and the font dependency.
- Adopt `▣` as documented: distinct, but it has no established meaning for users, so it must be learned, and it has the same font dependency.
- Wait for the Snora Design System (DEC-004), which brings tokens and primitives: may supersede the hand-made styles, but should not block a correctness-of-meaning fix.

## Decision (owner, 2026-11-09)

- **Q1 decided: the active navigation item gets the accent bar plus a bold label** (the architect's recommendation, accepted). A bar is a shape cue, not colour alone. It applies to the desktop side navigation and the mobile bottom navigation alike.

## Tests

- Unit: the indicator helper returns different elements for selected and unselected, and neither equals the success marker.
- Manual checklist as evidence: light and dark themes, a colour-blindness simulation, 1280 × 720 and mobile width, Linux (Wayland) and Windows.

## Acceptance criteria

No glyph carries two meanings; no state cue is a plain text symbol that depends on font fallback; docs and UI agree.

## Risk

Low. Visual only; no logic change.

## Amendment 2026-11-15 (from the RFC 027 spike): the focus ring is not the selection colour

A focus ring drawn in the accent colour merges with an accent-filled, checked box (measured on headless snapshots). Rule: **the accent colour (fill, ✓) means "selected / active" and nothing else; keyboard focus is a neutral, high-contrast ring, 2 px, offset 2 px outside the control** (RFC 027). The radio-like groups defined here are also the replacement for the two pick lists and, with RFC 034 D5, for the SVG quality slider (RFC 027): each option is a keyboard-reachable row with the one selection cue. J10 builds the cues; J14 (Customize step) swaps the pick lists and the slider for them.


## Amendment 2026-11-18 (snora 0.65): the cues are built on snora's form controls

snora 0.65.0 ships `snora::checkbox`, `snora::radio_group` (and `toggler`, `select`, `slider`), each keyboard-reachable with a ring and a 24 px target (RFC 027 amendment of the same day). This RFC's principle 4 ("single choice looks like a radio group; independent options keep real checkboxes") is what they are. Decision: **J10 builds the selection cues on them**, replacing the hand-made rows (`✓` and two-space markers in `settings_page.rs`, the marked buttons of `preview_panel.rs`, the iced checkboxes and pick lists of `advanced_drawer.rs`). J10 starts with a **short look phase** (the handoff): prototype the five places, judge them against the principles above (one meaning per cue; never colour alone; no font-glyph dependence; accent = selected), the layout properties of RFC 035 (24 px; both themes; Japanese) and "one message per action"; the architect reviews the pictures, then the build follows. The navigation cue (Q1: accent bar plus bold label) stays our own and is built in J10 on today's buttons; J14 makes the items `snora::control` without changing how they look.

## Decisions after the phase 0 look study (architect, 2026-11-19; the owner's philosophy: users must not be confused, less is more)

Study: `.git-exclude/review-request/rfc-021-selection-cues/` (reviewed in `.git-exclude/reviewed/rfc-021-selection-cues-phase0-review-2026-11-19.md`). Adopted: `snora::checkbox` and `snora::radio_group` on the engine path, as prototyped.

- **D1 The selection cue is neutral: shape plus a high-contrast black/white fill, not our blue.** snora's engine path draws the mark in the theme's accent only when it holds 3:1 against its hover tints, and on our two themes it does not (2.0 to 2.8:1), so the mark is black in Light and white in Dark. Accepted as it is: it is legible (21:1 / ≥ 11:1), never colour alone, and the stronger contrast serves RFC 034. The amendment of 2026-11-15 is refined: **the accent colour no longer means "selected" for form controls; it is for the primary action and the active page (the navigation bar of Q1)**; keyboard focus remains a neutral ring, separated from the mark by the control's padding (and verified per theme by a ring snapshot). No custom theme (a colour change that would touch every button) and no `design` feature.
- **D2 Single-choice groups are `radio_group`, vertical** (the group has no orientation setting): Settings language and theme; the preview panel's "View as" and "Surface"; the resize algorithm (5 choices); the SVG conversion choice (D4). The label sits **above** the group, always (simplest, and it keeps narrow windows from pushing the group into a narrow column). The disabled Surface group (Checker context) is dimmed as a whole, as snora draws it; this fixes today's misleading look (a disabled group that looks active). The taller preview panel (about 150 px) is accepted: it replaces a layout that already breaks at 360 px in Japanese.
- **D3 Independent options are `snora::checkbox`** (the 6 PNG presets, "Keep transparency", and the seven option checkboxes of Advanced). **The labels are drawn at the host's 16 px** (the controls have no size setter; our own labels are 13 px). Accepted: 16 px is the body size RFC 034 D1 will make uniform, the controls are what the user acts on, and it avoids changing every unsized text now. The application's default text size is **not** changed in this job. Customize gets taller (+45 %); it scrolls already.
- **D4 One control for the SVG conversion setting.** The slider (Simple · middle · Detailed) and the nested "Preset" pick list under "Output favicon.svg" both write `vtracer_preset` under different names (an accident of history: the slider maps Sharp to "Simple" and PhotoRich to "Detailed"). **Keep one: a `radio_group` of the three presets by their descriptive names, "Sharp (logos, icons)", "Default (balanced)", "Photo-rich (gradients)"** (existing strings, English and Japanese), in the main page's "SVG conversion" section where the slider is; **remove the nested pick list** from Advanced. Stored values do not change. The now unused `svg_conversion_simple`/`svg_conversion_detailed` (and `PresetLabel` if unused) **stay in the public `MessageKey`** (SemVer: nothing public is removed in 1.x): they are marked `#[deprecated(since = "1.28.0", …)]` with an item-level `#[allow(deprecated)]` on the dictionary match and a comment, removed at the next major. RFC 034 D5 ("labelled choices") is fulfilled by this.
- **D5 `snora::select` and `snora::slider` stay unused** (unchanged).
- **D6 Radio groups select as they move** (↑ ↓ ← →, wrapping; snora's rule). Choosing the language by arrow therefore changes the language at once, as in every desktop toolkit; no confirmation step.
- **D7 The build also**: builds the navigation cue (Q1); reserves `✓` for success/ready and `⚠` for warnings and errors; replaces the busy text glyph `⏳` (renders as a boxed "?" where the machine has no font for it, shown in the study) by a lucide `Loader`; removes what becomes unused in `ui/accessibility.rs`; updates `ui-a11y.md`, the README, `functional-spec.md` and the development instructions to the shipped design (the `▣` mentions go).
- **D8 Test follow-ups are part of the job** (the study found them): focus the custom-size field **by identity** in the J13b tests (three fail otherwise, and the completeness test would silently stop testing each field); add the **open preview panel** to the headless matrix; add the **stale-entry check** to `hit_area.rs` (as `overflow.rs` has); a ring snapshot per theme.

## Status after the J10 build (architect, 2026-11-19)

Built and reviewed (`21567ef`..`4a23e6d`): D1–D8 as decided; the navigation cue; `✓`/`⏳` replaced by lucide icons; one SVG preset control; the accessibility markers module removed. **One item of the acceptance criteria is still open and is assigned, not forgotten:** the disclosure triangles `▶`/`▼` of "Advanced settings" and of the result Preview toggle are open/closed state cues drawn as text glyphs. They become lucide chevrons in the RFC 027 job J14b (Customize and Result steps), where those headers become `snora::control`s anyway, and the glyph source test is extended to them then. RFC 021 moves to `done/` only with that step.

## Amendment 2026-11-24 (architect, from the J21 contrast job): the selection mark is the accent, in both themes

D1 (2026-11-19) accepted a **neutral** (black/white) selection mark because snora's engine path draws the mark in the theme's `primary.strong` only when that colour holds 3:1 against the page and its hover tints, and on the 1.28.0 palette it did not. That was an acceptance of a constraint, not a principle. The J21 palette (RFC 034) changed the constraint: on Light the marks are now a blue (`#4E54FF`, 5.2:1), on Dark still white. Two things are wrong with leaving it: **the themes disagree** (blue on Light, white on Dark) and **there are two different blues on one page** (the mark `#4E54FF` and the primary `#3C47B8`). Decision: **the selected mark (a checked box, a selected option) is the application's accent colour in both themes, the same colour as the primary fill of that theme**, obtained the way snora documents it (the theme's `primary.strong`, held at 3:1 against the page and the hover tints), defined once in `ui/theme.rs`. The accent therefore means: **the primary action, the active page and the selected state**; the keyboard focus ring stays a neutral, high-contrast outline (RFC 027), which is why the accent no longer merges with it. The tick on the mark reaches 7:1 against its fill. One meaning per cue is kept: a filled mark means "selected", the ring means "focused", a bar means "this page", a chip with fill means "do this".

