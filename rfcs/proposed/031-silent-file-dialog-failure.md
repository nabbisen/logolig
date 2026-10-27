# RFC 031 — A file dialog that cannot open must not fail silently

**Status.** Proposed
**Priority.** P2
**Tracks.** Found by the J5 implementer (RFC 020 phase 2), verified by the architect in the `rfd` source.
**Touches.** `crates/app/src/task_queue.rs` (`pick_file_task`, the two `save_file` tasks), `crates/app/src/app.rs` (messages), i18n (one toast), docs.

## Summary

On Linux the file chooser and the Save dialogs go through the XDG desktop portal (`rfd` default feature `xdg-portal`). When there is no portal service, or no backend for it, the dialog does not open and the application says nothing: "Choose file…", Ctrl+O and "Download" appear to be dead. The user cannot tell this from a cancelled dialog. A user must not be left confused about the main action.

## Evidence

- J5 review request §8: with a private session bus that has no portal service, Ctrl+O opens no window, shows no toast and writes nothing to stderr; the application stays alive. "Portal frontend without a backend" was inconclusive (the test bus was too minimal); it is unproven either way.
- `rfd` 0.15.4, `src/backend/xdg_desktop_portal.rs`: on a portal error the backend logs it through the `log` crate and returns `None` (lines about 71–77 for the file chooser, 106–112 for save). `pick_file()` and `save_file()` return `Option`, so the application receives the same `None` as for a cancel. The application installs no logger, so nothing is printed.
- `pick_file_task` maps `None` to `FilePicked(None)`; the two `save_file` tasks treat `None` as cancel.
- AUR package: `xdg-desktop-portal` is in `depends`, a backend (`xdg-desktop-portal-impl`) only in `optdepends`, because forcing a provider would install an unrelated desktop's backend on many systems. Users of a bare window manager are the exposed group.

## Requirements

1. When a dialog cannot be shown because the portal (or its backend) is unavailable, the user sees **one** error toast that says what is missing and what to do (for example: "The file dialog could not be opened. Install and start a desktop portal such as xdg-desktop-portal-gtk, or drag a file onto the window."). Drag and drop remains the documented fallback for opening; for saving, name a way out in the text.
2. A cancelled dialog stays silent.
3. No new setting, no new network access, no new `unsafe`.

## Design (open: needs a short spike first)

The difficulty is that the public `rfd` API cannot distinguish cancel from failure. Candidates, to be measured in the spike and decided by the architect:

- **A. Probe before opening:** ask the session bus whether `org.freedesktop.portal.Desktop` has an owner (and optionally whether a file chooser implementation is registered). Needs `zbus` or `ashpd`, already in the tree through `rfd`; adding a **direct** dependency edge requires the usual lock check (no new package).
- **B. Call the portal with `ashpd` directly** instead of `rfd` on Linux: exact errors, but replaces a dependency's job and grows the code. Probably too large for the benefit.
- **C. Timing heuristic** (a `None` that returns within a few hundred milliseconds counts as failure): rejected, a fast cancel would produce a false error.
- **D. Do nothing but document it:** the status quo; the package `optdepends` text and the packaging guide already say so. Acceptable only if A proves disproportionate.

Recommendation: spike A; if it is a few dozen lines and adds no package, do it; otherwise D with a sharper `optdepends` description.

## Related, recorded here to avoid a second RFC

- **No system font means no text.** The UI text uses system fonts through `fontdb`; only the icon font is bundled. With every system font hidden the window opens and shows icons only (J5 evidence). The AUR package depends on the virtual `ttf-font`, which solves it there; Flatpak runtimes ship fonts (RFC 023); RFC 015 phase 2b already needs a font policy. No action here; if a non-packaged use becomes important, bundling one font is a licensing and size decision for the owner.

## Test plan

Unit: the mapping from the probe result to a message (pure function). Manual (native Wayland, `wtype`, private bus as in the J5 evidence): no portal → one toast, and a second Ctrl+O gives one more toast, not a stack; with a portal → no toast; cancel → no toast.

## Security considerations

A bus-name probe reads no user data. No file access changes.
