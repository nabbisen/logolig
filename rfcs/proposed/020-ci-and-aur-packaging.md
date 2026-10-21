# RFC 020 — GitHub Actions CI and AUR packaging (`logolig`, `logolig-bin`)

**Status.** Proposed
**Priority.** P2 (the AUR names are already reserved; CI has been a recommended follow-up since the v1.25.0 handoff)
**Tracks.** Release engineering: automated quality gates, release artifacts, and AUR publication.
**Touches.** new `.github/workflows/`, new `packaging/aur/`, new `packaging/linux/` (desktop entry, icon), `scripts/`, `.cargo/audit.toml`, `README.md` install section.

## Summary

Run the project gates in GitHub Actions on every change, build release artifacts on every release tag, and publish to two AUR packages — `logolig` (builds from the tagged source) and `logolig-bin` (installs a prebuilt binary from the GitHub release) — **without removing the owner's release approval**.

## Evidence

Verified (file inspection, 2026-10-21):

- No `.github/` directory exists. Gates are run by hand (`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features`, `cargo audit`). All were green at commit `037d0ae` except that `cargo audit` was not run in this review.
- `.cargo/audit.toml` ignores `RUSTSEC-2026-0194` and `-0195` (quick-xml via wayland-scanner, build-time) with a stated removal condition.
- The AUR repositories (cloned under `.git-exclude/tmp/aur-repositories/`, remote `ssh://aur@aur.archlinux.org/<name>.git`) each contain one placeholder commit: `pkgver=0.0.0`, `sha256sums=('DUMMY-FOR-INITIALIZATION')`, `package()` that only echoes, `depends=(gcc-libs glibc fontconfig libxkbcommon vulkan-icd-loader)`, no `makedepends`, and for **both** packages `source` is the GitHub *tag source archive*. For `logolig-bin` that is wrong; it must point to a release binary asset.
- Release tags are `X.Y.Z` with no `v` prefix (workflow rule, and `archive/refs/tags/$pkgver.tar.gz` in the placeholders already assumes this).
- Application binary name is `logolig` (`crates/app/Cargo.toml` `[[bin]]`). The repository ships no `.desktop` file and no Linux icon; `packaging/windows/` holds the Microsoft Store manifest and logo assets.
- Declared MSRV is 1.85; the maintainer toolchain in this review is 1.99.1. No gate checks the MSRV.
- `makepkg`, `namcap`, `cargo-audit` and `cargo-deny` are installed on the maintainer workstation (useful for local dry runs).
- Not verified: the full list of runtime/build dependencies the GUI needs on Arch (needs a `namcap` pass in a clean container), and whether the release binary starts on a minimal system.

## Goals

1. Gates on every push and pull request: format, clippy `-D warnings`, tests, audit, RFC lint, i18n parity (covered by tests), Markdown link check, and an MSRV build.
2. Reproducible release artifacts on tag `X.Y.Z`.
3. AUR packages that build and install a working, correctly licensed application with a desktop entry and icon.
4. Publishing to AUR happens only after explicit owner approval of that release.

## Non-goals

crates.io publication (separate owner decision), Windows Store submission (existing `packaging/windows` stays manual), macOS bundles, auto-merge or auto-version-bump bots.

## Design

### 1. Workflows

| Workflow | Trigger | Jobs |
|---|---|---|
| `ci.yml` | `pull_request`, `push` to `main` | `fmt`, `clippy`, `test` (stable); `msrv` (1.85, `cargo check --workspace --locked`); `audit` (`cargo audit`, reads `.cargo/audit.toml`); `docs` (RFC lint from RFC 008, link check); `packaging` (only when `packaging/**` changes: `makepkg --verifysource`/build in an `archlinux` container plus `namcap`) |
| `release.yml` | push of tag `[0-9]+.[0-9]+.[0-9]+` | verify tag equals `workspace.package.version`; re-run gates; build `x86_64-unknown-linux-gnu` release with `--locked`; create a **draft** GitHub Release with `logolig-<ver>-x86_64-linux.tar.gz` and its `.sha256` |
| `aur.yml` | release **published** (the owner presses publish), or manual `workflow_dispatch` | render PKGBUILDs from templates, run `makepkg`/`namcap`, push to both AUR repos |

Ubuntu runners need the iced/winit system libraries at build time; the exact package list is determined in the first CI run and recorded in the workflow.

### 2. Governance of release and publication

- Pushing a tag creates only a **draft** release. Publishing the draft is the owner's release approval (workflow doc §3.1, §6.6).
- `aur.yml` runs in a protected GitHub **environment** (`aur`) with required reviewers, so the secret below is released only after approval. Pushing to AUR is outward-facing and not cleanly reversible (history is public), so no step in that job runs unattended on a tag push.
- Rollback policy: never force-push AUR. A bad package is superseded by bumping `pkgrel` or the version.

