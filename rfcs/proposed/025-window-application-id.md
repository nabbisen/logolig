# RFC 025 — Window application ID and desktop integration

**Status.** Proposed
**Priority.** P1 for packaging (affects the AUR package and any Flatpak)
**Tracks.** The application reports an empty Wayland `app_id`.
**Touches.** `crates/app/src/app.rs` (`iced::application(..)` window settings), `packaging/linux/` (desktop file name and `Icon=`), docs.

## Summary

Desktop environments match a window to its desktop entry (icon, name, grouping, "pin to dock") by the window's application id (Wayland `app_id`, X11 `WM_CLASS`). Logolig sets none, so an installed package would show a generic icon and ungrouped windows.

## Evidence

- Verified by execution (2026-10-21, niri, debug build, isolated config): `niri msg --json windows` reports `app_id` as the empty string for the Logolig window (title "Logolig").
- Verified by source reading: `app::run` builds `iced::application(AppState::boot, update, view)` with font, title, theme and subscription only; no window settings.
- Not verified: how iced 0.14 exposes the application id (expected: `window::Settings::platform_specific.application_id` on Linux; to be confirmed in the dependency source), and the X11 `WM_CLASS` value.
- RFC 020's phase-2 handoff told the implementer not to add `StartupWMClass` because no window class was verified; this RFC supplies the verified value.

## Design

1. **ID.** `io.github.nabbisen.Logolig` (reverse-DNS, also the Flatpak ID candidate in RFC 023). Decision D1.
2. Set it through iced's window settings on Linux; add a unit-level check where possible and a manual check (`niri msg windows`, and `xprop WM_CLASS` under X11).
3. **Names.** Desktop file `io.github.nabbisen.Logolig.desktop`, icon name `io.github.nabbisen.Logolig`, MetaInfo `io.github.nabbisen.Logolig.metainfo.xml`. The desktop file's `StartupWMClass` is set only if the X11 class differs from the ID (decide from the X11 evidence).
4. Windows and macOS are unaffected; the setting is Linux-specific.

## Decision requested

- **D1** Adopt `io.github.nabbisen.Logolig` as the application ID for all Linux packaging (AUR and Flatpak). The architect recommends yes. If yes, the RFC 020 phase-2 handoff and RFC 020's binding details are amended **before** that work starts.

## Tests and evidence

Manual: launch, read `app_id` with `niri msg windows`; under X11 read `WM_CLASS`. Installed-package check once packaging exists: the window shows the application icon and groups under the desktop entry.

## Acceptance criteria

The running window reports the agreed ID on Wayland and a matching class on X11; the desktop entry's file name equals the ID.

## Risk

Low. One settings value; a wrong value only affects icon matching.
