# RFC 0002: User-specified flatten color

- **Status**: Draft
- **Target version**: v1.22.0+ option
- **Author**: logolig maintainers
- **Created**: 2026-05-05

## Summary

When transparency is flattened (the v1.21.0 "Keep transparency: OFF"
mode), the alpha is currently composited against a fixed white
background. This RFC lets the user pick the background colour: white,
black, or an arbitrary RGB value entered as a hex code. White stays the
default.

## Background

v1.21.0 shipped the keep-transparency toggle with a deliberately
narrow scope: white-only background, no UI for choosing a colour.
That kept the v1.21.0 surface area small and matched the favicon-tool
convention (most generators flatten to white). The feedback we anticipate
once it ships is "I want it black" or "I want it `#1a1a1a`" — designers
making favicon variants for dark themes need the inverse.

The plumbing for the colour exists: `services::flatten::flatten_to_white`
is a thin wrapper over Porter-Duff "over"-composition with a hardcoded
white. Generalising it to any opaque RGB is one signature change plus a
new `ExportPlan` field. The non-trivial part is the UI for picking the
colour without bloating the Settings drawer.

## External design

### The control

A new control row appears in the Settings drawer's "その他 / Misc"
section, immediately under the existing "Keep transparency" checkbox:

```
☑ Keep transparency (alpha)
   When off, flatten with: ⚪ White  ⚫ Black  ●  Custom: [#FFFFFF ]
                                     ^         ^               ^
                                     radio     radio           hex input
```

Three exclusive options. The hex input next to "Custom" is editable
only when "Custom" is selected; otherwise it shows the current colour
greyed out (so the user always sees what would be applied).

The whole row is greyed out / disabled when "Keep transparency" is
checked, because the choice has no effect. Greying-out (rather than
hiding) keeps the UI from jumping when the user toggles transparency.

Validation rules for the hex input:
- Accepts `#RRGGBB`, `#RGB` (expanded to `#RRGGBB`), `RRGGBB`, `RGB`.
- Whitespace and the leading `#` are tolerated.
- Invalid input keeps the previous valid colour and shows a Warning
  toast on blur (same pattern as Web Manifest colour validation in
  v1.8.0).
- Alpha components (`#RRGGBBAA`) are rejected — flatten target must be
  fully opaque to be meaningful.

### Defaults and persistence

- Default is **White** (matches v1.21.0 behaviour exactly — existing
  users see no change).
- The choice is persisted in the existing `PersistedSettings` JSON via
  the new `ExportPlan::flatten_color` field. `serde(default)` ensures
  v1.21.0 settings files load with the white default.

### Result preview

When the user changes the flatten colour in the drawer, the Result
screen's asset card thumbnails do **not** automatically re-render —
v1.21.0 already established that plan changes don't trigger
re-conversion (the user re-drops the file). This RFC inherits that
behaviour. A future RFC may add live re-rendering as a separate scope.

## Requirements

1. The user must be able to choose between three flatten-background
   modes: White (default), Black, and Custom RGB.
2. The Custom mode must accept hex input in formats `#RRGGBB`, `#RGB`,
   `RRGGBB`, `RGB` (case-insensitive).
3. Invalid hex input must not corrupt persisted state. The previous
   valid value is retained and a Warning toast surfaces the input
   error.
4. The flatten-mode controls must be visually disabled (greyed out)
   when "Keep transparency" is checked, but still display the current
   selection.
5. v1.21.0 settings files must continue to load without error,
   defaulting `flatten_color` to white.
6. The flatten step in `services::flatten` must produce identical
   pixel output for white as v1.21.0's `flatten_to_white` (byte-for-byte
   compatibility test).
7. SVG output is unaffected (carry-over from v1.21.0 RFC scope).

## Design

### Domain model

In `crates/core/src/domain/export_plan.rs`:

