# Developer Handoff — RFC 015, Phase 2a: SVG safety and honesty (job J3)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`; **GUI evidence rules: `.git-exclude/rules/project-instructions-gui-common.md`, section "GUI Evidence and Automation Rules"**).

Governing RFC: [RFC 015](../../accepted/015-svg-text-and-external-resources.md), section "Phase 1 results and revised decisions". The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request. Phase 1 evidence: `.git-exclude/review-request/rfc-015-phase-1-spike/` and `.git-exclude/reviewed/rfc-015-phase-1-spike-review-2026-10-21.md`.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision): no topic branches, no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first, small commits, **chain every gate with `&&` and push only when all pass**, never rewrite history or force-push, no tags. This is job J3 in `ROADMAP.md`; prerequisite: J2b (snora 0.53) reviewed, which it is if `Cargo.lock` has `snora` 0.53.x.

## 1. Purpose

Two defects confirmed by execution in phase 1, fixed with the smallest change that is safe and honest. **This phase does not render SVG text** (that is phase 2b).

1. **Safety:** an SVG can name a local file in `<image href="...">` (absolute path, a path relative to the process working directory, or a nested SVG) and `usvg` reads it, already during ingest (`parse_svg_size`) and again during rendering. The file's pixels end up in the generated icons. After this job **no file outside the SVG is ever read**; only `data:` images are used.
2. **Honesty:** SVG `<text>` currently renders as nothing (empty font database), silently. After this job the user is told, once per loaded SVG, in one non-blocking toast, what is not shown and what to do about it.

## 2. Background (verified facts)

- `usvg::Options::default()` is built in two places: `crates/core/src/services/rasterize_svg.rs` (in `rasterize_to_canvas`, line 44) and `crates/core/src/services/ingest.rs` (in `parse_svg_size`, line 163). usvg 0.45's default `Options` has an empty font database (so text is dropped) and `ImageHrefResolver::default()` reads files through `resources_dir`/the working directory.
- Phase-1 experiment E8 produced a resolver that works with the locked versions (usvg/resvg 0.45.1): `usvg::ImageHrefResolver { resolve_data: usvg::ImageHrefResolver::default_data_resolver(), resolve_string: Box::new(|_href, _opts| None) }` applied through `usvg::Options { image_href_resolver: ..., ..usvg::Options::default() }`. With it no file was loaded (absolute, relative from two working directories, nested SVG) and a `data:` PNG still rendered (E9).
- Phase-1 experiment E15: a counter inside `resolve_string` counts ignored external images; a walk over the parsed tree finds text nodes (`usvg::Node::Text`).
- The application already shows advisory toasts after ingest (`push_warning_toast`; JPEG, transparency, non-square advisories in `crates/app/src/app.rs`) and runs follow-up work as `iced::Task`s with a stale-result guard by source path (see `PreviewBuilt`).
- `usvg`, `resvg`, `tiny-skia` are already direct dependencies of `logolig` core; **no new dependency is needed**.

## 3. Change scope (you may touch)

