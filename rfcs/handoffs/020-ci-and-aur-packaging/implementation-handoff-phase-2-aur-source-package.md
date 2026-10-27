# Developer Handoff — RFC 020, Phase 2: AUR source package `logolig` (templates, desktop entry, local dry run)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/`.

Governing RFC: [RFC 020](../../accepted/020-ci-and-aur-packaging.md) (Accepted, phased). This handoff covers **Phase 2, source package `logolig` only**. The RFC defines what and why; this document defines how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. For a package that means: it builds from verifiable source, installs exactly what it says, works out of the box, and nothing is published without a human.

Dependencies:
- the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) (CI gates) should be merged before you do §5.8. Everything else can start now.
- **Blocked on an owner action (RFC 020 D5): master icon artwork.** See §5.5. Do not invent artwork and do not upscale the 150 × 150 Microsoft tile.

## 1. Purpose

Produce a correct, validated `PKGBUILD` template for the AUR package `logolig` (builds the GUI binary from the tagged source), a desktop entry, a render script, and evidence from a clean-environment build, so that the owner can publish the first version by hand (Phase 3) with confidence.

## 2. Background (verified facts)

- AUR clones exist locally (untracked, owner-created): `.git-exclude/tmp/aur-repositories/logolig` and `.../logolig-bin`, remote `ssh://aur@aur.archlinux.org/<name>.git`. Each holds one placeholder commit: `pkgver=0.0.0`, `sha256sums=('DUMMY-FOR-INITIALIZATION')`, `package()` that only echoes, `depends=(gcc-libs glibc fontconfig libxkbcommon vulkan-icd-loader)`, no `makedepends`. **Read them; never push from them and never modify them.**
- The application binary is `logolig`, built by the package `logolig-app` (`crates/app/Cargo.toml`, `[[bin]] name = "logolig"`). Workspace `rust-version = "1.85"`, `Cargo.lock` committed, license Apache-2.0, repository `https://github.com/nabbisen/logolig`.
- Tags are `X.Y.Z` without `v` (for example `1.26.2`). GitHub tag archives extract to `logolig-<tag>/` (repository name plus tag).
- **The existing tag `1.26.2` contains no Linux packaging files** (`git ls-tree -r 1.26.2` shows only `packaging/windows/`). Anything you add now exists only from the next release on. Therefore the dry run **must not** use the remote 1.26.2 archive; it uses a local archive of your working tree (see §5.3). The first published AUR version will be a new release that contains your files; do not tag or release anything yourself.
- `rfd = "0.15"` is used with default features. Verified in the dependency's `Cargo.toml` (rfd 0.15.4): default features are `xdg-portal` and `async-std`. So on Linux the file chooser talks to the **XDG desktop portal over D-Bus**. If no portal service is running, the primary action ("Choose file…" and the Save dialogs) may fail or do nothing. This has **not** been tested.
- GUI dependencies loaded at run time through `dlopen` (winit/wgpu: Wayland, xkbcommon, X11/xcb, Vulkan/GL) are **invisible to `namcap`** and must be reasoned about explicitly.
- Tools on the maintainer workstation: `makepkg`, `namcap`, `cargo-audit`, `cargo-deny`. Whether `devtools` (clean chroot), `podman` or `docker`, `desktop-file-utils`, `xorg-server-xvfb` exist is **unknown**; check and report.

## 3. Change scope (you may add)

- `packaging/aur/logolig/PKGBUILD.in`
- `packaging/linux/io.github.nabbisen.Logolig.desktop` (**renamed 2026-10-21**: the application ID is the file name; see RFC 020 "Naming". The AUR itself does not require this.)
- `packaging/README.md` (release and rollback procedure for packagers; see §5.6)
- `scripts/render-aur.sh` (render and validate helper)
- `.github/workflows/packaging.yml` (§5.8, only after the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) is merged)
- Icon files under `packaging/linux/icons/hicolor/<size>x<size>/apps/io.github.nabbisen.Logolig.png` (and `scalable/apps/io.github.nabbisen.Logolig.svg` if the master is SVG) **only after** the owner supplies the master artwork (§5.5).

## 4. Non-change scope (you must not touch)

- `crates/`, `Cargo.toml`, `Cargo.lock`, `docs/`, `rfcs/`, root `README.md` (the install section is updated **after** publication, so that the README never announces a package that does not exist).
- `logolig-bin`: no template, no workflow, no release workflow (a later handoff).
- `ci.yml` from the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) (add a separate workflow file instead).
- The AUR clones in `.git-exclude/tmp/aur-repositories/`: read only. No `git push`, `git remote` changes, or `ssh` to `aur.archlinux.org`.
- No secrets, no repository settings, no tags, no GitHub Releases.
- Do not install the built package on the host with `pacman -U`.