```rust
/// RGB colour for compositing alpha when keep_transparency=false.
/// Components are 0..=255 in straight (un-premultiplied) sRGB space,
/// matching how Rgba8 pixels are stored elsewhere in logolig (core).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlattenColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FlattenColor {
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };

    /// Parse a hex string. Accepts "#FFF", "#FFFFFF", "FFF", "FFFFFF".
    /// Case-insensitive. Whitespace tolerated. Returns None for any
    /// input that includes alpha (`#RRGGBBAA`) or has invalid chars.
    pub fn from_hex(input: &str) -> Option<Self> { /* ... */ }

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Default for FlattenColor {
    fn default() -> Self { Self::WHITE }
}
```

Add to `ExportPlan`:

```rust
pub struct ExportPlan {
    // ... existing fields
    pub keep_transparency: bool,         // v1.21.0
    /// v1.22.x: When `keep_transparency=false`, the RGB to composite
    /// against. Default white preserves v1.21.0 behaviour.
    pub flatten_color: FlattenColor,
}
```

The struct-level `#[serde(default)]` on `ExportPlan` keeps v1.21.0
JSON files loadable.

### Flatten service

`services::flatten::flatten_to_white` becomes a thin wrapper:

```rust
pub fn flatten_to_white(src: &Rgba8) -> Rgba8 {
    flatten_to(src, FlattenColor::WHITE)
}

pub fn flatten_to(src: &Rgba8, bg: FlattenColor) -> Rgba8 {
    // Same Porter-Duff "over" math as v1.21.0, but the constants
    // 255.0 in the formulas become bg.r/g/b cast to f32.
    // The alpha=255 / alpha=0 short-circuit branches still apply:
    //   alpha=255 -> output is unchanged source RGB
    //   alpha=0   -> output is bg.{r,g,b,255}
}
```

The exporter call site in `render_at_size` becomes:

```rust
if plan.keep_transparency {
    Ok(rgba)
} else {
    Ok(crate::services::flatten::flatten_to(&rgba, plan.flatten_color))
}
```

### UI control

In `ui::advanced_drawer`'s "Misc" section, add three iced `radio` widgets
plus a `text_input` for the custom hex. The state on `AppState`:

```rust
pub enum FlattenMode {
    White,
    Black,
    Custom,
}

pub struct AppState {
    // ...
    pub flatten_mode: FlattenMode,
    pub flatten_custom_input: String,    // raw text the user is typing
}
```

The `FlattenMode` enum is **UI state only** — it doesn't go into
`PersistedSettings`. Persisted state is just `flatten_color: FlattenColor`.
On startup, derive `flatten_mode` from `flatten_color`:

```rust
fn flatten_mode_from(c: FlattenColor) -> FlattenMode {
    match c {
        FlattenColor::WHITE => FlattenMode::White,
        FlattenColor::BLACK => FlattenMode::Black,
        _                  => FlattenMode::Custom,
    }
}
```

This way, "Custom #FFFFFF" persisted on disk reads back as "White"
mode (with a `#FFFFFF` value loaded into the input) — which is correct
because they're indistinguishable.

### Messages

```rust
// In Message enum:
FlattenModeSelected(FlattenMode),
FlattenCustomHexEdited(String),
FlattenCustomHexSubmitted,    // on blur or Enter
```

