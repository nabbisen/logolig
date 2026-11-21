# RFC 036 — Upgrade snora 0.57 → 0.62 (the keyboard `control`)

**Status.** Implemented (1.28.0). Earlier: Accepted (architect, 2026-11-15; announced to the owner; release 1.28.0). Dependency upgrades are the architect's remit (as RFC 028, 032); the owner may veto.
**Priority.** P1 (prerequisite of the keyboard work, RFC 027).
**Tracks.** The RFC 027 phase 0 spike (J13) found that snora ≥ 0.59 exports `snora::control(content)`: the behaviour of iced's `button` with our own styles unchanged, plus a focus target in layout order, Enter/Space, a high-contrast offset ring, a ≥ 24 px target and an `id`. That makes iced's own focus the single focus owner and removes the need for an application-owned focus engine.
**Touches.** `Cargo.toml` (one version), `Cargo.lock`, release notes. No code change expected.

## Evidence (architect's trial, 2026-11-15, scratch copy of `main` at `5cccf0c`, not the repository)

- `snora = "0.62"` and nothing else changed: `cargo check --workspace --all-targets --all-features --offline` **succeeds with no code change**; the whole test suite (including the headless layout tests of RFC 035) **passes**; the lock has **587 packages before and after** (only the four `snora*` versions change: `snora`, `snora-core`, `snora-interaction`, `snora-widgets`); `cargo audit`: **0 vulnerabilities, 11 allowed warnings** (unchanged).
- Not measured by the trial: visual differences (toasts, the side bar, snora's own widgets) and runtime behaviour in a real window. Our UI uses snora for layout, toasts and the Lucide icon font only; the toast API in use (`Holds`, `ToastAttention`) compiled unchanged.
- snora 0.60 removed `dismiss_on_escape` (not used by the application).

## Requirements

1. `snora` "0.57" → "0.62" in the workspace `Cargo.toml`; `Cargo.lock` changes limited to the `snora*` packages (and anything they force, listed). No general `cargo update`.
2. Gates and CI as usual; MSRV 1.90; `cargo audit` stays 0 / 11.
3. **Visual equivalence proved at the cheapest level:** headless snapshots (`iced_test`, the RFC 035 harness) of Home, Converting, Result (7 cards), Customize (Advanced open) and Settings in Light and Dark at 1280 × 720, taken with 0.57 and with 0.62, compared **byte for byte**; every difference is described and judged. One real-window pass on the release commit (the toast appears, is readable, is dismissed) is part of the release checklist, not of this job.
4. Release notes: one line (internal; no user-visible change expected).

## Not in scope

Using `snora::control` (RFC 027 steps), `focus_zone`, menus, `Digits` shortcuts (all deliberately not enabled).

## Risk

Low: zero compile changes and a green suite on the trial; the unmeasured part is visual and is covered by requirement 3. Rollback is one line.

## Amendment 2026-11-18: the target becomes 0.65 (second step, job J13c)

snora 0.65.0 (released 2026-11-18; the team's letters are in `.git-exclude/upstream/snora/receive/0.65.0/`) adds **a keyboard-reachable checkbox, toggler, radio group, select and slider** (`snora::{checkbox, toggler, radio_group, select, slider}`, RFC-134), which is exactly what RFC 027's Customize and Settings steps were going to build by hand. J13a (0.57 → 0.62) is done; **J13c takes 0.62 → 0.65 in one line**, with the same proofs.

Architect's trial (scratch copy of `main` at `dfeea80`, `snora = "0.65"` only): `cargo check --workspace --all-targets --all-features` compiles **with no code change**; the whole suite (311 tests) passes; 587 packages before and after, only the four `snora*` versions change; `cargo audit` 0 vulnerabilities, 12 allowed warnings (the baseline). The 0.63 and 0.64 breaking changes (`row_dir` returning `LogicalRow`; the removed F6 types; eight widget functions taking a required `direction`) do not touch the functions we use. Contrast, typography (`title`) and right-to-left corrections in the letters concern us only as improvements: we do not redefine `title`, and the application is left-to-right (`LayoutDirection::Ltr` wherever a snora form control is used).