## 5. Required implementation

### 5.1 `packaging/aur/logolig/PKGBUILD.in`

A template with exactly these placeholders: `@PKGVER@`, `@SOURCE_URL@`, `@SHA256@`. Follow the Arch packaging and Rust package guidelines.

- `pkgname=logolig`, `pkgrel=1`, `arch=('x86_64')`, `url='https://github.com/nabbisen/logolig'`, `license=('Apache-2.0')`.
- `pkgdesc`: `Local-first favicon and logo asset generator (offline GUI)`. No claims beyond what the program does (no mention of formats it does not support).
- `source=("$pkgname-$pkgver.tar.gz::@SOURCE_URL@")`, `sha256sums=('@SHA256@')`. The default `@SOURCE_URL@` for real releases is `https://github.com/nabbisen/logolig/archive/refs/tags/$pkgver.tar.gz`.
- `makedepends`: `cargo` and whatever the build needs (determine by building in a clean environment; do not copy from the placeholder).
- `prepare()`: `cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"`.
- `build()`: `export RUSTUP_TOOLCHAIN=stable CARGO_TARGET_DIR=target` and `cargo build --frozen --release -p logolig-app`. Network access must not be needed in `build()`.
- `check()`: `cargo test --frozen -p logolig -p logolig-i18n` (the GUI-free crates; fast, no display). Do **not** run the whole workspace test suite here (compiles the GUI in debug mode, long and pointless for packaging).
- `package()`: install `target/release/logolig` to `/usr/bin/logolig` (mode 755), the desktop file to `/usr/share/applications/io.github.nabbisen.Logolig.desktop` (644), and the whole `packaging/linux/icons/hicolor` tree to `/usr/share/icons/hicolor` (644). The icon install must **fail loudly** if the directory is missing; no `|| true`, no silent skip.
- License file: follow the current Arch guideline for SPDX common licenses (Apache-2.0 is a common license, which is normally **not** copied into the package). Check what `namcap` says and follow the guideline; if the guideline and `namcap` disagree, report instead of choosing.
- `depends`: derive from evidence, not from the placeholder. Minimum evidence required for each entry: `ldd` of the built binary in the clean environment, plus an analysis of `dlopen`-loaded libraries (see §5.4). **Include `xdg-desktop-portal`** in `depends` (decision by the architect: the file chooser is the primary action and must work out of the box) unless your test in §5.4 shows the application does not need it, in which case report and stop. List portal backends and Vulkan drivers in `optdepends` with a short reason each.
- No `options` or flags that weaken the build (no `!strip`, no disabled LTO unless a documented reason exists; state any you add).

### 5.2 `packaging/linux/io.github.nabbisen.Logolig.desktop`

Freedesktop Desktop Entry (validate with `desktop-file-validate`):

```
[Desktop Entry]
Type=Application
Name=Logolig
Comment=Make favicons and app logos from one image, offline
Comment[ja]=1枚の画像から favicon とアプリ用ロゴを作成（オフライン）
Exec=logolig
Icon=io.github.nabbisen.Logolig
Terminal=false
Categories=Graphics;
```

The wording is fixed by the architect (short, no marketing). If `desktop-file-validate` rejects something, report instead of rewording. Do not add `MimeType` or `Keywords`. `StartupWMClass` is added only if the RFC 025 job (which runs before this one) recorded an X11 window class that differs from the application ID; read its review request, and if there is none, do not guess one. The window must report the ID `io.github.nabbisen.Logolig` (RFC 025); verify it with `niri msg windows` on the built binary and report a mismatch instead of fixing it here. The `Exec` has no `%f` because the application does not accept a file argument (verify in `crates/app/src/main.rs`).

### 5.3 `scripts/render-aur.sh`

Bash, `set -euo pipefail`, passes `shellcheck` without warnings (if `shellcheck` is absent, say so). Usage:

```
scripts/render-aur.sh logolig <version> <outdir> [--tarball <path>]
```

- Reads `packaging/aur/logolig/PKGBUILD.in`, substitutes the placeholders, writes `<outdir>/PKGBUILD`, and generates `<outdir>/.SRCINFO` with `makepkg --printsrcinfo`.
- Without `--tarball`: `@SOURCE_URL@` is the GitHub tag archive URL, and `@SHA256@` is the SHA-256 of the **downloaded** archive (download to a temp file, hash it, verify non-empty; fail on HTTP errors; never accept an unverified hash).
- With `--tarball <path>`: copies the tarball into `<outdir>`, sets `@SOURCE_URL@` to a `file://` URL, and hashes that file. Used for dry runs.
- Refuses to run if `<outdir>` is inside `.git-exclude/tmp/aur-repositories/` or is a git repository whose remote is `aur.archlinux.org` (never write into the AUR clones).
- Fails loudly for unknown arguments and unreplaced placeholders (`grep -n '@[A-Z0-9_]*@'` on the output must be empty).
- Never performs `git push` or any network write.

