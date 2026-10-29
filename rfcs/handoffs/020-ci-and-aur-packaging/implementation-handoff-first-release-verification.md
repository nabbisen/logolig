# Developer Handoff — RFC 020 first release: verify the AUR package from the real tag archive (job J7a)

Same roles, rules and workflow as the earlier handoffs (`.git-exclude/roles/`, `.git-exclude/rules/`; GUI rules in `.git-exclude/rules/project-instructions-gui-common.md`). Directly on `main`, `git pull --ff-only` first. **You do not push to the AUR, do not write into `.git-exclude/tmp/aur-repositories/`, do not install the package (`pacman -U` is prohibited), do not tag or edit releases, and do not edit `rfcs/` or `ROADMAP.md`.** Governing text: [RFC 020](../../accepted/020-ci-and-aur-packaging.md) phase 3 (first publication is a manual owner action) and `packaging/README.md`.

## Context

Release 1.27.0 is tagged (signed tag `1.27.0` on commit `49164e1`) and the GitHub Release exists. The PKGBUILD has so far been built only from local archives of the working tree. The AUR package must be built from the archive GitHub serves for the tag, with the hash `scripts/render-aur.sh` computes.

## Required work

1. `git fetch --tags && git pull --ff-only`; confirm `git rev-parse 1.27.0^{commit}` is `49164e1…` and that `packaging/aur/logolig/PKGBUILD.in`, `packaging/linux/` and `Cargo.lock` are in that commit.
2. Render from the **real tag archive** (no `--tarball`): `scripts/render-aur.sh logolig 1.27.0 <outdir>` with `<outdir>` in your scratch space (never under the AUR clones). Record the SHA-256 it prints and check it **independently** twice, a minute apart: `curl -L -s https://github.com/nabbisen/logolig/archive/refs/tags/1.27.0.tar.gz | sha256sum` (GitHub's generated archives are normally stable, but this is exactly what the AUR package will rely on).
3. Build: `makepkg -f --noconfirm` in `<outdir>` (host as a normal user is acceptable if a chroot is not possible: say which). Then `namcap PKGBUILD`, `namcap` on the package (no `E:`), `pacman -Qlp` (the exact expected file list as in `.github/workflows/packaging.yml`), `ldd` (no missing library), `.SRCINFO` equals `makepkg --printsrcinfo`.
4. Compare the rendered `PKGBUILD` with the one CI and the candidate build used: the only differences must be `pkgver`, `source` and `sha256sums`. Show the `diff`.
5. One proportionate smoke check of the **packaged** binary (extracted from the package into your scratch space, run from there; native Wayland, `wtype`; isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME`; **never start it without them**; pid focus check before keystrokes; no resizing): it opens with `app_id io.github.nabbisen.Logolig`, Ctrl+O opens the chooser, a PNG gives the Result screen. One capture per question at most; state the window size.
6. Prepare, **inside your scratch space only**, the exact two files the owner will publish (`PKGBUILD`, `.SRCINFO`) and list the commands from `packaging/README.md` "Publish by hand" with the real paths. Do not copy them into the AUR clone.

## Evidence and delivery

Commands and outputs (the hash checks, the build log tail, `namcap`, file list, diff, smoke record), skipped checks, deviations, open questions. Review request at `.git-exclude/review-request/rfc-020-first-release-verification/README.md`. If anything differs from the CI/candidate result, **stop and report**; do not fix by editing the template without the architect.