Core (`crates/core`):
- New module `src/services/svg.rs` (declare `pub mod svg;` in `services.rs`) with:
  - `pub(crate) fn options() -> usvg::Options<'static>`: the **single** options builder, with the data-URL-only resolver. Used by `rasterize_svg::rasterize_to_canvas` and `ingest::parse_svg_size` (replace both `usvg::Options::default()`), so the two call sites cannot diverge.
  - `pub fn analyze(bytes: &[u8]) -> SvgReport` and `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)] #[non_exhaustive] pub struct SvgReport { pub has_text: bool, pub ignored_external_images: usize }` (additive public API approved by the architect; `non_exhaustive` so phase 2b can add `missing_families`). `analyze` parses with an options value whose `resolve_string` counts calls (an `Arc<AtomicUsize>` captured by the closure), detects text by **counting calls of a font-selection hook** (see 5.2; the tree walk was proven impossible by the implementer's stop-and-ask report) and returns the report. Our own code in `analyze` must not `unwrap`/`expect`/index unchecked; unparsable input returns `SvgReport::default()` (the ingest and render paths report their own errors). **Do not wrap it in `catch_unwind`** (decision below).
  - Re-export `SvgReport` from `lib.rs` next to the other service types.
- `src/services/rasterize_svg.rs` and `src/services/ingest.rs`: only the options line.
- Core tests in `tests/svg.rs` (new file; **register it in `crates/core/Cargo.toml` `[[test]]`, `autotests = false`**), fixtures in `tests/fixtures.rs` (add helpers; do not change existing ones).
- i18n: `crates/core/src/message_key.rs` (new keys), `crates/i18n/src/dictionary.rs` (fields and lookup arms), `crates/i18n/locales/en.toml`, `ja.toml`, `crates/i18n/tests/i18n.rs` (add the new keys to the existing key lists).
- App (`crates/app/src/app.rs`): a new message and handler (see 5.3) and tests in the existing inline `mod tests` (known violation, RFC 017: add there, do not restructure). A small task helper in `task_queue.rs` is allowed.
- Docs: `docs/src/functional-spec.md` §15 (the sentence "resvg (uses OS fonts)" is false: replace it with the truth: SVG text is not drawn yet and external images are never read), §10 toast table (one row), `docs/src/export-spec.md` (a short "SVG sources" paragraph: embedded `data:` images are used, file references are ignored, text is not drawn yet).

## 4. Non-change scope (you must not touch)

- **Do not load any fonts** (`fontdb`, `load_system_fonts`, family repair): that is phase 2b. Text keeps rendering as nothing; the toast says so.
- `SourceAsset`, `ingest`'s return type, `SourceKind`, `ExportPlan`, persisted settings, `vectorize`, `exporter`, `canvas`, everything else in core.
- No dependency change, no `Cargo.toml` change except the `[[test]]` entry, no `Cargo.lock` change.
- The `design` feature of snora, any layout or style change, any other toast.
- `rfcs/` and `ROADMAP.md`.

## 5. Required behaviour

### 5.1 Safety
No code path reads a file named inside an SVG. Verified for both call sites (see tests T2, T3).

### 5.2 Report
`analyze(svg)` returns: `has_text` true iff the SVG contains a text element; `ignored_external_images` equals the number of non-`data:` image references that were refused (absolute path, relative path, nested SVG file all count; a `data:` image does not).
**Decided by the architect (2026-10-24, after the implementer's stop-and-ask report; option A).** usvg 0.45.1 does not keep text nodes when it cannot resolve a font (`src/parser/text.rs`: `if text::convert(..).is_none() { return; }`), so the tree never contains text in this phase. `has_text` is therefore derived from the documented `usvg::FontResolver` extension point: in `analyze` only, set `options.font_resolver.select_font` to a closure that counts its calls and returns `None`, and `select_fallback` to `usvg::FontResolver::default_fallback_selector()`; `has_text = calls > 0`. Meaning: "text that usvg would try to draw and that has content" (unused `<defs>` text and empty `<text>` do not count, which is right: the toast must not claim a loss where nothing is lost). The rendering options built by `options()` keep the default font resolver: rendering is unchanged and no font is loaded. Pin the behaviour with T6/T7 and the two extra cases below, so a future usvg that stops calling the hook fails a test at the next upgrade. No new dependency, no XML parser, no tree walk.

### 5.3 Advisory in the app
- After `IngestCompleted(Ok(asset))` for `SourceKind::Svg`, start one `Task` that runs `logolig::services::svg::analyze(&asset.raw)` off the UI thread and returns `Message::SvgAnalyzed(PathBuf, SvgReport)` (path = the asset's path).
- On `SvgAnalyzed(path, report)`: ignore it if `path` is not the current `source_asset`'s path (stale result); if `report.has_text || report.ignored_external_images > 0`, push **one** transient warning toast (`push_warning_toast`, never one per issue); otherwise nothing.
- Not for non-SVG sources; not again on `ShowLastResultRequested` or `EditCancelled`.
- Texts (architect-defined; do not reword). Title key `ToastSvgPartsTitle`; body keys `ToastSvgImagesBody` and `ToastSvgTextBody`; the toast body is the applicable body sentences joined with `\n` (images sentence first).
  - en title: `Some parts of your SVG are not shown`
  - en images: `It refers to image files on your computer. For your safety Logolig does not read them. Embed the images in the SVG to include them.`
  - en text: `Logolig does not draw text in SVG files yet. Convert the text to outlines (paths) in your design tool and load the SVG again.`
  - ja title: `SVG の一部が表示されません`
  - ja images: `SVG がこのコンピューター上の画像ファイルを参照しています。安全のため、Logolig はそれらを読み込みません。画像を SVG に埋め込むと反映されます。`
  - ja text: `Logolig は SVG 内の文字をまだ描画できません。デザインツールで文字をアウトライン化（パス化）してから、SVG を読み込み直してください。`
  Snake-case TOML keys follow the existing convention (`toast_svg_parts_title`, `toast_svg_images_body`, `toast_svg_text_body`). The ja texts have not had a native review: say so.

## 6. Required tests (write the failing ones first and record that they failed on the unmodified code)

Core, `tests/svg.rs`. Create every referenced file yourself in a temporary directory (a unique directory removed at the end; never read or reference a file you did not create; use a distinctive colour such as magenta for the "secret" PNG).

| # | Test | Assertion |
|---|---|---|
| T1 | Plain SVG (shapes only) | `analyze` = `{ has_text: false, ignored_external_images: 0 }`; rendering unchanged (byte-compare with the output before your change, captured first) |
| T2 | SVG with `<image href="<absolute path of secret.png>">` | `rasterize` output has **no** magenta pixels; `analyze.ignored_external_images == 1`. **Must fail on the unmodified code** (magenta present) |
| T3 | Same with a relative href, process working directory containing the file and not containing it | no magenta in either case; count 1. (Changing the working directory affects the whole test process: keep this test serial, in its own `#[test]`, and restore the directory; or resolve it differently if cleaner and say how.) |
| T4 | Nested SVG file reference (`href="inner.svg"`) | not loaded (no pixels of the inner SVG's colour); count 1 |
| T5 | `data:image/png;base64,...` image (generate a small PNG in the test) | renders with that image's pixels; count 0 |
| T6 | SVG with `<text>` (also inside nested groups, and drawn through `<use>`) | `has_text == true`; render is transparent as today (the test documents the current limit; phase 2b will change this assertion) |
| T7 | SVG with text **and** two external images | `has_text == true`, count 2 |
| T6b | Text that is **not** drawn: `<text>` only inside `<defs>`; an empty `<text></text>` | `has_text == false` (documents the chosen meaning) |
| T8 | Garbage bytes and an empty slice | `analyze` returns the default report, no panic |
| T9 | `ingest::ingest_bytes` for an SVG referencing a file: **the file is not opened** | Prove it (Unix: a named pipe created with `mkfifo` as the referenced "file"; a non-blocking open of its write end succeeds only if some process opened it for reading; skip with a clear message if `mkfifo` is unavailable, never silently pass). Must fail on the unmodified code |

App, in the existing inline test module: both advisory sentences present (text + images), images only, text only, none (no toast), stale path (no toast), non-SVG source (no analysis started), exactly one toast in the combined case, not repeated by `EditCancelled` + `ShowLastResultRequested`.
i18n: the three new keys exist for `en` and `ja` and are non-empty; `ja` differs from `en`.

## 7. Required evidence

1. Before any change: gates on the base commit and the byte hashes of the rendering of existing SVG fixtures (T1 oracle).
2. The failing logs of T2 and T9 on the unmodified code.
3. After: gates; `cargo audit` (expected unchanged: 0 vulnerabilities, 12 warnings; no new advisory).
4. **GUI check (rules: project GUI file):** isolated config, binary named `logolig`, one instance. Drop (a) a plain SVG: no toast; (b) an SVG with text only: the toast with the text sentence; (c) an SVG with an absolute external image reference to a magenta file you created: the toast with the images sentence and **no magenta in the result thumbnails** (a state-based check on the saved `favicon-32.png` bytes is preferred to a picture); (d) both: one toast with both sentences. At most four pictures (one per case, the toast visible), plus one in Japanese if cheap. Say which backend was used.
5. CI: record the run for the final commit and each of the five job conclusions (read the raw job log if the API shows a job as `in_progress` while the run is completed).

## 8. Acceptance criteria

T1–T9 and the app and i18n tests exist and pass, T2 and T9 shown failing before the fix; no file outside the SVG can be read by ingest or rendering; one combined toast per SVG; no font loaded; docs corrected; no dependency change; gates and five CI jobs green.

## 9. Prohibited shortcuts

- No font loading "to make text work", no new dependency (including an XML parser), no `SourceAsset`/`ingest` signature change, no opt-in setting for external images, no weakening of an assertion to pass, no ignores or `#[allow]`, no red pushes, no tags, no history rewriting.
- Do not report a test as passing that was skipped silently.

## 10. Security and compatibility constraints

- The core change is a **security hardening**: it removes a local-file read driven by untrusted input. Behaviour change for users: SVGs that relied on external image files lose those images (and are told so); this is intended and approved (RFC 015 D2).
- Persisted settings and all other output are unchanged.

## 11. Known risks

- The hook relies on usvg calling `select_font` while it converts text (observed in 0.45.1 with an empty database, pinned by `Cargo.lock`); T6, T6b and T7 guard it.
- Changing the process working directory in a test can disturb parallel tests; keep T3 isolated.
- The advisory text must stay true: when phase 2b renders text, the text sentence will change. Do not make it more general than the facts.

## 12. Required review request

`.git-exclude/review-request/rfc-015-phase-2a-svg/README.md` with everything required by section 7 of the mid-capability operating instructions, plus: the commit range; the T2/T9 failing logs; the T1 oracle hashes; the code of `options()` and `analyze`; gates, audit and CI verdicts; the GUI evidence (state first, pictures only as listed); the note about the unreviewed Japanese texts; skipped checks with reasons; deviations; open questions. Report to the owner only the path of the `README.md`.