Dry-run archive: create it from the working tree **including your uncommitted packaging files** under the exact prefix the real tag archive would have, for example `git archive --format=tar.gz --prefix=logolig-<ver>/ -o <path> <tree>` after staging or via `git stash create`; document the exact method you used. For the dry run use `pkgver` `0.0.0` (a version that cannot collide with a real release; `pkgver` must not contain hyphens) with the local archive, and say so in the review request. The dry-run archive must contain `Cargo.lock`.

### 5.4 Clean-environment build and validation (mandatory evidence)

Prefer, in this order: a clean chroot (`devtools`: `extra-x86_64-build`), a container (`archlinux:base-devel` with podman or docker), then the host as a last resort (state the limitation if used). In the clean environment run, as a non-root user:

1. `makepkg -f --syncdeps --noconfirm` on the rendered dry-run PKGBUILD. Record success, build time and the size of the package.
2. `namcap PKGBUILD` and `namcap <package>`: record every message; resolve errors; explain warnings you leave.
3. `desktop-file-validate packaging/linux/io.github.nabbisen.Logolig.desktop`.
4. `ldd /usr/bin/logolig` inside the environment: list unresolved libraries (there must be none).
5. **`dlopen` analysis.** Determine which shared libraries the binary may load at run time (inspect with `strings`/`readelf` on the binary and the crates `winit`, `wgpu`, `ashpd`/`zbus`, `rfd`, `fontdb` as locked in `Cargo.lock`). Produce a table: library, loaded by, needed for (Wayland, X11, Vulkan, GL, fontconfig, D-Bus), package that provides it on Arch, proposed dependency type (`depends` / `optdepends`).
6. **Launch test on the host session** (the maintainer uses niri on Wayland; follow `.git-exclude/rules/project-instructions-gui-common.md`): run the built binary from the package's `pkg/` directory (not installed), confirm the window opens, choose a file through the file chooser, and capture a screenshot, all under the **updated GUI rules** (`.git-exclude/rules/project-instructions-gui-common.md`, revised 2026-10-25, authoritative): isolate `XDG_CONFIG_HOME` **and** `XDG_STATE_HOME` under a scratch directory; do not resize, float or move a running window (state the size the window opened with); capture with `niri msg action screenshot-window --id <id> --path <ABSOLUTE path under the project root>` (niri rejects relative paths); never ImageMagick `import`; before every `wtype` keystroke confirm niri's focused window belongs to the instance under test (`niri msg -j focused-window`, `pid`). If a rule cannot be met by this application (for example AT-SPI actions: iced 0.14 exposes none), say so in the review request instead of improvising.
7. **Portal test (must report even if inconclusive).** In a session or container where `xdg-desktop-portal` is **not** running, click "Choose file…" and record what the user would see (nothing, an error toast, a crash). This decides whether the `depends` entry in §5.1 is sufficient and whether a follow-up UX RFC is needed. Do not change application code; report.
8. If an X11 test is possible (Xvfb or a nested server), run the binary for 10 seconds and record whether it stays alive and what it prints to stderr. State the limits of the evidence (no GPU, software rendering).

### 5.5 Icon (blocked on the owner)

RFC 020 D5: the repository only has the 150 × 150 Microsoft tile (`packaging/windows/Assets/Square150x150Logo.png`); there is no master. The owner will commit master artwork (SVG, or PNG of at least 1024 px) under `packaging/icons/`. **When it exists** (check at the start of work; if absent, do everything else first and mark this step blocked):

- Generate `48×48`, `64×64`, `128×128`, `256×256` PNGs for `packaging/linux/icons/hicolor/<size>x<size>/apps/io.github.nabbisen.Logolig.png` **with Logolig itself** (the application, default settings, Lanczos3) from the master, and record the exact procedure; if the master is SVG and an SVG icon is wanted, install it as `scalable/apps/io.github.nabbisen.Logolig.svg` unchanged.
- Verify the icons visually at each size and report any quality problem.
- Until then, for the dry run only, you may create a **temporary** placeholder icon in the dry-run archive (never committed, clearly labelled in the review request). A published package without a real icon is not acceptable.

### 5.6 `packaging/README.md`

Short and factual: how to render, how to test in a clean environment, how the owner publishes the first version by hand (clone the AUR repo, copy `PKGBUILD` and `.SRCINFO`, check, commit, push), how to bump later versions, and the **rollback rule**: never force-push the AUR repository; supersede a bad package by increasing `pkgrel` or the version. State plainly that publication to the AUR is an owner action in Phase 3 and that CI never pushes in Phase 2.

