# Developer Handoff — RFC 020, Phase 1: GitHub Actions CI gates

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/`.

Governing RFC: [RFC 020](../../accepted/020-ci-and-aur-packaging.md) (Accepted, phased). This handoff covers **Phase 1 only**. The RFC defines what and why; this document defines how to implement and prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users and maintainers must not be confused. For CI this means: least privilege, no unpinned third-party code, honest failures, no silent skips.

Dependencies: none. Runs in parallel with the RFC 010 handoff (no shared files). the RFC 020 phase-2 handoff (`implementation-handoff-phase-2-aur-source-package.md`) builds on the workflow you create, so keep job names stable.

## 1. Purpose

Run the project's release gates automatically on every pull request and every push to `main`, so that the red/green state of the tree is visible and no longer depends on maintainer discipline (a recommendation open since the v1.25.0 handoff).

## 2. Background (verified facts)

- No `.github/` directory exists today.
- Gates run by hand: `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace --all-features`; `cargo audit`. At commit `9d79f25`: fmt clean, clippy clean, 147 tests pass (0 failed). `cargo audit` was not run in the last review; an earlier review (2026-07-09) found it red, after which `.cargo/audit.toml` was added with two documented ignores (`RUSTSEC-2026-0194`, `-0195`, quick-xml via `wayland-scanner`, build-time).
- `Cargo.toml`: `resolver = "3"`, `edition = "2024"`, `rust-version = "1.85"`, `Cargo.lock` is committed. The maintainer's toolchain is 1.99.1; nothing currently checks the 1.85 floor.
- GUI crates: `iced 0.14` (wgpu/tiny-skia), `snora 0.25`, `rfd 0.15`. Which system packages the Linux build needs on an Ubuntu runner is **not known** and is part of this task.
- Release tags will be `X.Y.Z` without a `v` prefix. Not relevant to Phase 1 except for naming.

## 3. Change scope (you may add or touch)

- `.github/workflows/ci.yml` (new).
- `README.md`: one CI status badge in the existing badge block, and in the "Tests" section one sentence that CI runs these gates. Nothing else in the README.
- `.cargo/audit.toml`: **read only** unless the architect approves a change (see §5, audit).

## 4. Non-change scope (you must not touch)

- Anything under `crates/`, `docs/`, `rfcs/`, `packaging/`, `Cargo.toml`, `Cargo.lock`. If a gate fails in CI because of the code, **do not fix the code here**; report it.
- No `release.yml`, no `aur.yml`, no tags, no GitHub Releases, no secrets, no environments, no repository settings.
- No Windows or macOS jobs (out of scope for Phase 1).
- No third-party actions (see §5.3).

## 5. Required implementation

### 5.1 Workflow `ci.yml`

Triggers: `pull_request` and `push` to `main`. Also `workflow_dispatch` (harmless; useful later). `concurrency` group per workflow and ref with `cancel-in-progress: true` for pull requests only (never cancel runs on `main`).

Top level `permissions: contents: read`. No job gets more.

Job names are a contract (branch protection and the RFC 020 phase-2 handoff (`implementation-handoff-phase-2-aur-source-package.md`) depend on them). Use exactly: `fmt`, `clippy`, `test`, `msrv`, `audit`.

| Job | Toolchain | Command |
|---|---|---|
| `fmt` | stable | `cargo fmt --check` |
| `clippy` | stable | `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` |
| `test` | stable | `cargo test --workspace --all-features --locked` |
| `msrv` | `1.85` (read it from `Cargo.toml` `rust-version`, do not hardcode twice) | `cargo check --workspace --all-targets --all-features --locked` |
| `audit` | stable | `cargo audit` (reads `.cargo/audit.toml`) |

Use `runs-on: ubuntu-latest` and a `timeout-minutes` on every job (suggest 30; measure and adjust).

### 5.2 Toolchain and dependencies

- Install toolchains with the `rustup` already present on the runner (`rustup toolchain install stable --profile minimal [-c rustfmt,clippy]`, `rustup default`), not with a third-party action.
- Install `cargo-audit` with `cargo install cargo-audit --locked` (cache the binary; see 5.4). Print the version in the log.
- Install the Linux system packages the build needs with `apt-get` (record the final list, with a comment on why each is needed). Determine it by iterating in CI; start from what `iced`/`winit`/`wgpu`/`rfd` document (xkbcommon, wayland, x11/xcb, fontconfig, vulkan loader headers). Do not install more than needed.

### 5.3 Supply chain

- Use only first-party `actions/*` actions (`actions/checkout`, `actions/cache`), each **pinned to a full commit SHA** with the tag as a trailing comment, for example `uses: actions/checkout@<40-hex> # v4.x.y`. Resolve the SHAs from the official repositories at the time of implementation and record how you verified them.
- `actions/checkout` with `persist-credentials: false`.
- No `curl | sh`, no unpinned downloads.
- Never use `pull_request_target`. Never echo secrets (there are none in Phase 1).

### 5.4 Caching

Cache `~/.cargo/registry`, `~/.cargo/git` and `target/` with `actions/cache`, keyed by OS, toolchain and `hashFiles('Cargo.lock')`, with a restore-key fallback by toolchain. Cache the installed `cargo-audit` binary separately. A cache must never change a result: if you suspect staleness, say so in the review request.

### 5.5 `msrv` job: what to do if it fails

`rust-version = "1.85"` has never been verified by a gate. Some locked dependencies may require a newer compiler. **If `msrv` fails, do not change `Cargo.toml`, `Cargo.lock`, or pin crates.** Report the failing crates and their required versions in the review request. The architect and owner decide whether to raise the MSRV or hold dependencies. Meanwhile keep the job in the file so the result is visible, and state clearly that it is red and why.

### 5.6 `audit` job: what to do if it fails

`cargo audit` fails on vulnerabilities (not on warnings) by default; keep that default. If it reports vulnerabilities that are **not** covered by `.cargo/audit.toml`, **do not add ignores and do not change dependencies**. Report them (advisory id, crate, version, fixed version, reachable or build-time) so the architect can triage. A red `audit` job is an acceptable and honest outcome of this task; a silenced one is not.

### 5.7 Badge and README

Add the standard GitHub Actions status badge for `ci.yml` on `main` to the existing badge block, in the same style as the others. In the "Tests" section add one sentence that the same gates run in CI.

## 6. Required tests and evidence

CI cannot be unit-tested; evidence is runs:

1. Validate the YAML locally (`actionlint` if available; otherwise state that it was not available and use `python -c 'import yaml'` for syntax only, and say that this is weaker).
2. Work on branch `ci-phase-1`. Push it and open a **draft pull request** into `main` so the `pull_request` trigger runs. Record the run URL and the result of each of the five jobs.
3. Demonstrate that the gates can actually fail: on a throwaway commit in the same branch, introduce a deliberate formatting error, a clippy warning, and a failing test (one at a time or together), push, confirm that `fmt`, `clippy` and `test` each go red, then **revert the commits** (use `git revert`; do not force-push or rewrite history). Record the run URLs. A gate that has never been seen failing is not evidence.
4. Local reproduction: run the same five commands locally on the same commit and compare the verdicts with CI. Differences must be explained.
5. Record wall-clock time per job and whether the second run benefited from the cache.

## 7. Acceptance criteria

- `ci.yml` exists, with exactly the five jobs named in §5.1, least-privilege permissions, pinned first-party actions only, and no secrets.
- On the draft PR: `fmt`, `clippy`, `test` are green; `msrv` and `audit` have an honest, explained result (green, or red with the report required by §5.5/§5.6).
- The deliberate-failure demonstration was performed and reverted.
- README badge and sentence added; no other file changed.
- The review request contains the final apt package list and the SHA-verification notes.

## 8. Prohibited shortcuts

- No `continue-on-error: true`, no `|| true`, no skipping a job on failure, no `if: false`.
- No loosening of the commands (for example dropping `-D warnings`, `--all-targets`, `--all-features`).
- No ignoring advisories, no dependency or MSRV changes, no code changes to make CI green.
- No `--no-verify`, no force-push, no rewriting of history, no tags.
- No adding of repository secrets or environments. Branch-protection and "required check" settings are an **owner action**; list the five job names in the review request for that purpose, but do not attempt to set them.

## 9. Security and compatibility constraints

- `permissions: contents: read` only. No write tokens, no `GITHUB_TOKEN` escalation.
- Pull requests from forks must run with no secrets (none exist; keep it that way).
- The workflow must not alter the repository, create releases, or upload artifacts.

## 10. Known risks

- First runs may fail for missing system libraries; iterate, and keep the list minimal.
- `cargo install cargo-audit` is slow without the cache; do not replace it with an unpinned third-party action.
- GitHub Actions minutes and runner image changes (`ubuntu-latest` moves). Report the Ubuntu image version seen in the logs; the architect may decide to pin `ubuntu-24.04`.

## 11. Delivery

Branch `ci-phase-1`, small commits, push the branch, one draft PR. **Do not merge it.** The architect reviews and merges.

## 12. Required review request

In `.git-exclude/review-request/rfc-020-phase-1-ci/` with a `README.md` entry point, containing everything required by section 7 of the mid-capability operating instructions, plus: all run URLs; the per-job verdict table; the apt package list with reasons; the SHA pinning notes; MSRV and audit findings in full if red; the deliberate-failure evidence and its reversal; local-versus-CI comparison; timings; and the list of owner actions (required checks). Report to the owner only the path of the `README.md`.
