# RFC 033 — Adopt the new application logo (documentation, Linux package, Microsoft Store)

**Status.** Proposed (the goal was requested by the owner on 2026-11-10; accepted when the owner supplies a master with transparent corners and the provenance statement, see "Needs from the owner")
**Priority.** P2 (visible identity of the first releases; no behaviour change)
**Tracks.** Owner request 2026-11-10: use the renewed logo "for documentation and publishment including Microsoft Store". Files: `.git-exclude/tmp/app-logo-261110/` (`logolig-app-logo-base.png` and a favicon bundle made with Logolig).
**Touches.** `packaging/icons/`, `packaging/linux/icons/hicolor/`, `packaging/windows/Assets/`, `packaging/README.md`, `.github/workflows/packaging.yml` (the expected file list), `README.md`, `docs/assets/`. **No** code under `crates/`.

## Summary

The repository carries an AI-derived stand-in icon (`packaging/icons/logolig.svg`, a flat re-interpretation of the old Store tile). The owner has designed the real logo. This RFC replaces the stand-in everywhere it is published, from one master, without leaving the old design visible anywhere.

## Evidence (architect, 2026-11-10; pixels read directly)

- `logolig-app-logo-base.png`: PNG 1254 × 1254, **8-bit RGB, no alpha channel**. The area outside the rounded square is **opaque black** (corner pixels `(1,1,1)` and `(0,0,0)`), not transparent.
- The bundle made from it (`favicon-512.png`, `favicon-192.png`, `favicon-32.png`, `apple-touch-icon.png`) is RGBA, but it has **0 transparent pixels** (the 512 px file: 0 of 262 144; corners `(0,0,0,255)`). The black corners were carried through. On a light page, a launcher or a Store tile they show as black corner blocks around the rounded icon. The application did not add them; the input had none.
- `favicon.svg` in the bundle is a **trace** (vtracer, 28 paths, a full 512 × 512 background rectangle `#2F19BA`): the soft gradients of the design become flat bands and the black corners are in it. It is a useful web favicon only if the corners are fixed; it is **not** a master and must not become the Linux `scalable` icon.
- Current published identity: `packaging/icons/logolig.svg` is the master of the Linux hicolor PNGs (48, 64, 128, 256) **and** the `scalable` SVG; the four Store tiles in `packaging/windows/Assets/` are the old hexagon art. The packaging workflow checks an exact file list that includes the `scalable` SVG. Icon themes prefer a `scalable` SVG over PNGs, so if the old SVG stayed, desktops would keep showing the old logo.
- `packaging/windows/Assets/Square150x150Logo.png` is also used by the GUI evidence of earlier jobs as a convenient input image; replacing it changes that input (see "Consequences").

## Needs from the owner (blocks acceptance)

1. **A master with transparent corners.** Best: the PNG exported as RGBA (alpha outside the rounded square, anti-aliased edge), at the same size or larger; or a vector (SVG) original. If neither is possible, tell the architect: the fallback is to derive an alpha mask from the shape in this repository, which is feasible but leaves the risk of a thin dark fringe on the edge, and is worse than a proper export ("clean, safe and secure").
2. **Provenance and licence of the artwork**, in the owner's words, for `packaging/icons/README.md`: who made it, whether any part is AI-assisted (Flathub asks for disclosure, RFC 023), and under which terms the logo is offered (the code is Apache-2.0; a logo is often kept under separate terms such as "all rights reserved, usable to identify the project"). The architect will not invent this statement.

## Requirements

1. **One master.** The owner's file is stored unchanged in resolution, as `packaging/icons/logolig.png` (RGBA, ≥ 1024 px; or `logolig.svg` if a vector is supplied, in which case the scalable icon stays). Every other file is derived from it; `packaging/icons/README.md` says how (command or Logolig settings), the provenance (above), and that the old stand-in is gone (history keeps it).
2. **Linux package.** Hicolor PNGs **32, 48, 64, 128, 256 and 512** under `packaging/linux/icons/hicolor/<N>x<N>/apps/io.github.nabbisen.Logolig.png`, generated with Logolig (Lanczos3), RGBA with transparent corners. **The `scalable` SVG and the old `logolig.svg` are removed** (raster master). `packaging.yml`'s expected file list, `packaging/README.md` and the templates are updated to the new set; `PKGBUILD.in` already copies the whole tree and needs no change, but the packaging job must pass (`makepkg` with tests, `namcap` without `E:`, exact list). A vector can replace the PNG-only state later without another design change.
3. **Microsoft Store.** The four tiles `StoreLogo.png` 50 × 50, `Square44x44Logo.png` 44 × 44, `Square150x150Logo.png` 150 × 150, `Wide310x150Logo.png` 310 × 150 are regenerated from the master with Logolig's own Microsoft app-logos feature (the wide tile contain-fit, transparent padding), RGBA, transparent outside the rounded square. `AppxManifest.xml` is **not** changed (names stay; the version stays as the owner decided). The architect looks at each tile at actual size and at 200 %.
4. **README.** The logo at the top of `README.md` (centred, about 128 px wide, with alt text "Logolig logo"), from a derived PNG in `docs/assets/`. It must render on GitHub and on crates.io (relative image path; the dev checks how the three crates' `readme` fields resolve it and reports).
5. **Nothing else changes.** No code, no window icon (see "Not in scope"), no screenshot, no web favicon bundle in the repository.
6. **Release.** The shipped tag `1.27.0` keeps the old icon (immutable, verified). The new logo ships with 1.28.0; the release notes of 1.28.0 say so in one line. A first AUR publication of 1.27.0 therefore shows the old icon; that is expected.

## Consequences

- Later GUI evidence that uses `packaging/windows/Assets/Square150x150Logo.png` as input uses the new art from the day this lands. Comparisons are always base-versus-new inside one job with the same file, so they stay valid; baselines taken **before** this job are not comparable with captures after it. Handoffs written after this RFC name the input explicitly.
- Repository size grows by the master (about 1.4 MB for the current file).

## Not in scope (offered, owner decides)

- A **window icon** inside the application (taskbar/title bar on Windows and X11; Wayland takes the icon from the desktop entry). Small (`window::Settings::icon`, an embedded 256 px PNG) but it is a code change and a binary-size addition; separate RFC if wanted.
- A **Windows `.exe` icon resource** (needs a build-time step on Windows).
- **GitHub social preview** (1280 × 640) and **Microsoft Store listing images** (those are uploaded in the owner's accounts; the architect can prepare files in the sizes the Store page asks for if the owner wants them).
- The **web favicon bundle** from the owner's folder: for the owner's own sites, not part of this repository.

## Acceptance criteria

All derived files exist with the stated sizes and an alpha channel whose corners are transparent (checked by a pixel read, not by eye); no reference to the old `logolig.svg` remains; the packaging CI job is green with the new exact list and `namcap` has no `E:`; the four tiles pass the size check and the architect's visual check; the README shows the logo on GitHub; the provenance file states what the owner stated.

## Risks

| Risk | Handling |
|---|---|
| Black corners reach users | The master must have alpha; acceptance is a pixel check |
| Gradient detail is lost at 32–48 px | The architect reviews each size at actual size; sizes below 32 are not shipped |
| Old design stays visible somewhere (scalable SVG, Store tile) | The old SVG is deleted; acceptance greps for it |
| Owner wants a vector later | Allowed: the `scalable` icon returns without redesign |
