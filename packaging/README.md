# Packaging

What is here and how it is used. Publication to the AUR is **an owner action** (RFC 020, phase 3): nothing in this repository, and no CI job, pushes to the AUR.

| Path | Purpose |
|---|---|
| `aur/logolig/PKGBUILD.in` | Template of the AUR source package `logolig` (builds the GUI binary from the tagged source) |
| `linux/io.github.nabbisen.Logolig.desktop` | Desktop entry; the file name is the application ID the window reports |
| `linux/icons/hicolor/` | Application icons installed under `/usr/share/icons/hicolor` (PNG 48, 64, 128, 256 and the SVG) |
| `icons/logolig.svg` | Master artwork; every icon is generated from it (see `icons/README.md`) |
| `windows/` | Microsoft Store manifest and assets (manual, unrelated to the AUR) |
| `../scripts/render-aur.sh` | Fills the template and writes `PKGBUILD` and `.SRCINFO` |

## Render and test locally

```sh
# Real release: downloads the GitHub tag archive of <version> and hashes it.
scripts/render-aur.sh logolig <version> <outdir>

# Dry run: use a local archive instead (the tag does not need to exist yet).
git ls-files -z -co --exclude-standard \
  | tar --null -T - --transform 's,^,logolig-0.0.0/,' -czf /tmp/logolig-0.0.0.tar.gz
scripts/render-aur.sh logolig 0.0.0 /tmp/render --tarball /tmp/logolig-0.0.0.tar.gz

cd /tmp/render
makepkg -f --syncdeps --noconfirm      # builds, runs the tests of the GUI-free crates, packages
namcap PKGBUILD && namcap ./*.pkg.tar.zst
```

Prefer a clean chroot (`extra-x86_64-build` from `devtools`, as a user with sudo) over building in your own session. `pkgver` must not contain a hyphen; `0.0.0` cannot collide with a release. The script refuses to write into a directory under `.git-exclude/tmp/aur-repositories/` or into a repository whose remote is `aur.archlinux.org`.

The workflow `.github/workflows/packaging.yml` does the same in an `archlinux:base-devel` container (pinned by digest) for every change under `packaging/` or to the script, and fails on a `namcap` error or an unexpected file list. It reads no secret and pushes nothing.

## The first release

The first AUR version must be a release whose tag archive **already contains `packaging/linux/`** (that is, a release made after this packaging landed): the PKGBUILD installs the desktop entry and icons from the source archive and fails loudly if they are missing. `pkgver` must equal the tag (tags are `X.Y.Z`, no `v`). Choosing and creating that release is a separate owner decision.

## Publish by hand (owner, phase 3)

1. `scripts/render-aur.sh logolig <version> /tmp/aur-render` (this downloads the published tag archive and hashes it).
2. Build and `namcap` it as above.
3. In the local AUR clone: copy `PKGBUILD` and `.SRCINFO` from the render directory, check `git diff`, commit, and `git push`.
4. Later versions: render again with the new version; `pkgrel` goes back to 1 for a new `pkgver`.

## Rollback

Never force-push an AUR repository. A bad package is superseded: publish a fix with a higher `pkgrel` (same source) or a new `pkgver`. Users then update normally; deleting history is not possible and not needed.

## Dependencies

The `depends` list of the PKGBUILD is derived from evidence (the packaging review request): the binary links only `glibc` and `libgcc`; the windowing, keyboard and graphics libraries are loaded at run time and are invisible to `namcap`, so they are listed explicitly; text needs a system font (`ttf-font`); the file chooser needs `xdg-desktop-portal` **and** a portal backend (`optdepends`: without a portal service, "Choose file…" does nothing and shows no message).
