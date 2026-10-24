# RFC 029 — Toast lifetime proportional to its content

**Status.** Proposed
**Priority.** P1 if the owner authorizes it into release 1.27.0 (architect's recommendation); otherwise P2
**Tracks.** UX defect: advisory toasts disappear before they can be read.
**Touches.** `crates/app/src/app.rs` (`push_warning_toast` and callers), tests, `docs/src/functional-spec.md` §10.

## Summary

Every warning toast uses snora's default lifetime of **4 seconds**, while the application's advisory toasts contain 30 to 60 words. A user cannot read them, and an advisory that cannot be read is a message that did not arrive: it teaches the user nothing, which is the "users must not be confused or misunderstand" rule the owner set.

## Evidence

- Verified (source): `push_warning_toast` builds `Toast::new(..)` with no `with_lifetime`; `ToastLifetime::DEFAULT` is `Transient(4 s)` (`snora-core` `toast.rs:164`, unchanged in 0.53; the application sets 7 s explicitly only for the long success toast).
- Verified (GUI picture, RFC 015 phase 2a review, `svg-d-both.png`): the combined SVG advisory fills **nine lines** and covers a result card; the dev measured that the toast is gone about 4 s after it appears.
- Computed (architect, 200 words per minute, an ordinary adult reading rate, to be taken as an estimate): SVG combined advisory 57 words ≈ 17 s; JPEG advisory (shipping since v1.11) 41 words ≈ 12 s; non-square advisory 31 words ≈ 9 s. All three are shown for 4 s.
- Not verified: how long real users need; whether users in the target group read the toast at all while the Result screen loads (the toast appears during the "Importing…" state).
- Related: the toast overlaps cards at 1280 × 720 (RFC 026).

## Design

1. **Lifetime by content, not a constant.** Transient toasts get `max(4 s, 3 s + 0.35 s × words)`, capped at 20 s; the word count is taken from the displayed title plus body (a Unicode-aware count: for Japanese, which has no spaces, count characters ÷ 4 as the estimate of "words"). One helper, one place, covered by table tests. Example: 9 s for the non-square advisory, 12.5 s JPEG, 20 s (cap) SVG combined.
2. **Hover/focus does not extend it** in this RFC (snora 0.53 has no such hook; do not invent one). Every toast keeps its `×`.
3. **Alternative considered: persistent advisories** (dismissed by the user, as errors are). Rejected as the default: stacking several persistent advisories clutters the screen ("less is more") and they never go away if ignored; revisit if user feedback says transient is not enough.
4. Errors stay persistent; the 7 s success toast becomes the same formula (a path of eight words reads in 6 s, so it stays about the same).
5. Documentation: `functional-spec.md` §10 states that toast time scales with length.

## Tests (design-derived)

- Table tests of the helper: 0 words, 10, 31, 41, 57 words, a long text beyond the cap, a Japanese string.
- `update()` tests: each advisory toast pushed by the app carries a lifetime of at least 3 s + 0.35 s per word of its own text (parameterised over the existing advisory kinds).
- Manual: one capture of each advisory showing it still visible at 8 s (state: the toast list is non-empty; no frame loops).

## Acceptance criteria

No advisory toast expires before a typical reader can finish it; the formula is in one function with tests; every toast still has a working `×`.

## Decisions requested

- **D1** Add this as a small job **J3b** to release 1.27.0, before the keyboard job? The architect recommends yes: the SVG, JPEG and non-square advisories are part of 1.27.0 and are only useful if they can be read; the change is one helper and tests (size S). It is a scope addition and therefore needs the owner's authorization.
- **D2** The formula and the 20 s cap are the architect's proposal; the owner may prefer other numbers.

## Risk

Low. A longer-lived toast covers content for longer (RFC 026 addresses placement); the `×` mitigates it.
