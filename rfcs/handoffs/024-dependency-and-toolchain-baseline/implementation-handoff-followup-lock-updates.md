# Developer Handoff — RFC 024 follow-up (J1b): three single-crate lock updates

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (relative paths with `cd`).

Governing RFC: [RFC 024](../../accepted/024-dependency-and-toolchain-baseline.md) ("Review outcome and further decisions"). On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design.

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first, one commit per crate, push after each commit that passes **all** local gates. **Chain the gates so the chain stops on the first failure (`&&`) and never push a red commit** (the J1 review recorded one red push; do not repeat it). Never rewrite history or force-push. Do not tag or release.

**Position:** job J1b in `ROADMAP.md`, after J1 (reviewed), before J2.

## 1. Purpose

Clear three more `cargo audit` warnings (unsound-crate notices) because patched releases exist within the semver range, without touching anything else.

## 2. Background (verified by the architect from the advisory database and the J1 review)

| Advisory | Crate (locked) | Patched in | Pulled in by |
|---|---|---|---|
| RUSTSEC-2026-0190 | `anyhow` 1.0.102 | `>= 1.0.103` | `av-scenechange` chain via `image` (AVIF encoder, not enabled by the workspace's image features) |
| RUSTSEC-2026-0221 | `event-listener` 5.4.1 | `>= 5.4.2` | `async-broadcast` via `zbus` (Linux file dialog / theme detection) |
| RUSTSEC-2026-0186 | `memmap2` 0.9.10 | `>= 0.9.11` | `fontdb` via `cosmic-text` / `usvg` (font files) |

After J1, `cargo audit` reports 0 vulnerabilities and 14 warnings; these three are among them.

## 3. Change scope

`Cargo.lock` only: `cargo update -p anyhow`, `cargo update -p event-listener`, `cargo update -p memmap2`, each as its own commit. If a crate has two locked versions, update the one named in the advisory and report both.

## 4. Non-change scope

Everything else: `Cargo.toml`, every other dependency (no bare `cargo update`, no `--precise` downgrade tricks), CI, `.cargo/audit.toml`, source code, docs, RFCs. If an update forces any other crate to move, stop and report instead of committing.

## 5. Required validation (per commit, chained with `&&`)

```
cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit
```

Record, for each crate, the old and new locked version and the audit warning count before and after (expected 14 -> 13 -> 12 -> 11). After the last push record the GitHub Actions run of the final commit and the verdict of each of the five jobs. Run the application once with an isolated config (`XDG_CONFIG_HOME` at a scratch directory; never the maintainer's real settings), open a file with the chooser and confirm the window works (these crates touch the file dialog and font paths); screenshot per `.git-exclude/rules/project-instructions-gui-common.md`.

## 6. Acceptance criteria

The three advisories no longer appear; no new advisory; all gates and all five CI jobs green; the lock diff of each commit contains only that crate (plus nothing else); the GUI starts and opens the file chooser.

## 7. Prohibited shortcuts

No ignores, no edits to `.cargo/audit.toml`, no `[patch]`, no pinning of other crates, no editing CI, no skipping gates, no red pushes.

## 8. Required review request

In `.git-exclude/review-request/rfc-024-followup-lock-updates/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the three old/new versions; warning counts before/after; the lock diff summary; gate results per commit; the CI run verdicts; the GUI check evidence; and open questions. Report to the owner only the path of the `README.md`.
