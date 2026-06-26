# RFC 0006: Drop-zone Drag-and-Drop Repair

- **Status**: Implemented (v1.26.1)
- **Target version**: v1.26.1
- **Author**: nabbisen / ChatGPT
- **Created**: 2026-06-26

## Summary

Repair the choose-file page so the Empty-screen drop zone is no longer a
passive visual card. The whole drop-zone card becomes a large click target,
and the app listens to native file-hover/drop events through the general iced
runtime event stream so drag feedback and drops are handled in one place.

## Background

The previous Empty screen visually promised drag-and-drop, but the visible card
itself was passive: only the small **Choose file…** button opened the picker.
Native file-drop support also depends on platform event delivery. For example,
some Linux Wayland environments do not deliver OS file-drop events through the
iced/winit window event path. The app should still feel usable and honest when
that platform limitation appears.

## External design

- On the Empty screen, the entire bordered drop-zone card opens the file picker
  when clicked.
- The small **Choose file…** control is presented as a visual chip inside the
  card, not as a nested button.
- When a native file is hovering over the window and the platform reports that
  event, the drop zone changes both border and background. This is feedback,
  not the only way to operate the page.
- The accepted format copy includes PNG, SVG, WebP, JPG, and JPEG.

## Internal design

- Add `AppState::file_drag_hovering: bool` as transient UI state.
- Add `Message::FileDragHovered` and `Message::FileDragLeft`.
- Replace the previous `iced::window::events()` subscription with
  `iced::event::listen_with(...)` so hover, leave, drop, open, and resize are
  handled from a single runtime-event subscription.
- Map native events as follows:
  - `FileHovered(_)` → `FileDragHovered`
  - `FilesHoveredLeft` → `FileDragLeft`
  - `FileDropped(path)` → `FileDropped(path)`
- Clear `file_drag_hovering` when import starts, when Back/Cancel is pressed,
  or when hover leaves.
- In `ui/drop_zone.rs`, render the card as a full-size button whose inner
  container owns the visual border/background styling.

## Testing

- Verify that clicking anywhere in the drop-zone card opens the file picker.
- Verify that selecting a supported image still starts the usual
  Empty → Converting → Result flow.
- Verify that native file-hover feedback appears on platforms that emit
  `FileHovered`.
- Verify that native file-drop starts conversion on platforms that emit
  `FileDropped`.
- Verify that cancelling the file picker leaves the Empty screen unchanged.
- Verify that Back from Result clears hover feedback and returns to Empty.
