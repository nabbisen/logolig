# snora Feature Request: BottomSheet Internal Scroll + Sticky Footer

> **Status (as of logolig v1.13.0): Partially resolved — historical document**
>
> In snora 0.8.0 the old `BottomSheet` was restructured into the generic
> `Sheet` widget, with its "pure content carrier" design intent documented
> explicitly in the API. As a result:
>
> - **Internal scroll**: the responsibility of wrapping content in
>   `iced::widget::scrollable` was clarified as the application's own, so
>   logolig can handle this itself (implemented in v1.15.0).
> - **Sticky footer**: likewise achievable on the application side by
>   placing a `scrollable` region above a fixed row inside `Sheet`'s
>   content column.
>
> No additional API from snora is required. This document is retained as
> a record of the original problem statement.

## Problem

snora's `BottomSheet` widget is well-suited for auxiliary UIs with many
settings items. logolig uses it for the advanced settings drawer.

However, when the content exceeds the sheet's allocated height, two
problems arise.

### Problem 1: Content below the visible area is unreachable

When sheet content is taller than the sheet's display height, the bottom
of the content — for example the "Reset" and "Close" action buttons — is
clipped and cannot be reached. The sheet occupies a fixed portion of the
screen (for example 50%); content that overflows is simply cut off.
Users experience the sheet as broken.

### Problem 2: Footer actions are buried in the scroll area

Action buttons at the bottom of a sheet (confirm / cancel / close) should
always be visible regardless of how long the content is. The current
`BottomSheet` has no mechanism for pinning a footer, so these buttons end
up in the same scrollable column as the other content and disappear when
the content is long.

## Expected behaviour

### Expected 1: Internal content scroll

The content area inside `BottomSheet` should scroll independently when it
exceeds the sheet's height:

- The sheet's outer geometry (position, size, overlay backdrop) is
  unchanged.
- Only the content region scrolls.
- The user can reach the bottom of the content via scrollbar or scroll
  gesture.
- The sheet does not overflow or shift during scrolling.

### Expected 2: Sticky footer

`BottomSheet` should support an optional footer slot whose contents are
always pinned to the bottom edge of the sheet regardless of scroll
position:

- The footer is independent of content scrolling.
- Footer buttons (e.g. "Reset", "Close") are always pressable.
- When a footer is present, the content region's height is reduced by the
  footer's height (no content hidden behind the footer).

### Expected 3: Footer is optional

Not all `BottomSheet` usages need a footer. When omitted, the entire sheet
is a scrollable content area (Expected 1 applies; no layout change
otherwise).

## Illustration

Desired layout for logolig's advanced settings sheet:

```
┌──────────────────────────────────┐  ← sheet top (managed by snora)
│ ▼ What to export                 │
│   [ ] favicon.ico                │  ↕ scrollable content region
│   [ ] apple-touch-icon.png       │  ↕
│   [ ] favicon.svg                │  ↕
│       └ Vectorize raster sources │  ↕
│       └ Preset: Sharp ▾          │  ↕
│   [ ] favicon-snippet.html       │  ↕
│   PNG sizes: at defaults: ...    │  ↕
│   ICO sizes: at defaults: ...    │  ↕
│ ▶ Extras                         │  ↕
│ ▶ Rendering quality              │  ↕
├──────────────────────────────────┤  ← footer (sticky, always visible)
│  [ Reset ]              [ Close ]│
└──────────────────────────────────┘  ← sheet bottom
```

Scrolling the content area leaves the footer row in place; the user can
always reach "Reset" and "Close".

## Compatibility preferences

- The existing `BottomSheet` API (`new(content)`, `with_height`, etc.)
  should remain backward-compatible.
- When no footer is specified, behaviour should match the current
  implementation except for the addition of internal scroll (Expected 1).
- Intended for a minor version release (e.g. 0.5).

## Background

logolig's advanced settings were reorganised into four accordion groups
in v1.10.x, and v1.10.3 reduced the default-expanded group to one
("What to export") to keep the sheet within its height budget. However,
when the user expands all groups, or as more settings are added, in-sheet
scroll and a sticky footer would be the correct component-level solution.
