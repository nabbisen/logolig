# RFC 012 — Web manifest input validation

**Status.** Implemented (1.29.0). Earlier: Accepted (architect, under the owner's delegation of 2026-11-22 "review them carefully with my fundamental philosophy"; release 1.29.0; decisions in the section "Decisions 2026-11-22")
**Priority.** P2
**Tracks.** Defect: manifest colour/name validation exists but is never invoked.
**Touches.** `crates/core/src/domain/web_manifest.rs`, `services/exporter.rs`, `crates/app` (Customize page, `app.rs` handlers), `crates/i18n` (reuse of dormant keys), docs.

## Summary

`WebManifestSettings::is_valid_color` and `has_required_text` are tested but called nowhere in production code. Since v1.16 conversion starts automatically after ingest, so the "validated at export time" behaviour that the README still promises no longer exists. Invalid values are written straight into `manifest.webmanifest`.

## Evidence

Verified by source reading.

- `grep` over `crates/` finds `is_valid_color` / `has_required_text` only in their definition and in `crates/core/tests/manifest_writer.rs`.
- `app.rs` comment on `WebManifestThemeColorChanged`: validated "at export time", yet no export step exists.
- `MessageKey::ToastInvalidColorTitle` / `ToastInvalidColorBody` are never referenced.
- README: "If a malformed color is detected at export, a Toast is shown and the export is blocked."
- Each keystroke in the four manifest fields calls `persist_settings`, so half-typed values (`#FF`) are written to `settings.json`.
- The core design decision (v1.8) is `#RRGGBB` only.

## Design

1. **Edit buffers in the app.** Like `png_size_input`, keep raw text for the four fields in `AppState`. The `ExportPlan` and `settings.json` only receive a value when it is valid; the previous valid value is retained otherwise.
2. **Inline feedback on the Customize page**, shown after the field loses focus or Enter is pressed: a text message plus a `⚠` marker, never colour alone (ABDD). Reuse `ToastInvalidColor*` text as inline message keys, or retire the toast keys (coordinate with RFC 016).
3. **Defence in depth in core.** `exporter::run_in_memory` validates the manifest settings when `web_manifest` is `Some` and returns a typed `AppError::Export` with an i18n key. Reachable only through a hand-edited `settings.json`, since the UI cannot commit invalid values.
4. **Loading.** On startup, invalid manifest values found in `settings.json` are replaced by defaults with a warning toast; the app does not fail to start.
5. `name` / `short_name` must be non-empty after trimming (`has_required_text`); same inline treatment.
6. Share the hex-input component with RFC 002 (user-specified flatten colour) if both are accepted.

## Alternatives considered

- Block conversion with an error toast: punishes the user at drop time for a setting they edited long before; rejected.
- Normalise `#RGB` to `#RRGGBB`: reasonable and friendly, but a new feature; left to the owner (Q1).

## Open question

- **Q1** Accept `#RGB` shorthand and normalise? Recommendation: yes, because RFC 002 already proposes it for flatten colour.

## Tests (design-derived)

- Core: `run_in_memory` rejects `theme_color = "red"` and an empty name; accepts defaults.
- App: typing `#FF` does not change `export_plan` or persisted state; committing `#FFAA33` does.
- App: invalid committed value keeps the previous valid colour and exposes an error state for the view.
- Settings load: invalid values in JSON are replaced by defaults.

## Acceptance criteria

No invalid colour or empty name can reach `manifest.webmanifest` through the UI or a hand-edited settings file; README text matches the implemented behaviour.

## Risk

Low-medium: touches input handling in the largest file; sequence after RFC 017.

## Decisions 2026-11-22 (architect; the owner's philosophy)

- **Q1 decided: yes, `#RGB` is accepted** and **canonicalised to `#RRGGBB` (upper case)** when the value is committed; once accepted, the field shows the canonical form, so **what the user sees is what is written** to the manifest (no silent difference). Core keeps `#RRGGBB` as the only stored form.
- **Inline feedback uses an icon, not the `⚠` glyph** of the design section above: the application bans text-glyph state cues (RFC 021 D7; a source test enforces it). Use the lucide warning icon beside a plain-language message under the field; never colour alone. The message names what is wrong and what is accepted ("Use a colour like #1A2B3C or #1AB").
- Invalid text is **never saved**: the previous valid value stays in `settings.json` (the edit buffer holds the raw text), which also fixes half-typed values being persisted on every keystroke.
- The retired toast keys `ToastInvalidColorTitle`/`Body` stay in the public `MessageKey` (SemVer, RFC 016 amendment: no removal in 1.x); the inline messages use new keys.
