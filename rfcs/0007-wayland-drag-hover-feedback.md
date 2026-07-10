# RFC 0007: Wayland Drag-Hover Feedback Follow-Up

- **Status**: Draft
- **Target version**: future v1.x
- **Author**: nabbisen / ChatGPT
- **Created**: 2026-07-10

## Summary

Improve or document file drag-hover visual feedback on Linux Wayland sessions.
This is a follow-up to RFC 0006 and is not a v1.26.1 release blocker because
the choose-file flow remains usable through the full-card file-picker fallback.

## Background

RFC 0006 repaired the Empty-screen drop zone by making the whole card clickable
and by wiring native `FileHovered`, `FilesHoveredLeft`, and `FileDropped` events
through the iced runtime event stream. Manual v1.26.1 checks later showed that
per-file save, ZIP save contents, and ZIP timestamp metadata pass, but
drag-hover visual feedback did not appear during a Wayland hover test.

The current implementation is still acceptable for release because native file
drag/drop event delivery is platform- and compositor-dependent, and the user can
always click the drop zone to open the file picker.

## External design

- The Empty-screen drop zone should continue to be fully clickable.
- If the platform emits reliable native file-hover events, the drop zone should
  show the stronger hover border/background.
- If the platform does not emit file-hover events, the UI should remain usable
  and should not imply that conversion is blocked.
- Any future user-facing wording should be minimal; the main affordance remains
  the clickable drop zone.

## Internal design

Investigate whether Wayland sessions can provide better file-hover coverage
through the current iced/winit event path or a future iced/winit upgrade. If
not, keep the implementation as-is and document the platform limitation.

Do not add compositor-specific hacks unless they are small, well-contained, and
covered by a manual QA note. The fallback file-picker path is the primary
reliability mechanism.

## Testing

- Verify drag-hover feedback on at least one X11 session or desktop session
  known to emit native file-hover events.
- Verify behavior on at least one Wayland session.
- Verify click-to-pick still works when drag-hover feedback is absent.
- Keep the existing app-state tests for `FileDragHovered`, `FileDragLeft`, and
  `FileDropped` transitions.

## Release Policy

This RFC tracks future polish and platform compatibility. It must not block
v1.26.1 unless a later review finds that file picking or file drop itself is
broken.