Handlers:
- `FlattenModeSelected(White)` → `flatten_color = WHITE`, `flatten_mode = White`, `persist_settings`.
- `FlattenModeSelected(Black)` → analogous with BLACK.
- `FlattenModeSelected(Custom)` → `flatten_mode = Custom` (don't change colour yet — wait for valid hex).
- `FlattenCustomHexEdited(s)` → update `flatten_custom_input` only.
- `FlattenCustomHexSubmitted` → parse, success → set `flatten_color` and persist; failure → warning toast.

### Disabled state

Disable the radios + text input when `keep_transparency == true`. iced
0.14's `radio` and `text_input` both support `.style(...)` and have
`on_press`/`on_input` parameters that, when `None`, render the widget
non-interactive.

## Test plan

### Unit tests in `services::flatten`

Add tests in `services/flatten.rs`:

| Test | Scenario |
| --- | --- |
| `flatten_to_white_matches_v1_21_byte_for_byte` | Run `flatten_to_white` and `flatten_to(WHITE)` on the same `Rgba8`; assert byte equality. |
| `flatten_to_black_inverts_white_extreme` | Half-alpha pixel `(0, 0, 0, 128)` flattened against black yields `(0, 0, 0, 255)` (no blending visible). |
| `flatten_to_black_blends_with_red` | Half-alpha pixel `(255, 0, 0, 128)` flattened against black yields `(~127, 0, 0, 255)`. |
| `flatten_to_arbitrary_color_blends_correctly` | Half-alpha white `(255, 255, 255, 128)` against `(0, 100, 200)` yields approximately `((255+0)/2, (255+100)/2, (255+200)/2, 255)`. |

### Unit tests for hex parsing

In `domain/export_plan.rs` (or `domain/flatten_color.rs` if extracted):

| Test | Input | Expected |
| --- | --- | --- |
| `parses_full_hex` | `"#FFAA33"` | `Some({r: 0xFF, g: 0xAA, b: 0x33})` |
| `parses_short_hex` | `"#FA3"` | `Some({r: 0xFF, g: 0xAA, b: 0x33})` |
| `parses_no_leading_hash` | `"FFAA33"` | `Some(...)` |
| `parses_lowercase` | `"#ffaa33"` | `Some(...)` |
| `parses_with_whitespace` | `"  #FFAA33  "` | `Some(...)` |
| `rejects_alpha` | `"#FFAA3380"` | `None` |
| `rejects_invalid_chars` | `"#GGAA33"` | `None` |
| `rejects_wrong_length` | `"#FFAA"` | `None` |
| `roundtrip` | parse then to_hex | equal to canonical `#RRGGBB` form |

### Integration test in `tests/exporter.rs`

```rust
#[test]
fn keep_transparency_false_with_custom_color_blends_to_that_color() {
    let asset = ingest_bytes("ht.png", fixtures::png_4x4_half_alpha_red()).unwrap();
    let mut plan = ExportPlan::default();
    plan.png_sizes = vec![32];
    plan.include_ico = false;
    plan.include_apple_touch = false;
    plan.include_html_snippet = false;
    plan.include_svg = false;
    plan.keep_transparency = false;
    plan.flatten_color = FlattenColor { r: 0, g: 0, b: 0 };  // black

    let arts = run_in_memory(&asset, &plan).unwrap();
    let png_art = arts.iter().find(|a| a.relative_path.to_string_lossy() == "favicon-32.png").unwrap();
    let img = image::load_from_memory(&png_art.bytes).unwrap();
    let rgba = img.to_rgba8();
    let center = rgba.get_pixel(16, 16);

    // Half-alpha red (0xCC, 0x33, 0x33, 0x80) blended with black halfway:
    // R ≈ 0xCC * 128/255 ≈ 0x66, alpha=255.
    assert!((0x55..=0x77).contains(&center[0]));
    assert_eq!(center[3], 255);
}
```

### Migration test

Verify that loading a v1.21.0 settings JSON (no `flatten_color` field)
yields `flatten_color = WHITE`. Add to `PersistedSettings` tests:

```rust
#[test]
fn loads_v1_21_json_with_white_flatten_default() {
    let v1_21_json = r#"{
        "export_plan": { "keep_transparency": true /* no flatten_color */ }
    }"#;
    let parsed: PersistedSettings = serde_json::from_str(v1_21_json).unwrap();
    assert_eq!(parsed.export_plan.flatten_color, FlattenColor::WHITE);
}
```

### Manual checks

| Check | Steps | Pass condition |
| --- | --- | --- |
| White preserves v1.21.0 | Default config, OFF transparency, drop half-alpha PNG. | Output matches v1.21.0 binary. |
| Black blending visible | Set Custom = `#000000`. | Output PNG has darkened semi-transparent regions. |
| Invalid hex rejected | Type `#GGGGGG` and blur. | Toast appears, previous colour retained. |
| Disabled state | Toggle Keep transparency ON. | Radios + input greyed out, can't be clicked. |
| Persistence | Set Custom `#1a1a1a`, restart. | Custom radio still selected, input shows `#1A1A1A`. |

## Security considerations

The hex parser must not panic on adversarial input (very long strings,
non-ASCII, control characters). Tests cover the common bad cases;
fuzz-testing `FlattenColor::from_hex` with `cargo-fuzz` is overkill but
worth recording as a follow-up if the parser grows beyond simple hex.

No file IO, no network. The flatten colour is opaque RGB so cannot
encode hidden alpha that affects downstream tools.

## Related ROADMAP entry

See `docs/src/architecture.md` ROADMAP row for `v1.22.0+ (option)`,
sub-bullet (b).
