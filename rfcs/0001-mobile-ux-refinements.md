# RFC 0001: Mobile UX refinements

- **Status**: Draft
- **Target version**: v1.22.0+ option
- **Author**: logolig maintainers
- **Created**: 2026-05-05

## Summary

Tighten the mobile-layout experience added in v1.20.0 by addressing four
issues left out of scope at that time: (1) the locale/theme picker
popups are hard to use on narrow screens, (2) the popups float in the
window centre instead of next to the sidebar/bottom-nav icon that opened
them, (3) on phones the popup feels foreign because it is not the
"sheet that slides up from the bottom" pattern users expect from native
mobile apps, and (4) the bottom nav has no growth path past three icons.

This RFC scopes each sub-topic individually so implementers can land
them as separate sub-versions (v1.22.0, v1.22.1, …) rather than a
single mega-PR.

## Background

v1.20.0 introduced the desktop-vs-mobile split via `app::is_mobile`
(window width `< 768px`). The sidebar (90px wide) becomes a bottom nav
(64px tall, 3 equal cells), the settings Right Sheet widens to fill
most of the screen, and the asset card grid drops from 3 columns to 2.
What was deliberately deferred was anything that needed iced 0.14
overlay-positioning APIs we hadn't surveyed, and anything that would
have required new icons or surfaces (which would force locale work, a
new design pass, and so on).

In particular the picker popup (`ui::picker_overlay`) is rendered into
snora 0.8's `context_menu` slot, which positions the overlay roughly in
the centre of the work area. On a 1280×720 desktop this looks fine —
the popup hovers near the middle of the window and the user's eye
finds it. On a 375×667 phone it looks *wrong*: the popup is the same
220px wide regardless, and it appears far away from the icon the user
just tapped. We accepted this in v1.20.0 because the alternative — a
sheet that slides up from the bottom — collides with the settings sheet
(snora 0.8 has only one `Sheet` slot at a time), and overlay
positioning needs investigation we hadn't done.

## External design

Four sub-topics. Each lands on a distinct screen state so they can be
shipped independently.

### 1.a Picker popup width on mobile

When the picker is open and `is_mobile(state)` returns true, the popup
fills `window_width - 32px` (16px margin per side) instead of the
fixed 220px. Locale and theme rows reflow naturally — they're just
text and a `✓` prefix — so there's nothing else to redesign.

The 16px margin is symmetric with the settings Right Sheet's mobile
clamp (`window_width - 16` with 16px reserved on the side facing the
content), and reads as "this is a screen-level surface, not a tooltip".

On desktop the popup stays at 220px.

### 1.b Picker popup placement near the icon

Today the popup appears in the window centre (snora's
`context_menu` slot is centred by default). We want it placed adjacent
to the icon that opened it:

- **Desktop (sidebar at left)**: the popup's left edge sits 8px to the
  right of the sidebar's right edge (so 90 + 8 = 98px from the left
  window edge). Vertically it aligns its top edge with the icon's
  top (or close to it — see implementation note in §Design).
- **Mobile (bottom nav)**: the popup's bottom edge sits 8px above the
  bottom nav's top edge. Horizontally it's centred under the icon
  cell that opened it (so the locale popup centres under cell 2 of 3,
  the theme popup under cell 3 of 3).

The "near the icon" placement removes the need to mentally connect
"I tapped here" with "the menu appeared there". Closing behaviour is
unchanged: tap outside (or `Esc` once we wire it) dismisses it.

### 1.c Bottom Sheet–style picker (mobile only, alternative to 1.b)

This is an alternative path to 1.b, not an addition. On mobile, instead
of a small popup near the bottom nav icon, the picker becomes a
full-width sheet that slides up from the bottom edge of the screen.
Visually this matches what users get from native iOS / Android apps
(action sheets, share menus). On desktop the popup behaviour from 1.a +
1.b is unchanged.

The blocker for this is that snora's `Sheet` slot is single-tenant: if
the user has the settings Right Sheet open and then taps a bottom-nav
icon, we'd need to dismiss the settings sheet first. We accept that
behaviour — opening the picker dismisses the settings sheet — and call
it out in copy ("Settings closes when you switch to Language / Theme")
only if user testing shows the implicit dismissal confuses people.

§Design (below) lays out which of 1.b vs 1.c we recommend; both are
implementable.

### 1.d Hamburger menu

When a 4th sidebar / bottom-nav icon is added (e.g. a future "About"
or "Help" entry, or a per-locale region picker), the bottom nav runs out
of horizontal real estate on a 375px phone (4 cells × ~94px is fine,
but 5 cells × 75px is squeezed). At that point the rightmost cell
becomes a hamburger (`lucide::Menu`) that opens a vertical list of the
overflow items.

This RFC does not add the 4th icon; it only specifies the *mechanism*
so that whoever does add it knows where to plug in. Until that day the
hamburger code path is dead.

## Internal design

This was a "Background" + "External design" heavy RFC; the internal
work is mechanical. Each sub-topic has a bounded code blast radius.

### 1.a Width

In `ui::picker_overlay`:

```rust
const POPUP_WIDTH: f32 = 220.0;       // existing desktop default

fn popup_width(state: &AppState) -> f32 {
    if crate::app::is_mobile(state) {
        (state.window_size.width - 32.0).max(280.0)
    } else {
        POPUP_WIDTH
    }
}
```

