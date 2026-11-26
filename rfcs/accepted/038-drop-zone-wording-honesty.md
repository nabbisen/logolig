# RFC 038 — Drop-zone wording that is true on every system

**Status.** Accepted (architect; owner authorized 2026-11-26: "Authorized." to my recommendation of the same day)
**Priority.** P2
**Target version.** 1.30.0
**Touches.** `crates/i18n/locales/{en,ja}.toml` (four strings), `crates/app/src/ui/drop_zone.rs` (no layout change expected), the tests that name the old strings, `docs/src/functional-spec.md`

## Evidence (2026-11-26)

- The owner ran the application on Wayland (niri), as a raw build and as a Flatpak: **dragging a file from the file manager onto the window does nothing in both.** The code has said for a long time that some Wayland setups do not deliver file drops through iced/winit (`app/state.rs`, `ui/drop_zone.rs`; RFC 007, RFC 006). Where drops do arrive (other platforms) was not verified by us for Windows, macOS or X11.
- The Empty screen headline says **"Drop PNG, SVG, WebP, or JPEG"** and the secondary line says "or click anywhere in this area to choose a file". On a system that never delivers drops, the first thing a user reads is a promise that does not work.
- The chooser-unavailable toast (`ToastDialogUnavailableChooserBody`) says: "No file dialog is available on this system. **Drag a file onto the window instead**, or install a desktop portal…". On a system where drops do not arrive and no file dialog opens, that advice leaves the user with nothing.
- The application cannot tell in advance whether a drop will ever be delivered, so the wording must be true whichever it is.

## Decision

The click is the sure path and is **named first**; a drop is **named as something that may also work**, never as the way.

| Key | English (new) | Japanese (new, for the owner's review) |
|---|---|---|
| `drop_zone_headline` | Choose a PNG, SVG, WebP, or JPEG | PNG / SVG / WebP / JPEG を選択 |
| `drop_zone_secondary` | Click anywhere in this area. You can also drop a file here, if your system allows it. | この領域のどこでもクリックして選べます。お使いのシステムが対応していれば、ファイルをここにドロップすることもできます。 |
| `toast_dialog_unavailable_chooser_body` | No file dialog is available on this system. Install a desktop portal with a file dialog (for example xdg-desktop-portal-gtk), or, if your system allows it, drop a file onto the window. | このシステムでは、ファイルダイアログを利用できません。ファイルダイアログを備えたデスクトップポータル (例: xdg-desktop-portal-gtk) をインストールするか、お使いのシステムが対応していれば、ウィンドウにファイルをドロップしてください。 |

`drop_zone_accepted_formats` is unchanged. `drop_zone_instruction` (already on the unused list) is not touched. The drop **behaviour** is unchanged: the card is one click target, and `FileDropped` is handled wherever the platform delivers it.

**Why not remove the mention of dropping:** on systems where drops work, users expect a drop zone to accept one, and nothing is lost by saying so. **Why not "Drop … here" first:** it is false on the owner's system. **Why not detect support:** the application cannot know before the first event; a guess would be a second source of untruth.

## Not in scope

Making drops work on Wayland (a toolkit matter, RFC 007); a visual change to the card; new locales; the MetaInfo screenshot (the Result screen does not change; the Empty screen is not in it).

## Acceptance

- The three strings above in both locales; every test that named the old strings follows (`ui_layout_tests/controls_home.rs` and any other found by search), none weakened; the card fits at the window sizes the layout tests already cover (including the phone layout, the largest interface size and Japanese, where the new secondary line is longer): no clipped or hidden text, and the card stays one tab stop with the same action.
- `functional-spec.md`: the drag-and-drop sentence says it works where the platform delivers file drops, and that the click always works; nothing in the docs or the store text promises a drop unconditionally.
- Gates as always; the Japanese strings are listed for the owner's review (the architect adds them to the owner's list).

## Risks

A longer secondary line may wrap to three lines at the largest interface size on the phone layout: the layout tests decide, and the wording may be shortened (the meaning must stay: click always; drop only if the system allows it).
