# RFC 021 — Selection-state indicators

**Status.** Proposed
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

## Questions for the owner

- **Q1** Is the accent-bar treatment for navigation acceptable visually, or do you prefer a simpler fill-only treatment with bold label? (A fill plus bold is not colour alone, because weight and fill change shape and contrast, but the bar is the stronger cue.)

## Tests

- Unit: the indicator helper returns different elements for selected and unselected, and neither equals the success marker.
- Manual checklist as evidence: light and dark themes, a colour-blindness simulation, 1280 × 720 and mobile width, Linux (Wayland) and Windows.

## Acceptance criteria

No glyph carries two meanings; no state cue is a plain text symbol that depends on font fallback; docs and UI agree.

## Risk

Low. Visual only; no logic change.
