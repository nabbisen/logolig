# RFC 020 — GitHub Actions CI and AUR packaging (`logolig`, `logolig-bin`)

**Status.** Accepted (phased; phase 2's icon step waits for the owner's artwork; phase 4 waits for the D4 set-up, 2026-10-21)
**Priority.** P1 (owner priority, 2026-10-21: AUR publication first)
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
| `aur.yml` | **manual `workflow_dispatch` only**, with the release tag as input; runs in the protected `aur` environment | verify the release is published and its checksum matches, render PKGBUILDs from templates, run `makepkg`/`namcap`, push to the AUR repo(s) |

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
- `prepare()` runs `cargo fetch --locked` so that `build()` can use `--frozen` (AUR convention: no network during `build()`).
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

## Phases

| Phase | Content | Needs from the owner |
|---|---|---|
| 1 | `ci.yml` gates (fmt, clippy, test, MSRV, audit; docs jobs once RFC 008/018 land). No secrets, no release. | nothing |
| 2 | `packaging/` templates, `.desktop`, icon, container build with `makepkg` + `namcap`, dry run on the maintainer's Arch machine. `logolig` (source) first. | D5 master artwork |
| 3 | **First publication of `logolig` is done by the owner by hand** from a verified PKGBUILD (`git push` to AUR). Proves the package before any secret exists. Then `logolig-bin` (needs `release.yml` artifacts and attestation). | release approval |
| 4 | `aur.yml` automation for later releases. | D4 set-up: CI-only key and protected `aur` environment (conditions in D4) |

## Decisions (2026-10-21, applying the owner's philosophy: clean, safe, secure, robust; users not confused)

- **D1 — two packages: accepted, sequenced.** `logolig` (source build, verifiable by anyone from the tagged source) is published first. `logolig-bin` follows because compiling an iced application takes long and a binary package has real user value, but it asks users to trust a binary, so it carries two additions: a SHA-256 in the PKGBUILD **and** a GitHub build-provenance attestation for the release asset (`actions/attest-build-provenance`, pinned by SHA), with the verification command in `packaging/README.md`. Scope: Linux x86_64, glibc, as built on the CI runner; no promise of bit-for-bit reproducibility is made in the docs. `logolig-bin` declares `provides=(logolig)` and `conflicts=(logolig)` so users are never left with two copies.
- **D2 — approval design: accepted, tightened.** A tag creates only a draft release; the owner's publishing of that release is the release approval; the AUR push is a separate, manual, environment-protected step (`workflow_dispatch`), so no outward-facing publication can happen unattended. For a one-person project the environment's reviewer is the owner; "prevent self-review" is left off because it would block the only maintainer, and the manual trigger is the real gate.
- **D3 — AUR push by plain shell: accepted.** Clone, copy `PKGBUILD`, regenerate `.SRCINFO` with `makepkg --printsrcinfo` (run in an Arch container as a non-root user, because `makepkg` refuses root), commit, push. No community deploy action. All actions pinned to commit SHAs; `permissions:` minimal per job.
- **D4 — AUR account and key: decided by the owner (2026-10-21): the owner's own AUR account is used, and the owner sets the secret in a GitHub Actions environment.** The architect had recommended a separate bot account as co-maintainer, because an AUR SSH key belongs to an account, not to a package. The owner chose otherwise; this is the owner's risk decision and it is recorded here as an **accepted residual risk**: a leaked CI key could modify every AUR package of that account, not only `logolig` and `logolig-bin`. Conditions that keep the risk small (all part of the Phase 4 acceptance criteria):
  1. A **dedicated SSH key pair for CI only**, generated for this purpose and never reused for anything else. Register its public key on the AUR account in addition to the owner's personal key (to the architect's knowledge the AUR accepts more than one key per account; confirm in the account settings, and if only one key is allowed, stop and ask before replacing the personal key). The CI key can then be revoked alone.
  2. The private key lives **only** as a secret of the GitHub environment `aur`; never as a repository-level secret, never in a file, log or artifact.
  3. The environment is restricted: deployment allowed only from `main` (or release tags), required reviewer = the owner, and the workflow is `workflow_dispatch` only (RFC 020 D2). Pull requests and forks cannot reach it.
  4. The AUR step prints no secret material, does not enable shell tracing (`set -x`), and pins `known_hosts` for `aur.archlinux.org` from a verified fingerprint committed in the repository (not `StrictHostKeyChecking=no`).
  5. Rotation and revocation are documented in `packaging/README.md` (rotate yearly and after any doubt; remove the public key from the AUR account to revoke).
  6. The owner can move to a separate bot account later without changing the workflow (only the secret and the AUR co-maintainer setting change). Recommended if the owner ever maintains other AUR packages that the key should not be able to touch.
  Phase 4 does not start until conditions 1–4 are set up and the owner confirms it.
- **D5 — icon and desktop entry: owner will provide the master artwork on request (2026-10-21: "Give me request later").** The architect issues a written request when Handoff 004 reaches the icon step (§5.5). Until then that step stays blocked and nothing is invented. Finding: the repository contains the logo only as the 150 × 150 Microsoft tile (`packaging/windows/Assets`), a rounded blue square with the hexagon mark, and no master artwork. Upscaling it would produce a blurry Linux icon, which would be a poor first impression for a tool whose selling point is output quality. Request: commit the master (SVG preferred, or a PNG of at least 1024 px) to `packaging/icons/`. Then generate every size, Windows assets included, **with Logolig itself**, which also serves as a standing end-to-end test of the product. Desktop entry text is localised (`en`, `ja`) and keeps the project's "less is more" wording.

## First-publication readiness (recommendation; the release decision remains with the owner)

Publishing to the AUR makes the application discoverable to a new audience, and users there will not read the RFCs. Recommended before the first AUR release:

1. RFC 009 fixed (ZIP failure with monochrome on is a visible, data-losing error).
2. RFC 019 step 2 done (documentation states honestly what keyboard support exists) and, ideally, step 1 evidence recorded.
3. RFC 015 spike done: confirm or refute the SVG-text defect. If confirmed, fix it or document it prominently, since logo SVGs often contain text.
4. README install section updated for AUR.
5. Gates green on the release commit, `cargo audit` result recorded.

A natural vehicle is a patch release (for example 1.26.3) that carries these fixes and is the first release produced by the new pipeline.

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
| AUR key leak | High impact (account-wide: the owner's account is used, D4), low likelihood | Conditions 1–6 under D4: CI-only key, environment secret, branch-restricted environment with required reviewer, manual dispatch, pinned `known_hosts`, rotation; separately revocable key |
| Bad package published | Medium | Draft-release gate, `makepkg`/`namcap` in CI, supersede-don't-rewrite policy |
| Missing runtime libraries on Arch | Medium | Container install test; record results |
| Workflow supply-chain compromise | Medium | SHA-pinned actions, minimal permissions |
| CI minutes/cost | Low | Public repository; cache `~/.cargo` and `target` |

## Dependencies

Phase 1 starts independently. The RFC-lint job is added when RFC 008 lands. RFC 017 is not a prerequisite.
