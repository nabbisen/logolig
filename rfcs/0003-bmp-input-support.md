# RFC 0003: BMP input support

- **Status**: Draft
- **Target version**: v1.22.0+ option
- **Author**: logolig maintainers
- **Created**: 2026-05-05

## Summary

Add `.bmp` (Windows Bitmap) as an accepted input format alongside PNG,
WebP, JPEG, and SVG. The work is mostly enabling the `bmp` feature on
the `image` crate dependency and adding a `SourceKind::Bmp` variant.

## External design

The drop zone copy gains "BMP". Locale strings:

- en: "Drop PNG, SVG, WebP, JPEG, or BMP"
- ja: "PNG / SVG / WebP / JPEG / BMP をドロップ"

The file picker filter (in `task_queue::pick_file_task`) gains the
`.bmp` extension. No other UI changes — once the file is loaded the
pipeline is identical to PNG.

## Internal design

Module placements follow the v1.11.0 JPEG precedent exactly. Anyone who
implemented v1.11.0 should be able to do this in an afternoon.

### `crates/core`

1. **`Cargo.toml`** (workspace level): add `bmp` to the `image` crate's
   features list. Currently:
   ```toml
   image = { version = "0.25", default-features = false, features = ["png", "webp", "jpeg"] }
   ```
   Becomes:
   ```toml
   image = { version = "0.25", default-features = false, features = ["png", "webp", "jpeg", "bmp"] }
   ```

2. **`domain/source.rs`** (or wherever `SourceKind` lives): add
   `SourceKind::Bmp`. Update the `Display` impl and any exhaustive
   `match` arms. The compiler will surface every site that needs an
   arm — there should be exactly:
   - `services::ingest` (magic byte / extension dispatch)
   - `services::exporter::run_in_memory` (raster decode branch)
   - `services::preview::render_at` (raster decode branch)

3. **`services/decode_bmp.rs`** (new): mirrors `decode_jpeg.rs` exactly.
   ```rust
   //! BMP デコーダ (v1.22.x)。
   pub fn decode(asset: &SourceAsset) -> Result<Rgba8, AppError> {
       let img = image::load_from_memory_with_format(&asset.raw, image::ImageFormat::Bmp)
           .map_err(|e| AppError::ingest(format!("decode BMP: {e}")))?;
       let rgba = img.to_rgba8();
       let (w, h) = rgba.dimensions();
       let pixels: std::sync::Arc<[u8]> = rgba.into_raw().into();
       Rgba8::try_from_raw(w, h, pixels)
           .ok_or_else(|| AppError::ingest("BMP yielded zero-size buffer"))
   }
   ```
   Register in `services.rs`: `pub mod decode_bmp;`.

4. **`services/ingest.rs`**:
   - Magic byte: BMP starts with `42 4D` ("BM"). Add to the dispatch
     table in `detect_kind_from_bytes` (or whatever helper exists).
   - `parse_bmp_size`: BMP DIB header at offset 14 contains `width: i32`
     (little-endian) at offset +4 and `height: i32` LE at +8. Height
     is signed (negative means top-down rows) — take the absolute value.
   - Extension dispatch: `.bmp` accepted alongside `.png`, etc.

5. **`services/exporter.rs::run_in_memory`**: add a branch in the
   raster decode match to call `decode_bmp::decode`.

6. **`services/preview.rs::render_at`**: same one-line addition.

7. **`services/transparency_audit.rs`**: BMP supports a 32-bit RGBA
   variant but the common case is 24-bit RGB (no alpha). After decode
   via `to_rgba8()`, alpha will be 255 for 24-bit BMP. Audit logic is
   unchanged — `FullyOpaque` is the correct classification and the
   v1.7.0 warning toast will fire. **Do not add a BMP-specific toast**
   — unlike JPEG, where transparency is *impossible*, BMP can carry
   alpha. The generic "fully opaque source" warning is correct here.

### `crates/logolig-i18n`

Update both `en.toml` and `ja.toml`:

- `drop_zone_headline_en` / `drop_zone_headline_ja` already exists from
  v1.11.0; just append BMP to the format list.
- No new `MessageKey` variants needed.

### `crates/logolig-app`

`task_queue::pick_file_task`'s file dialog filter gains `"bmp"`:

```rust
.add_filter("Images", &["png", "svg", "webp", "jpg", "jpeg", "bmp"])
```

## Test plan

Mirror `tests/decode_jpeg.rs` and the JPEG branches of `tests/ingest.rs`:

1. `tests/fixtures.rs`: add `bmp_4x4_red()` (use `image` crate's BMP
   encoder; BMP minimum size is 1×1 but use 4×4 to match other fixtures).
2. `tests/decode_bmp.rs` (new): 5 tests parallel to decode_jpeg —
   roundtrip, dimensions, alpha=255, error on empty input, error on
   wrong format bytes.
3. `tests/ingest.rs`: 4 new cases — magic byte detection, `.bmp`
   extension, `.BMP` (uppercase) extension, malformed-header recovery.

No new exporter tests — the exporter's behaviour is identical for any
raster source, and v1.21.0 keep-transparency tests already cover the
"opaque raster" path through PNG / JPEG.

## Security considerations

BMP has historically been a vector for parser exploits (run-length
encoding heap overflows, etc.). The `image` crate's BMP decoder is
written in safe Rust and is fuzz-tested upstream by the maintainers.
We benefit from that work for free; logolig adds no new parsing code.

The one site where logolig *does* parse BMP is `parse_bmp_size`, which
reads two `i32`s from a fixed offset. Bounds-check the file length
(must be at least 26 bytes for the BITMAPCOREHEADER variant) before
reading; on too-short input return `None` so the caller falls back to
"unknown size". This matches the JPEG path (`parse_jpeg_size`).

## Related ROADMAP entry

See `docs/architecture.md` ROADMAP row for `v1.22.0+ (option)`,
sub-bullet (c).
