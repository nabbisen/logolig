# Developer Handoff — RFC 024 follow-up (J2a): update `tokio`

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (relative paths with `cd`).

Governing RFC: [RFC 024](../../accepted/024-dependency-and-toolchain-baseline.md). Workflow: directly on `main`, no branches, no concurrent jobs; chain every gate with `&&` and push only when all pass; never rewrite history; no tags.

**Position:** job J2a in `ROADMAP.md`, after J2 (reviewed), before J2b (snora upgrade, which will also touch `Cargo.lock`).

## 1. Purpose

Clear RUSTSEC-2026-0386 (`tokio` 1.52.3, unsound `AsyncFd` constructors; patched `>= 1.51.6, < 1.52.0` and `>= 1.53.3`). The architect's dry run: `cargo update -p tokio` moves it to 1.54.1 and nothing else.

## 2. Scope

`Cargo.lock` only: `cargo update -p tokio`, one commit. Do not touch `Cargo.toml`, any other crate, CI, `.cargo/audit.toml`, code or docs. If any other crate would move, stop and report. The yanked `clipboard-win` 5.4.1 warning is Windows-only and out of scope; do not act on it.

## 3. Validation (chained with `&&`)

```
cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit
```

Record old/new version, audit warning count before (13) and after (expected 12), and that RUSTSEC-2026-0386 is gone with no new advisory. Record the CI run of the pushed commit and each of the five job conclusions; if GitHub shows a job as `in_progress` while the run is completed, read the raw job log (`gh api --allow-escape-sequences repos/nabbisen/logolig/actions/jobs/<id>/logs`) instead of guessing. Run the app once with an isolated `XDG_CONFIG_HOME`, open a file through the chooser, convert it, and confirm the Result screen appears (tokio drives the async tasks); screenshot per `.git-exclude/rules/project-instructions-gui-common.md`.

## 4. Acceptance

Only tokio changed in the lock; advisory gone; all gates and five CI jobs green; GUI conversion works.

## 5. Review request

`.git-exclude/review-request/rfc-024-followup-tokio/README.md` with everything required by section 7 of the mid-capability instructions. Report only that path.
