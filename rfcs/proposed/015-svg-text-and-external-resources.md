# RFC 015 — SVG input: text rendering and external resources

**Status.** Proposed
**Priority.** P1 for text rendering (likely visible output defect); P2 for resource hardening
**Tracks.** (a) SVG `<text>` probably renders as nothing; (b) SVG parsing may read local files.
**Touches.** `crates/core/src/services/rasterize_svg.rs`, `services/ingest.rs` (`parse_svg_size`), a new shared `svg_options()` helper, docs, tests.

## Summary

Core builds `usvg::Options::default()` in two places and never configures fonts or the resource resolver. Logo SVGs very often contain text.

## Evidence

Verified from dependency source (usvg 0.45) and project source; **rendering not executed**.

- `usvg::Options::default()` sets `fontdb: Arc::new(fontdb::Database::new())` — an **empty** font database (usvg `parser/options.rs`).
- `grep` in `crates/core/src` finds no `load_system_fonts` / `fontdb_mut`.
- Therefore `<text>` elements have no fonts to resolve and are most likely dropped from the rendering. `functional-spec.md` §15 claims "resvg (uses OS fonts)", which is not what the code configures.
- usvg's default string resolver (`ImageHrefResolver::default_string_resolver`) resolves `<image href="...">` through `Options::get_abs_path` and reads the file if it exists. With `resources_dir = None`, an absolute path (or a path relative to the process working directory) is read from disk. Supported formats: PNG, JPEG, GIF, WebP, nested SVG.
- Consequence: an SVG chosen by the user can embed a local image file into the generated PNG/ICO. For a local-first tool the user already supplied the SVG, so the practical risk is information disclosure when someone converts an SVG they received from a third party and publishes the output. Severity low-medium.

## Design

1. One function `services::svg::options() -> usvg::Options` used by `rasterize_svg` and `ingest::parse_svg_size`.
2. **Fonts (D1).** Load system fonts (`fontdb_mut().load_system_fonts()`), and set generic family defaults. Cost: first-use font scan latency; do it once (`OnceLock<Arc<fontdb::Database>>`) and share it. Alternative: refuse SVG with text and tell the user to convert text to outlines — safer for reproducibility but unfriendly.
3. **Determinism note.** Text output depends on installed fonts, so the same SVG can render differently on different machines. Document this in `export-spec.md`; recommend outlining text for brand assets. A bundled fallback font would add binary size and needs a licence review; out of scope here (Q1).
4. **External resources (D2).** Replace the image-href resolver with one that accepts only `data:` URLs; all file paths are rejected and logged as a warning to the user ("External images in the SVG were ignored"). Same for nested SVG.
5. Keep `unsafe` forbidden; no new dependencies expected (`fontdb` is already in the tree through usvg).

## Open questions

- **Q1** Bundle a fallback font? Recommendation: no, document instead.
- **Q2** Should an SVG that uses text or external images produce an advisory toast? Recommendation: yes, one non-blocking toast listing "text uses installed fonts" and "external images ignored".

## Tests (design-derived)

- Fixture SVG with `<text>` renders non-transparent pixels where glyphs are expected (skip with a clear message when the test machine has no fonts; do not silently pass).
- Fixture SVG referencing an absolute path to a temp PNG renders without that image.
- `data:` URL images still render.
- `parse_svg_size` and `rasterize` agree on dimensions for both fixtures.

## Acceptance criteria

SVG text renders on machines with fonts; no file outside the SVG is read during parsing; behaviour is documented.

## Risk

Medium for rendering differences between machines; low otherwise. First task of the work should be a 1-hour spike confirming the text defect by executing a render, since this RFC's evidence is from source reading.
