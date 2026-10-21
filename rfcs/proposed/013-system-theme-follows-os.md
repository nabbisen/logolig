# RFC 013 — "System" theme follows the operating system

**Status.** Proposed
**Priority.** P2
**Tracks.** Defect: `ThemeMode::System` always renders the Light theme.
**Touches.** `crates/app/src/app.rs` (`resolve_theme`, `subscription`, `AppState`), preview Surface "System", docs.

## Summary

The Settings page offers Light / Dark / System, and `ThemeMode` documents that `System` "follows the OS colour scheme". The implementation maps `System` to `Theme::Light` and never asks the OS. Users on dark desktops who keep the default get a light window.

## Evidence

- Verified (source): `app::resolve_theme` — `ThemeMode::System | ThemeMode::Light => Theme::Light`, with a comment "will query OS theme in a future step".
- Verified (dependency source, iced 0.14.0): `iced_runtime::system::theme() -> Task<theme::Mode>` and `system::theme_changes() -> Subscription<theme::Mode>` exist; `theme::Mode` is `None | Light | Dark`.
- Not verified: whether those APIs return a value on every target (Linux compositors without the XDG settings portal, Windows, macOS). Needs a spike.
- Per snora's 0.53 migration guide: on Linux iced reads the desktop colour scheme over D-Bus through its default `linux-theme-detection` feature. This project builds iced with default features, so the detection code is already linked; it is also the source of three "unmaintained" advisories (RFC 024, RFC 028). Keep the feature on.
- The preview Surface picker also has a "System" option that is resolved through the same function.

## Design

1. `AppState.system_theme: theme::Mode` (default `None`).
2. At boot, run `iced::system::theme()` as a `Task`; subscribe to `theme_changes()` for live updates. Both map to `Message::SystemThemeChanged(Mode)`.
3. `resolve_theme`: `Light` → Light, `Dark` → Dark, `System` → by `system_theme`; `Mode::None` (unknown) falls back to **Light**, the current behaviour, so no platform regresses.
4. The Settings page states the resolved theme next to "System" (text, not only colour), for example "System (currently Dark)".
5. The preview "System" surface uses the same resolved value.

## Open question

- **Q1** Fallback when the OS cannot report a theme: Light (current) vs Dark? Recommendation: Light — no behaviour change.

## Tests (design-derived)

- `update()` tests with `SystemThemeChanged(Dark)`: `System` resolves to Dark; explicit `Light` stays Light.
- `Mode::None` resolves to Light.
- Manual matrix recorded as evidence: GNOME dark, KDE dark, niri/Wayland without portal, Windows dark, macOS dark (platforms available to the team; others marked untested).

## Acceptance criteria

On at least one platform where iced reports a mode, switching the OS theme changes the app while `System` is selected; all other platforms behave as today.

## Risk

Low. Additive; fallback preserves current behaviour.
