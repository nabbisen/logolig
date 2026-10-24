# RFC 015 — SVG input: text rendering and external resources

**Status.** Accepted, phased (owner, 2026-10-21). Phase 1 = verification spike by execution; phase 2 = fix, designed after the evidence exists.
**Priority.** P1 for text rendering (likely visible output defect); P2 for resource hardening
**Handoffs.** [rfcs/handoffs/015-svg-text-and-external-resources/](../handoffs/015-svg-text-and-external-resources/README.md)
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

## Decisions and phasing (2026-10-21, applying the owner's philosophy: clean, safe, robust; users must not be misled)

- **D1 — fonts: load the system fonts, once, shared.** A logo SVG whose text silently disappears is a misleading result, and refusing text SVGs would reject a very common, legitimate input. The font database is built **once** (`OnceLock<Arc<fontdb::Database>>`) and shared by `ingest::parse_svg_size` and every `rasterize` call: `rasterize` runs once per output size, so loading fonts per call would multiply a multi-hundred-millisecond cost by the number of outputs. Phase 1 measures that cost.
- **D2 — external resources: only `data:` URLs.** Every file path in `<image href>` (absolute, relative, nested SVG) is rejected. Safe by default; nothing outside the SVG is ever read. No opt-in setting (that would be a setting nobody can judge).
- **Q1 — bundled fallback font: no.** Binary size, licence review and a hidden dependency; document the dependency on installed fonts instead and recommend converting text to outlines for brand assets.
- **Q2 — advisory: yes, but only for things that matter, and the mechanism is decided after phase 1.** Intended behaviour (one short, non-blocking toast per loaded image, no noise for ordinary SVGs): (a) when external images were **ignored**, because content the user expects is missing; (b) when the SVG contains text, a single sentence that the logo is drawn with fonts installed on this computer and that outlining the text gives the same result everywhere. The architect decides after the spike how core reports this to the app (a field on `SourceAsset` is a public-API change and needs its own decision).

**Phase 1 (the spike handoff)** was evidence only; no production code changes. **Phase 2** (implementation and tests per "Design" above, plus the advisory mechanism) is handed off after the architect has reviewed the phase-1 evidence.

## Phase 1 results and revised decisions (architect review, 2026-10-21)

Phase 1 was executed (spike, evidence only). Verified by execution on one Linux machine (usvg/resvg 0.45.1, fontdb 0.23.0):

- **C1/C2 confirmed:** today SVG text renders **nothing** (0 visible pixels), and the documentation sentence "resvg (uses OS fonts)" is wrong.
- **C3/C4 confirmed:** a local file named in `<image href>` (absolute path; relative path resolved against the process working directory; nested SVG) is read and **ends up in the output**, already at ingest time (`parse_svg_size`).
- **C5 confirmed with a significant caveat:** a data-URL-only resolver works (no file loaded; `data:` images still render), but loading system fonts is **not sufficient**: `font-family="Arial"`, `"serif"`, `"Helvetica"`, `"Times New Roman"` drew **"Aα"** (fontdb resolved the generic serif to a symbol font), and under a minimal fontconfig no generic family resolved at all (text vanished although fonts were installed). A logo with wrong letters is worse than one with missing letters.
- **C6 only partly confirmed:** a shared font database is ~14x faster than rebuilding it per call, but the absolute cost is small (5–9 ms per full load on the test machine). The shared `OnceLock` stays good practice, not an urgent one. Cold-disk, slow machines, Windows and macOS were not measured.
- The parsed tree reports only the *requested* font families; a substituted font is detectable only by querying the font database per requested family (cheap, demonstrated).

**Revised decisions** (these supersede D1 and Q2's mechanism above; D2 stands):

- **Resources (D2, unchanged, now urgent):** only `data:` URLs. One shared options builder (`services::svg::options()`) used by **both** `parse_svg_size` and `rasterize_svg`; the two call sites change together.
- **Reporting mechanism: R1.** A new function `services::svg::analyze(bytes) -> SvgReport { has_text, ignored_external_images, missing_families }`, called by the app after a successful SVG ingest. No change to `SourceAsset` or to `ingest`'s return type (no public-API change).
- **Phase 2a (security and honesty, small; targeted before the first public release):** the data-only resolver at both call sites; `analyze`; **one** combined advisory toast per loaded SVG (never one toast per issue) when external images were ignored and/or the SVG contains text; while text is still not drawn, the text sentence says so plainly and recommends converting text to outlines; documentation corrected (`functional-spec.md` §15). New `en`/`ja` keys.
- **Phase 2b (text rendering, after 2a):** shared font database; **generic-family repair** (after loading, check that `sans-serif`, `serif`, `monospace` resolve to a face that actually has Latin glyphs, using the face's character map, not its name; otherwise point the generic at a chosen installed text family; never accept a symbol face); the honest substitution advisory (F3) driven by `analyze(missing_families)`. The advisory text changes from "text is not drawn" to "text uses a substitute font; outline your text for the same result everywhere". No bundled font (Q1 stands).
- **Why split:** 2a closes a privacy-relevant file read and ends silent misleading output with little code and no cross-platform font risk; 2b is a visible feature with platform-dependent behaviour (including Flatpak, whose sandbox exposes a different font set: RFC 023) and needs its own evidence.
- **Release scope** (whether 2a must precede the first AUR release) is the owner's decision; the architect recommends yes.

## Phase 2a result (architect review, 2026-10-24)

Phase 2a is implemented and approved (`.git-exclude/reviewed/rfc-015-phase-2a-svg-review-2026-10-24.md`): both usvg parse sites use the data-URL-only `services::svg::options()`; `svg::analyze` reports `has_text` and `ignored_external_images`; one combined advisory per loaded SVG. **For phase 2b:** the text sentence of the toast and the test that asserts an empty render of text-only SVGs (T6) must change when text is drawn; the sentence must then say that a substitute font may be used and that outlining the text gives the same result everywhere. `favicon.svg` remains a verbatim copy of the source (documented in `export-spec.md`).

## Phase 2a design amendment (architect, 2026-10-24)

The phase 2a implementer stopped at the handoff's stop condition and proved by execution (experiment E16) that usvg 0.45.1 never keeps text nodes when no font can be resolved (`src/parser/text.rs`), so `has_text` cannot be read from the parsed tree as phase 1 suggested. **Decision: `SvgReport::has_text` is the result of counting calls of the `usvg::FontResolver::select_font` hook (a documented extension point) during an analysis-only parse.** It means "text that usvg would try to draw and that has content". Rejected: an XML scan (second parse; false positives for never-drawn and empty text, and a hand-rolled parser the project forbids), loading fonts (contradicts the 2a scope). The hook is also the natural place for phase 2b to record missing font families. The job is **not split**: nothing ships before the whole of J3 is reviewed, so one reviewed step is simpler than two. `analyze` is not wrapped in `catch_unwind`: usvg is already parsed by ingest and rendering without such a wrapper, and hiding panics would hide bugs; resource limits for hostile input belong to RFC 014.

## Tests (design-derived)

- Fixture SVG with `<text>` renders non-transparent pixels where glyphs are expected (skip with a clear message when the test machine has no fonts; do not silently pass).
- Fixture SVG referencing an absolute path to a temp PNG renders without that image.
- `data:` URL images still render.
- `parse_svg_size` and `rasterize` agree on dimensions for both fixtures.

## Acceptance criteria

SVG text renders on machines with fonts; no file outside the SVG is read during parsing; behaviour is documented.

## Risk

Medium for rendering differences between machines; low otherwise. First task of the work should be a 1-hour spike confirming the text defect by executing a render, since this RFC's evidence is from source reading.