The `.max(280.0)` floor exists because the locale picker has a
"システム設定に従う" row that needs a sensible minimum. On any phone
sold in the last decade the actual width comes from the
`window_width - 32` branch.

Replace `POPUP_WIDTH` references in `popup_container` with
`popup_width(state)`. No other changes.

### 1.b Anchor positioning

iced 0.14 does not have a CSS-`position: absolute` analogue. The
relevant primitive is `iced::widget::overlay::menu::Menu`, which
positions itself relative to the parent button. Using it directly
means the popup stops being a `context_menu` slot occupant and becomes
an overlay attached to the sidebar/bottom-nav button.

Concretely the migration is:
1. Drop `AppLayout::context_menu(...)` from `shell.rs`.
2. In `ui::sidebar` / `ui::bottom_nav`, render each picker button as a
   `pick_list`-style widget whose options come from
   `LocaleNameJa / LocaleNameEn / LocaleSystem` (or theme equivalents).
3. The widget already handles "open beneath / above based on available
   space", which gives us the desktop and mobile placements for free.

If `pick_list` doesn't visually match the existing picker (it
typically renders as a button-with-arrow), we instead build our own
custom overlay using `iced::advanced::overlay::Element`. That is
more code but gives exact control over the popup's appearance and
position. We expect the `pick_list` route to be sufficient; only fall
back to the custom overlay if a designer review rejects the look.

### 1.c Bottom Sheet style

When `is_mobile(state)` and a picker is active, render the picker as a
`Sheet` at `SheetEdge::Bottom` with `SheetSize::Pixels(...)` calculated
to fit the option rows (~80px header + 48px per row + 16px padding).
This requires:
1. Detect collision: if `state.advanced_open` is true *and* a picker
   becomes active, set `advanced_open = false` first.
2. Replace the `context_menu` rendering path on mobile with a
   `Sheet`-rendering path. Desktop continues to use `context_menu` (or
   the anchored overlay from 1.b — pick one).

We **recommend 1.b over 1.c**. Reasoning:
- 1.b unifies desktop and mobile (same primitive, same code path).
- 1.b doesn't conflict with the settings sheet.
- 1.c requires either dismissing the settings sheet (jarring) or
  finding a second sheet slot in snora (upstream change).

If a designer comes back and insists on 1.c, the implementation above
still applies; just be ready to ship the dismissal copy.

### 1.d Hamburger overflow

Add `state.hamburger_open: bool` to `AppState`. When the implementer
who adds the 4th icon arrives, they:
1. Replace the rightmost `bottom_nav` cell with a hamburger button
   (`lucide::Menu`) that toggles `hamburger_open`.
2. When `hamburger_open` is true, render a vertical menu in the
   `context_menu` slot listing items 4..N.
3. The visible "primary" cells (1..3) stay the most-used three. New
   features start in the overflow and are promoted to primary only
   after usage telemetry confirms demand (logolig has no telemetry, so
   this means "after a designer decides to promote them").

No code is added in this RFC — only the convention is documented so
future hands know where to put the 4th icon.

## Requirements

1. On mobile (`is_mobile(state) == true`), the picker popup width must
   equal `(window_width - 32px).max(280)`.
2. On desktop, the picker popup width must remain 220px.
3. The picker popup must be visually anchored to the icon that opened
   it (1.b) or rendered as a bottom sheet (1.c). The picker must not
   appear in the centre of the work area on either form factor.
4. Opening a picker must not silently close the settings sheet unless
   the visible behaviour is documented in copy. (Implementation 1.b
   avoids this; 1.c requires the warning.)
5. The bottom nav must continue to work with exactly 3 icons after this
   RFC is implemented; the hamburger path is dormant until a 4th icon
   is added.
6. All locale strings (`SidebarLabelLocale`, `LocaleNameJa`, etc.) must
   continue to be reachable through `MessageKey` + `Translator` — no
   new hardcoded text.

## Design

(Same content as "Internal design" above; this section header is
included so the document conforms to the medium-scope template.)

## Test plan

### Manual checks

| Check | Steps | Pass condition |
| --- | --- | --- |
| Width on phone | Resize to 375×667. Open locale picker. | Popup is `375-32 = 343px` wide. |
| Width on desktop | Default 1280×720. Open locale picker. | Popup is 220px wide. |
| Anchor (desktop) | Default 1280×720. Click locale icon. | Popup left edge ≈ 98px from window left. |
| Anchor (mobile) | 375×667. Tap theme icon. | Popup horizontally centred under cell 3 of 3. |
| Settings collision | Open settings, then tap locale. | Settings closes (1.c) **or** picker appears next to icon (1.b). |
| 3-icon bottom nav | Default mobile, no 4th icon. | All three cells equal width, no overflow. |

### Automated tests

- New unit test in `crates/logolig-app/src/ui/picker_overlay.rs`
  verifying `popup_width(state)` for representative window sizes
  (375, 600, 768, 1024, 1920) crosses the mobile boundary correctly.
- No new integration tests — the picker is UI surface, exercised by
  manual checks above.

## Security considerations

N/A. The picker reads state from `AppState` and emits Messages already
defined in v1.18.0; no new IO, no new parsing, no persisted secrets,
no network calls.

## Related ROADMAP entry

See `docs/architecture.md` ROADMAP row for `v1.22.0+ (option)`,
sub-bullet (a). This RFC is the detailed handoff for that bullet.