### 3. Secrets and supply chain

- One dedicated SSH key pair for AUR, private key stored only as an environment secret in `aur`; public key registered with the AUR account. The key is used for these two packages only.
- Workflows declare least-privilege `permissions:` (`contents: read` by default; `contents: write` only in the release job).
- Third-party actions are pinned to full commit SHAs. Prefer first-party `actions/*` and plain shell for the AUR push (clone, copy `PKGBUILD`, regenerate `.SRCINFO` with `makepkg --printsrcinfo`, commit, push) over a community deploy action; each pinned action is an additional trust decision (D3).
- No secrets are exposed to pull requests from forks; the `aur` environment is unavailable to them.
- Build with `--locked`; the committed `Cargo.lock` is authoritative.

### 4. AUR package definitions

Templates live in the main repository (`packaging/aur/logolig/PKGBUILD.in`, `packaging/aur/logolig-bin/PKGBUILD.in`), so changes are reviewed in pull requests and validated by the `packaging` job before any push. The AUR clones stay mirrors.

`logolig` (source):
- `source` = tag archive, `sha256sums` computed by the workflow, `makedepends=(cargo pkgconf)` plus whatever the container pass requires, `build()` = `cargo build --release --locked --frozen -p logolig-app`, `package()` installs `target/release/logolig` to `/usr/bin/`, `LICENSE` to `/usr/share/licenses/logolig/`, the `.desktop` file and the icon.

`logolig-bin`:
- `source` = release asset `logolig-<ver>-x86_64-linux.tar.gz`, checksum from the `.sha256`, `provides=(logolig)`, `conflicts=(logolig)`, installs the same files, `depends` as above.

Both: `arch=(x86_64)`, `license=('Apache-2.0')`, `url` unchanged, `pkgdesc` real text (placeholders removed).

### 5. New repository files

`packaging/linux/logolig.desktop` (name, `Exec=logolig`, `Icon=logolig`, categories Graphics;, localized `Name[ja]`), a scalable or 256-px icon derived from existing assets, and a short `packaging/README.md` describing the release procedure and the rollback rule.

## Alternatives considered

- Keep PKGBUILDs only in the AUR repos and bump them by hand: simplest, but nothing validates them and releases depend on manual discipline (the v1.22/v1.23 incidents were manual-edit failures).
- Fully automatic AUR push on tag: removes the human release gate; rejected by the governance rules.
- AppImage/Flatpak instead of AUR: possible later; out of scope.

## Decisions requested

- **D1** Confirm the two-package scheme and that `logolig-bin` targets glibc x86_64 only.
- **D2** Confirm the approval design (draft release + protected `aur` environment).
- **D3** Allow a pinned community action for the AUR push, or require plain shell (recommended).
- **D4** Who holds the AUR account, and is a single dedicated key acceptable?
- **D5** Icon/desktop-entry source assets (use the Windows logo assets or provide new artwork).

## Tests and evidence

- Every workflow validated on a fork or a throwaway branch before relying on it; workflow run links are the evidence.
- Local dry run: `makepkg --noextract`/`namcap` against rendered templates on the maintainer's Arch machine before the first AUR push.
- First release through the pipeline is a patch release with no functional change, so pipeline faults are not confused with product faults.
- Smoke test of the `-bin` package: install in a clean Arch container, `logolig --version` is not available (no CLI), so test by launching under `xvfb`/headless compositor for N seconds without crash, and record the limits of that check.

## Acceptance criteria

- `ci.yml` is green on `main` and required for merging.
- A tagged release produces a draft release with binary and checksum; nothing is published to AUR until the owner approves.
- Both AUR packages install a launchable application with a desktop entry, icon and license file; `namcap` reports no errors.
- The AUR secret is available only to the `aur` environment.

## Risks

| Risk | Rating | Mitigation |
|---|---|---|
| AUR key leak | High impact, low likelihood | Environment secret, required reviewers, dedicated key, rotate on any doubt |
| Bad package published | Medium | Draft-release gate, `makepkg`/`namcap` in CI, supersede-don't-rewrite policy |
| Missing runtime libraries on Arch | Medium | Container install test; record results |
| Workflow supply-chain compromise | Medium | SHA-pinned actions, minimal permissions |
| CI minutes/cost | Low | Public repository; cache `~/.cargo` and `target` |

## Dependencies

Benefits from RFC 008 (RFC lint script) and RFC 017 (smaller files make clippy/test faster to review), but can start independently with the gate jobs.
