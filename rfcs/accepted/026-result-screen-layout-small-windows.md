# RFC 026 — Result screen layout at small window heights

**Status.** Accepted (owner authorization 2026-11-09, release 1.28.0)
**Priority.** P2
**Tracks.** The "← Back" button is clipped at the bottom of the Result screen.
**Touches.** `crates/app/src/ui/result_view.rs`, possibly `shell.rs`.

## Summary

With seven or more result cards the content is taller than a 1280 × 720 window. The "Download all (ZIP)" button is visible, but the "← Back" control below it is cut off, so a mouse user cannot reliably use it without a scroll that is not obvious.

## Evidence

- Observed by the RFC 010 implementer and **confirmed by the architect** on the stored screenshot (`02-result-cards.png`, 1280 × 720, default plan with seven cards): a partial rounded rectangle is visible at the very bottom-left, below "Download all (ZIP)", clipped by the window edge. A scrollbar is present on the right edge.
- Reported as pre-existing and unrelated to RFC 010 / RFC 009.
- Not verified: behaviour with the Customize options that add more cards (monochrome, Microsoft logos, manifest: the RFC 009 all-options run produced 16 cards) and on the mobile layout; whether the whole page scrolls or only the card grid.
- Related: the "← Back" control is the only way to return to the drop zone (keyboard support is absent, RFC 019), so a clipped Back is a navigation problem, not a cosmetic one.

## Design (to be refined after a short investigation)

1. Reproduce at 1280 × 720 and 1024 × 600 with the default plan and with every option on; record where scrolling happens.
2. Move the primary navigation ("← Back") to a position that is always visible (for example a header row above the grid), keeping "Download all (ZIP)" as the primary action; the exact layout follows "less is more" and is checked against the mock conventions in the docs.
3. Ensure the whole result page scrolls as one, so nothing is unreachable at any height.

## Acceptance criteria

At 1280 × 720 and at the mobile width, with every output option enabled, "← Back" and "Download all (ZIP)" are fully visible or reachable by an obvious scroll; screenshots recorded as evidence.

## Risk

Low. Layout only. Coordinate with RFC 021 (indicators) and RFC 019 (keyboard) so the same screen is not reworked three times.

## Amendment 2026-11-12 (from the JH review): the headless level measured it, and found a second page

The headless layout tests of RFC 035 (job JH) measured the defect on all seven matrix sizes (1280 × 720 down to 360 × 640, both languages, 7 or 16 cards): "← Back" and "Download all (ZIP)" are **not fully visible on the Result screen at any size** (on the 1280 × 720 reproduction: window height 720, "← Back" at y = 835). They also found a defect nobody had listed:

- **The Settings page is not scrollable and `iced` squeezes its last rows to zero width when the window is short or narrow:** at 1024 × 600 and 800 × 600 the "Show this list" row and the Tab-key note vanish; at 768 × 480 the five shortcut descriptions; at 767 and 640 × 480 the heading "Keyboard shortcuts" too; at 360 × 640 the last row and the note (English only; the Japanese strings fit). In the usual window (1042 × 1144) the list is complete.

Decision (same principle as above, "the whole page scrolls as one, so nothing is unreachable at any height"): **every page** (Home, Result, Customize, Settings) scrolls as one when its content is taller than the window, or is laid out so that nothing is squeezed out; the shortcut list on Settings is covered by the same rule. Acceptance additions: the entries "Back" and "Download all (ZIP)" are added to property P2 of `ui_layout_tests/reachable.rs` (asserted, red-first) together with the fix; the Settings entries of `overflow.rs::KNOWN` (26 entries today) are deleted; the headless matrix is green without them. RFC 021 and RFC 034 keep their own jobs; this one fixes layout reachability only.
