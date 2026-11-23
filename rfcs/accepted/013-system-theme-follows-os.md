# RFC 013 — "System" theme follows the operating system

**Status.** Accepted (architect, under the owner's delegation of 2026-11-22 "review them carefully with my fundamental philosophy"; release 1.29.0; decisions in the section "Decisions 2026-11-22")
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

## Decisions 2026-11-22 (architect; the owner's philosophy)

- **Q1 decided: fall back to Light** when the system cannot report a theme (no behaviour change on those systems).
- **Honesty about the fallback:** Settings shows "System (currently Dark)" or "System (currently Light)" when the system reported it, and **"System (not reported; using Light)"** when it did not, so a user is never led to believe their system is light when the program merely does not know (text, not only colour).
- **Spike first, evidence only:** on this machine (niri, native Wayland and XWayland) record what `iced::system::theme()` returns and whether `theme_changes()` fires when the desktop setting is switched (a private `dconf`/portal setting is not touched on the owner's real session: use the isolated session only for reading; if switching the real setting is the only way to test, ask the owner in words). The result decides the handoff; "documented, not detected on this platform" is an allowed outcome for a platform where the API stays silent.
- A **live change** while the application runs repaints the theme (no restart); the preview's "System" surface uses the same resolved value (one function).

## Result of the spike and the design it fixes (architect, 2026-11-23)

Spike: `.git-exclude/review-request/rfc-013-system-theme-spike/` (reviewed in `.git-exclude/reviewed/rfc-013-system-theme-spike-review-2026-11-23.md`). Measured on this machine with a private D-Bus bus and a stub of `org.freedesktop.portal.Settings` (the owner's real settings were never touched; the real portal's answer was read once: dark preferred): with no service the application gets `Mode::None`; with a service `1` → Dark, `2` → Light, `0` → None; `theme_changes()` fires on every `SettingChanged` within 0 to 3 ms and the page repaints; native Wayland and X11 behave alike. iced learns the theme through `mundy` (portal `org.freedesktop.appearance` / `color-scheme`; `1` dark, `2` light, `0`, any other number **or any error** → `Mode::None`). **`Mode::None` therefore means both "no preference" and "nothing answered".** At start the application is told `None` first (winit cannot report a theme on Wayland or X11, so iced's window-creation step resets it) and the portal's value follows in the same millisecond batch; this order is structural but not promised by iced.

**Decisions (the RFC's design, as written, with these fixes):**
1. **Source of truth: the `theme_changes()` events in order, last wins, plus one `iced::system::theme()` task at boot, both into one message** (`SystemThemeMode`). A leading `None` is not read as "no service".
2. **No "re-ask once after the window opens"** (the spike's refinement 2): `theme()` returns iced's own stored value, which is exactly what the window step may have reset, so a second query cannot recover it. Instead the ordering assumption is **pinned**: the real-window stub run (start dark: the page is dark) is a **check of every iced upgrade and of every release candidate** (the release checklist gets a line). If iced ever reorders and the page starts Light under a dark system, the next theme change corrects it and the check shows it.
3. **Settings text, honest about `None`:** "System (currently Dark)", "System (currently Light)", and "System (not reported; using Light)" (this last one covers "no service" and "no preference" alike: the program cannot tell them apart). English and Japanese; text, not colour alone.
4. **The preview's "System" surface follows the same resolved value** (the four `match` arms in `mockups.rs` and the picker label): the spike found they match `ThemeMode::System` themselves and would keep showing Light.
5. **The setting stays `ThemeMode::System` in `settings.json`**; the resolved mode lives only in `AppState`.
6. **Windows and macOS are not verified** (no machine): by winit's documentation the same code path should work (winit reports the theme there); one risk is recorded: the application's `theme()` makes iced set the window theme to the application's mode. The release notes say so.

