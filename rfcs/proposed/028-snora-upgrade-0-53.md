# RFC 028 — Upgrade snora from 0.25.2 to 0.53.0

**Status.** Proposed
**Priority.** P1 if the owner authorizes it into release 1.27.0 (architect's recommendation); otherwise P2
**Tracks.** Dependency currency, UI correctness fixes, and the basis for the keyboard work (RFC 019 phase 2, RFC 027).
**Touches.** workspace `Cargo.toml` (`snora`), `Cargo.lock`, possibly a few lines in `crates/app`, `docs/assets/screenshot-01.png`, README screenshot, docs that describe appearance.

## Summary

The project pins snora 0.25.2; the current release is 0.53.0. snora's upstream sent migration guides for every step (received 2026-10-21, `.git-exclude/upstream/snora/receive/0.53.0/…/README.md` and `guides/`). The jump is mostly additive, but it brings bug fixes and keyboard support we need, and its default appearance has changed several times, so doing it **before** the first public release avoids invalidating published screenshots later.

## Evidence

Source: snora's own guides (read by the architect, 2026-10-21). **Not yet verified by building the project against 0.53.0**; that is the first task of the handoff.

- **API compatibility.** snora states the upgrade is "mostly additive"; "code changes, if any, are a few lines, and the compiler points at each one". Four steps *may* break a build: 0.33 (removed `snora_widgets::design::{style, theme}`), 0.40 (`lucide-icons` no longer enables iced's `advanced` feature), 0.42 (snora no longer enables iced's `canvas` and `tokio` features), 0.45 (removed `snora_design::{Emphasis, Size}`).
- **None applies to this project (verified by `grep` of `crates/`):** the app imports only `snora::{AppLayout, LayoutDirection, render, Toast, ToastIntent, ToastLifetime, Icon, lucide, widget::icon_element_sized, toast::{subscription, sweep_expired}}`; it does not use `snora::design`, `iced::advanced`, canvas, or the removed types; the workspace enables iced's `tokio` feature itself.
- **Rust version.** snora 0.53.0 requires Rust 1.88 (0.25.2 declared none). RFC 024 sets the project minimum to 1.90, so there is no conflict. iced stays 0.14.
- **Behaviour fix (0.41).** Overlays now contain pointer input: a click on a dialog's own content no longer dismisses it, and a click on a toast body no longer reaches the widget underneath. The old behaviour was a bug (snora's wording); there is no flag to restore it. Logolig shows toasts, so this closes a real input-integrity defect for our users.
- **Accessibility fixes on the default path (0.42 and later):** toast text colours that failed WCAG AA now pass; the modal dim follows the theme (0.53); sidebar buttons, tab bar, label sizes follow-ups (0.50 to 0.52). These change the **default look**; snora says to retake screenshot baselines.
- **Keyboard (0.35, opt-in).** `snora::keyboard::cycle_zones` and `snora::focus::{Cycle, FocusZone, ZonePresence, next_zone}` provide **F6 zone navigation** (move between regions such as side navigation, body, footer). snora is explicit about limits: no widget-level focus, no accessibility tree (iced 0.14 has none), focus trapping "staged, not shipped", keyboard dismissal is application-owned. So RFC 027's app-owned focus model is still required; F6 zones fit it (RFC 027 is amended accordingly).
- **Advisories.** RUSTSEC-2026-0370, -0371, -0372 mark `zvariant`, `zbus_names`, `zvariant_derive` as *unmaintained* (not vulnerabilities). They are already among the 14 allowed warnings in this project's audit. They reach the graph on Linux through iced's default `linux-theme-detection` feature (reads the desktop colour scheme over D-Bus). No upgrade clears them while iced uses `zbus` 5. This feature is what RFC 013 needs for "System" theme, so it stays on.
- **snora's published guarantee was corrected (0.53 §4).** The old sentence "output is unchanged from v0.25 without `design`" had not been true since 0.42. A search of this repository finds no citation of it. The surviving guarantee is that snora's *composition* (layer order, dismissal, mirroring) is unchanged without `design`.
- **Screenshots.** The only committed screenshot is `docs/assets/screenshot-01.png` (README). There are no automated visual baselines.

## Design

1. Change `snora = { version = "0.25", ... }` to `0.53` with the **same** features (`default-features = false`, `["widgets", "lucide-icons"]`); do **not** enable `design` (DEC-004 unchanged: adopting snora's design system is a separate decision and is not needed for this upgrade).
2. `cargo update -p snora` only; review every lock change (snora-core, snora-widgets and new snora crates); confirm iced is still 0.14.x and report all other crates that moved.
3. Fix only what the compiler names (expected: nothing).
4. Verify behaviour: toasts (success, warning, error, dismissal, auto-expiry), a click on a toast body does not trigger the page beneath, nav side bar and bottom bar, theme Light/Dark, language switch, Result and Customize screens.
5. Retake the README screenshot; update any document that describes the old look.
6. Run `cargo audit`: expect no new vulnerability; report the warning delta.
7. Out of scope: using F6 (RFC 019 phase 2 and RFC 027), adopting `design`, changing any layout.

## Tests and evidence

Gates (`fmt`, `clippy -D warnings`, `test`, MSRV, `audit`), all CI jobs green; before/after screenshots at 1280 × 720 for Home, Result, Customize, Settings in Light and Dark; a manual toast click-through check; the list of changed crates in the lock; and a note on every visible difference.

## Acceptance criteria

The application builds and passes all gates on snora 0.53.x; no behavioural regression (screens, toasts, themes, languages); visible differences are listed and acceptable to the owner; README screenshot current; audit shows no new vulnerability.

## Decisions requested

- **D1 — schedule (owner authorizes the scope change).** Recommendation: add this job to release 1.27.0, right after RFC 025 and before the SVG and keyboard jobs. Reasons: the first public audience gets the overlay-click fix and AA-contrast toasts; the keyboard work can use F6 zones; screenshots are not published yet, so retaking them costs nothing later; the jump is small today and grows with every release. Cost: one small job (S–M) and a visible, mostly cosmetic change of look. Alternative: 1.28.0 (then keyboard phase 2 cannot use F6, and public screenshots go stale).
- **D2 — `design` feature: not adopted now.** It changes more, needs UI migration, and belongs with RFC 021 (selection indicators) if at all; revisit then.

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| Visible change of default look | Low-Medium | Before/after screenshots; owner reviews; no `design` feature |
| Behaviour change of overlays | Low | It fixes a bug; the toast/dismissal manual check proves it |
| Larger dependency graph / licences | Low | Report the lock diff and licences |
| Scope growth of 1.27.0 | Low | One small job; owner authorizes it first |
