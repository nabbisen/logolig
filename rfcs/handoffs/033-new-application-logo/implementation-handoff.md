# Developer Handoff — RFC 033: adopt the new application logo (job J7f)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (English only; relative paths with `cd`).

Governing RFC: [RFC 033](../../accepted/033-new-application-logo.md) (Accepted, owner request 2026-11-10). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request.

**Workflow (owner decision 2026-10-21): no topic branches, no concurrent jobs.** You are the only job running. Work on `main`: `git pull --ff-only` first, small commits, gates chained with `&&`, never a red push, no tags. **Position in the schedule:** job J7f in `ROADMAP.md`, after J7e (the lock hygiene) is reviewed. This job changes no code under `crates/`.

## 0. Preconditions (check first; if one fails, stop and report, change nothing)

1. `.git-exclude/tmp/app-logo-261110/logolig-app-logo-master-rgba.png` exists, is PNG 1254 × 1254 **RGBA**, sha256 starts `987eb24a74be9212…`. Read it: pixel `(0,0)` must have alpha 0 and pixel `(627,627)` alpha 255 (for example `magick <file> -format '%[pixel:p{0,0}] %[pixel:p{627,627}]\n' info:`).
2. `.git-exclude/tmp/app-logo-261110/provenance.md` exists (the owner's own statement). If it does not, **stop**: the owner writes it, nobody else.
3. `.git-exclude/tmp/app-logo-261110/derive-alpha.sh` exists (documents how the alpha channel was made).
4. The old files are the ones the RFC names: `packaging/icons/logolig.svg`, `packaging/linux/icons/hicolor/{48x48,64x64,128x128,256x256}/apps/io.github.nabbisen.Logolig.png`, `packaging/linux/icons/hicolor/scalable/apps/io.github.nabbisen.Logolig.svg`, the four tiles in `packaging/windows/Assets/`.

## 1. Scope (you may touch)

- New: `packaging/icons/logolig.png` (the master, **byte-identical copy** of the file in precondition 1), `docs/assets/logolig-logo.png` (256 px, derived).
- Replaced: the Linux hicolor PNGs and the four Microsoft tiles (below); `packaging/icons/README.md` (rewritten); `packaging/README.md` (the two table rows that name the icons and the master); `.github/workflows/packaging.yml` (**only** the expected icon lines of the exact file list); `README.md` (the logo at the top).
- Removed: `packaging/icons/logolig.svg` and `packaging/linux/icons/hicolor/scalable/` (the old stand-in design must not stay anywhere; icon themes prefer a scalable SVG over PNGs).
- Not touched: `crates/`, `Cargo.*`, `AppxManifest.xml`, `PKGBUILD.in` (it copies the whole hicolor tree; if you find it needs a change, stop and ask), `docs/assets/screenshot-01.png`, `rfcs/`, `ROADMAP.md`.

## 2. Required implementation

1. **Derived images, from the master only, Lanczos3, RGBA with transparent corners:**
   - Linux: `hicolor/{32x32,48x48,64x64,128x128,256x256,512x512}/apps/io.github.nabbisen.Logolig.png` (square, exact sizes).
   - Microsoft Store: `StoreLogo.png` 50 × 50, `Square44x44Logo.png` 44 × 44, `Square150x150Logo.png` 150 × 150, `Wide310x150Logo.png` 310 × 150 (the logo contain-fit and centred on a transparent canvas, as Logolig's own wide-logo output does).
   - README image: `docs/assets/logolig-logo.png`, 256 × 256.
   Preferred tool: **Logolig itself** (the same resampling the product uses): the core crate's exporter from a small scratch program **outside the repository** (not committed), or the application with one run (Customize: PNG sizes, Microsoft app logos on). If you use something else (ImageMagick `-filter Lanczos -resize`), say so and why in the review request. Whichever you use, the master and the outputs are compared in §3.
2. `packaging/icons/README.md`: (a) heading **Provenance and licence**: the owner's `provenance.md` text, **verbatim**, no edits, no additions in that section; (b) heading **Technical derivation**: the owner's file `logolig-app-logo-base.png` (sha256 `53718618b22e…`) was RGB with opaque black outside the rounded square; the architect derived the alpha channel with the method in the header of `derive-alpha.sh` (copy that header here, and say that every opaque pixel is byte-identical to the owner's file); master sha256; (c) how every derived file is produced (the exact commands or settings you used) and the list of derived files and sizes; (d) the old stand-in was removed (history keeps it).
3. `README.md`: directly under the `# Logolig` title, before the badges, a centred logo: `<p align="center"><img src="docs/assets/logolig-logo.png" alt="Logolig logo" width="128" height="128"></p>`. All three crates use this README (`readme = "../../README.md"`): check how crates.io resolves a relative image path for a README taken from outside the crate directory (read crates.io's documented behaviour; say what you found). If a relative path will not render there, tell the architect **before** pushing; do not switch to an absolute URL on your own.
4. `.github/workflows/packaging.yml`: in the exact expected file list replace the four old icon lines and the `scalable` line by the six new icon lines, sorted as the list is sorted. Nothing else in the workflow changes.
5. `packaging/README.md`: the row for `linux/icons/hicolor/` says PNG 32, 48, 64, 128, 256 and 512; the row for `icons/logolig.svg` becomes `icons/logolig.png` (master artwork).
6. `grep -rn "logolig.svg\|scalable" packaging .github scripts README.md docs` must return nothing that refers to the removed files (history-only mentions in `rfcs/` are fine).

## 3. Required evidence

- **Pixel checks by machine, not by eye** (a table in the review request, one row per file): size; has alpha; pixel `(0,0)` alpha is 0; centre pixel alpha 255. For the 50/44 px tiles also check that the corner pixel is transparent.
- **Fidelity:** the 512 px hicolor icon equals the master resized to 512 (a pixel-difference count of 0 against a fresh ImageMagick/Lanczos or Logolig resize, or say what differs).
- **Visual check by you** at actual size on a light and on a dark background, for 32, 48 and 64 px and the 44/50 px tiles: say in words whether the mark is recognisable (the architect looks at them as well). Create the comparison images in your scratch directory, not in the repository.
- **Packaging, as in J5/J6:** `git archive --format=tar.gz --prefix=logolig-<version>/ HEAD` of the final tree, `scripts/render-aur.sh logolig <version> <dir> --tarball <archive>`, `makepkg -f --noconfirm` as a normal user, `namcap` (PKGBUILD and package: no `E:`), `pacman -Qlp`: the file list equals the new list of §2.4. Not installed.
- **CI** on the pushed commit: `fmt`, `clippy`, `test`, `msrv`, `audit` and the Packaging workflow (it starts by itself on changes under `packaging/`; if it does not, dispatch it): job `aur-source` success, by name.
- Gates (`fmt`, `clippy -D warnings`, `test`, `+1.90 check`, `audit`) are expected unchanged (nothing in `crates/` changes); run them chained and say so.
- No GUI evidence is required unless you used the application to make the images (then follow `.git-exclude/rules/project-instructions-gui-common.md`).

## 4. Acceptance criteria

All files exist with the exact sizes and transparent corners; no reference to the old SVG remains; packaging job and local `makepkg` pass with the new exact list; the README starts with the logo; the provenance section equals the owner's text word for word; no change outside §1.

## 5. Prohibited

No edit of the owner's text, no invented provenance or licence wording, no change to `AppxManifest.xml` (not even the version), no code change, no `unsafe`, no tag or release, no force-push, no committing of scratch programs or comparison images, no absolute-URL workaround for the README image.

## 6. Notes for the evidence of later jobs

`packaging/windows/Assets/Square150x150Logo.png` becomes the new art when this lands. Earlier handoffs use it as a convenient input image; comparisons are always base-versus-new inside one job, so they stay valid, but baselines taken before this job are not comparable with captures after it.

## 7. Required review request

`.git-exclude/review-request/rfc-033-new-application-logo/README.md` with everything required by section 7 of the mid-capability operating instructions, plus the pixel-check table, the fidelity result, the visual notes, the packaging and CI results, what you found about crates.io and the README image, skipped checks, deviations and open questions. Report to the owner only the path of that `README.md`.