### 5.7 Versions and the first release

Document in `packaging/README.md` that the first AUR version must be a release whose tag archive already contains `packaging/linux/` (that is, a release after this work lands), and that `pkgver` must equal the tag. Do not choose or create that release.

### 5.8 CI check (after the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) is merged)

Add `.github/workflows/packaging.yml`, first-party actions only, pinned by SHA, `permissions: contents: read`, trigger `pull_request` with `paths: ['packaging/**', 'scripts/render-aur.sh']`. Job `aur-source` runs in a container (`archlinux:base-devel`, pinned by digest, with a comment explaining the pin and how to refresh it), runs `pacman -Syu`, creates a non-root build user, renders with `--tarball` from `git archive` of the PR checkout (so the PR's own packaging files are tested), then runs `makepkg`, both `namcap` commands, and `desktop-file-validate`. It must never read secrets or push anywhere. Provide the evidence runs as in the RFC 020 phase-1 handoff (`implementation-handoff-phase-1-ci-gates.md`) §6 (green run, plus a deliberate failure reverted with `git revert`).

## 6. Required tests and acceptance criteria

- `makepkg` succeeds from the rendered dry-run PKGBUILD in a clean environment; `namcap` has no unexplained errors; `desktop-file-validate` is clean; `ldd` shows no missing library.
- The installed file list of the package is exactly: `/usr/bin/logolig`, `/usr/share/applications/io.github.nabbisen.Logolig.desktop`, the icon files. Show `pacman -Qlp <package>`.
- `render-aur.sh`: negative tests, each recorded: unknown argument; missing `--tarball` file; output directory inside the AUR clones (must refuse); unreplaced placeholder (inject one in a copy of the template). Positive test with the real tag `1.26.2` only to prove the **download and hash path** works (the build from that archive is expected to fail at the icon step because the tag has no packaging files, which also proves the loud-failure rule; record both outcomes).
- The dependency table (§5.4 item 5) and the portal finding (item 7) are present.
- No file changed outside §3.

## 7. Prohibited shortcuts

- No placeholder checksums (`DUMMY…`) and no `SKIP` in anything that is meant to be published; every source has a real SHA-256.
- No `|| true`, no silent skips, no `--nocheck`, `--skipinteg`, `--skippgpcheck`, `--nodeps` in evidence runs.
- No guessing of dependencies, window classes or categories; evidence or a reported open question.
- No new artwork, no upscaled icons.
- No pushing to AUR, no changes in the AUR clones, no tags or releases, no repository settings or secrets.
- No application code changes to make packaging work; report them as findings.

## 8. Security and compatibility constraints

- The package builds from a checksum-verified source archive and the committed `Cargo.lock` (`--locked`/`--frozen`).
- Build and test runs as a non-root user.
- No network access in `build()`; only `prepare()` fetches.
- Do not embed credentials, tokens or personal paths in any committed file or log; scrub review artefacts.
- Compatibility: Arch Linux x86_64, current rolling release; the declared Rust floor is 1.90 (RFC 024) and is irrelevant to Arch's current `cargo`; do not change `rust-version`.

## 9. Known risks

- GUI start-up in a container or under Xvfb may fail only because of missing GPU or Vulkan; separate "environment cannot show it" from "package is wrong" and state which.
- File chooser without a portal (item 7) may reveal a user-facing defect; report, do not fix here.
- The `check()` step compiles `logolig` core crates again; watch build time and report it.

## 10. Delivery

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** Work directly on `main`, one small commit at a time (match the repository's short imperative subject style), `git pull --ff-only` before you start, and push after each commit that passes the local gates. Never rewrite history or force-push. Do not tag or release. The architect reviews the commit range afterwards; corrections are new commits. The owner publishes to the AUR in Phase 3. This job starts only when the preceding jobs in `ROADMAP.md` are reviewed.

Because the workflow now runs on `push` to `main`, the packaging workflow of §5.8 triggers on `push` with the same `paths` filter (in addition to `pull_request`), and the "deliberate failure" demonstration is done with two commits on `main` (a failing one, then its `git revert`), announced in the review request so that the red run is not mistaken for a regression.

## 11. Required review request

In `.git-exclude/review-request/rfc-020-phase-2-aur-source/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the exact commands and logs for §5.4 and §6; the dependency table; the portal finding; tool availability on the workstation; the `pacman -Qlp` listing; the negative-test results; the note on how the dry-run archive was built; the state of the icon step (blocked or done); and open questions for the architect. Report to the owner only the path of the `README.md`.
