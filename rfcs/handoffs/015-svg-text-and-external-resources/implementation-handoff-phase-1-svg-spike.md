# Developer Handoff — RFC 015, Phase 1: SVG text and external-resource verification spike

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/`.

Governing RFC: [RFC 015](../../accepted/015-svg-text-and-external-resources.md) (Accepted, phased). This handoff covers **Phase 1 only: an investigation that produces evidence**. Its section "Decisions and phasing" is binding. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled. A claim in this area is only worth something if it was **executed**.

Dependencies: none. Independent of the other handoffs. Do not wait for them.

## 1. Purpose

The RFC's claims come from reading source, not from running anything. Confirm or refute each one by execution, measure the cost of the proposed fix, and prove that the proposed fix mechanism works with the locked dependency versions. The architect uses your evidence to finalise the phase-2 design and write its handoff.

## 2. Claims to test (from the RFC; all currently **unverified by execution**)

- **C1** `usvg::Options::default()` has an empty font database, so SVG `<text>` is dropped from the rendering produced by `logolig::services::rasterize_svg`.
- **C2** The documentation statement "resvg (uses OS fonts)" (`docs/src/functional-spec.md` §15) is therefore wrong.
- **C3** usvg's default image resolver reads files named in `<image href="...">` (absolute path; relative path resolved against the process working directory because `resources_dir` is `None`), so a local PNG/JPEG/GIF/WebP/SVG file can end up inside generated icons.
- **C4** The same happens through `ingest::parse_svg_size` (it also builds `usvg::Options::default()`), at ingest time.
- **C5** (feasibility of the fix) Loading system fonts into a shared `Arc<fontdb::Database>` makes C1 text render, and a custom `image_href_resolver` that accepts only `data:` URLs makes C3 not happen while data-URL images still render.
- **C6** (cost) `rasterize_svg::rasterize` is called once per output size, each time constructing options; loading system fonts per call would be too slow, a shared `OnceLock` is needed. Measure.

## 3. Change scope

**No production code changes.** Do all work in a throwaway Cargo project at `.git-exclude/tmp/spike-015/` (untracked, ignored by git). It is a separate workspace, so give its `Cargo.toml` its own empty `[workspace]` table (otherwise Cargo will try to attach it to the repository workspace). To get identical dependency versions, copy the repository's `Cargo.lock` into the spike directory before the first build and use `--locked`/`--offline` where possible. The spike may depend on `logolig = { path = "../../../crates/core" }` (to test current behaviour through the real code path) and on `usvg = "0.45"`, `resvg = "0.45"`, `tiny-skia = "0.11"`, plus `fontdb` at the version in `Cargo.lock` (0.23.0), `image` for decoding results. Nothing from the spike is committed to the repository.

The only repository files you may add are the review-request files described in §9 (also untracked).

## 4. Non-change scope (you must not touch)

- Everything under `crates/`, `docs/`, `rfcs/`, `packaging/`, `Cargo.toml`, `Cargo.lock`, `.github/`.
- No `cargo update`, no new versions, no changes to `.cargo/`.
- No branch, commit or push is required for this task (it produces evidence only). Do not push anything.

## 5. Required experiments (record command, input, observed result for each)

Create fixture SVGs yourself under the spike directory. Use only files you create; never point a test at personal or system files.

| # | Experiment | Expected by the RFC | Record |
|---|---|---|---|
| E1 | SVG with a large black `<text>` (for example "Aa" at font-size 100 on a transparent background), 128 × 128. Render through `logolig::services::rasterize_svg::rasterize` at 128 px using the **unmodified** code path. | C1: almost all pixels transparent | count of non-transparent pixels; save the PNG |
| E2 | Same SVG with `Options` carrying a database filled by `fontdb_mut().load_system_fonts()`, rendered with `resvg` directly | glyph pixels appear | pixel count; saved PNG; list of font families found on this machine (count and a sample); which family was used |
| E3 | E2 repeated with a font family that does not exist on the machine (`font-family="NoSuchFont"`) | fallback font is used, no error | what was drawn; does `usvg` report anything usable to detect the fallback? (inspect the `Tree`; say plainly if nothing is detectable) |
| E4 | SVG containing `<image href="/abs/path/to/secret.png">` where `secret.png` is a small, distinctively coloured PNG **you created** in a temp directory. Render through the unmodified `rasterize` | C3: the colour appears in the output | pixel colour found; saved PNG |
| E5 | Same with a **relative** href (`secret.png`) and the process started with that temp directory as its working directory, then from another directory | resolved against the working directory | both outcomes |
| E6 | Nested SVG: `<image href="inner.svg">` pointing at a file you created | loaded | outcome |
| E7 | Same as E4 but through `ingest::ingest` + `parse_svg_size`, observing whether the file is opened (use `strace -f -e trace=openat` or equivalent; if `strace` is not installed, say so) | C4 | the `openat` lines for the fixture file |
| E8 | Custom `usvg::Options` with `image_href_resolver = ImageHrefResolver { resolve_data: <default data resolver>, resolve_string: Box::new(|_, _| None) }` | C5: E4/E5/E6 fixtures no longer load the files; no panic | outcomes; the exact code that worked (paste it, it becomes the phase-2 design input) |
| E9 | SVG with an `<image href="data:image/png;base64,...">` (embed a small PNG you generate), rendered with the E8 options | still renders | pixel colour found |
| E10 | A mixed realistic fixture (a logo-like path plus a text label) at sizes 16, 32, 48, 180, 192, 512, rendered with a **shared** database built once versus options+database **rebuilt per call** | C6: per-call is far slower | timings for `load_system_fonts()` (first call, cold and warm), and total time for the six renders in both modes, release build, on this machine; state CPU/OS |
| E11 | Output stability: render the E2 fixture twice in two separate processes | identical bytes (same fonts) | byte equality; note anything non-deterministic |
| E12 | Behaviour when the machine has **no** fonts: run E2 with `FONTCONFIG_FILE` pointing at an empty config (or an empty `fontdb`) | text missing, no crash | outcome |

For each experiment also record: what you expected, what happened, and whether the RFC claim is **confirmed**, **refuted** or **inconclusive**.

## 6. Required validation

No production code is changed, so the project gates are not required. State explicitly in the review request: "Gates not run; no repository code changed." Do run `git status --short` at the end and include its output to prove the repository is unchanged (apart from untracked review artefacts).

## 7. Acceptance criteria

- Every experiment in §5 has a recorded result, or a stated reason it could not be run and what risk remains.
- For each of C1–C6 a verdict: confirmed, refuted or inconclusive, with the evidence reference.
- The working E8 resolver code and the shared-database construction code are included verbatim.
- A short list of **findings beyond the RFC** (anything surprising: for example `usvg` features, text-to-path behaviour, `fontdb` scan time, panics, memory use).
- A recommendation for the phase-2 design **as options with trade-offs**, not as a decision (the architect decides). In particular say how core could cheaply report to the app that (a) external images were ignored and (b) the SVG contains text, without changing `SourceAsset` (alternatives: a separate analysis function in core; the ingest result). Do not implement it.

## 8. Prohibited shortcuts and constraints

- Do not infer a result from reading source when an experiment is specified; execute it. Mark inferences as inferences.
- Do not adjust a fixture after seeing the output to make a claim come out true; keep the original fixture and record the variants separately.
- Do not read, embed or print any file you did not create. No network access beyond what Cargo needs from its existing cache.
- Do not leave test images containing personal data; scrub paths and the user name from logs placed in the review request.
- Do not change repository files or push.

## 9. Required review request

In `.git-exclude/review-request/rfc-015-phase-1-spike/` with a `README.md` entry point, containing everything required by section 7 of the mid-capability operating instructions (adapted to an investigation): task and RFC identifier; handoff; summary of verdicts C1–C6; the experiment table with results; saved images (before/after); commands executed; timings; the E8 resolver code; findings beyond the RFC; the phase-2 option analysis; skipped or inconclusive items with reasons; the `git status` proof; and open questions for the architect. Report to the owner only the path of the `README.md`.
