# Developer Handoff — RFC 024: Dependency and toolchain baseline (MSRV 1.90, `zip` upgrade, audit triage)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 024](../../accepted/024-dependency-and-toolchain-baseline.md) (Accepted). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled. A false compatibility statement and a dependency with known vulnerabilities are both things this job removes.

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** You are the only job running. Work directly on `main`: `git pull --ff-only` first, small commits (short imperative subject, like the existing history), push after each commit that passes the local gates. Never rewrite history or force-push. Do not tag or release.

**Position in the schedule:** this is job J1 in `ROADMAP.md`. Prerequisite: `main` already contains the reviewed merges of RFC 009, RFC 010, RFC 019 phase 1 and the CI workflow (`.github/workflows/ci.yml`). Check with `git log --oneline` and by looking for `.github/workflows/ci.yml` and `crates/app/src/task_queue/zip_tests.rs`. **If they are missing, stop and report; do not merge branches yourself.**

## 1. Purpose

Make the two honest red CI jobs green by making the facts true:

1. The declared minimum Rust version must equal the real one (1.90).
2. `cargo audit` must stop reporting the three `zip` 2.4.2 vulnerabilities by upgrading `zip` to a fixed release.
3. Produce a one-time triage of the 14 audit warnings.

## 2. Background (verified facts)

- `cargo check --workspace --locked` fails on rustc 1.85.1: `app-json-settings 2.0.3` needs 1.90; `iced 0.14.0`, `image 0.25.10`, `wgpu 27.0.1` need 1.88; others 1.87. `+1.90` passes, `+1.89` fails (CI run 39558919968 and the implementer's local probe).
- `README.md` says "Requires Rust 1.85 or newer"; workspace `Cargo.toml` has `rust-version = "1.85"`.
- `cargo audit` (cargo-audit 0.22.2): RUSTSEC-2026-0367, -0368, -0369 on `zip 2.4.2`, fixed in `>= 9.0.0`. All concern **extraction**; production code only writes archives (`ZipWriter` in `crates/app/src/task_queue.rs`); the one `ZipArchive` use is in tests. Plus 14 allowed warnings (adler, ansi_term, atty (two), anyhow, event-listener, lru, memmap2, paste, rustybuzz, ttf-parser, zbus_names, zvariant, zvariant_derive).
- `.cargo/audit.toml` ignores RUSTSEC-2026-0194 and -0195 (quick-xml via `wayland-scanner`, build-time) with a stated removal condition.
- `zip` 9.0.5 declares `rust-version = "1.88"` (crate metadata in the local registry). Not verified: its API and feature names compared with 2.4.2. The workspace uses `zip = { version = "2", default-features = false, features = ["deflate"] }`; the app uses `ZipWriter`, `zip::write::SimpleFileOptions`, `CompressionMethod::Deflated`, `last_modified_time`, `zip::DateTime::from_date_and_time`, and the tests use `ZipArchive::new`, `by_name`, `last_modified`.
- DEC-002 / C-4 (decision log): `resolver = "3"` and `edition = "2024"` in `Cargo.toml` are load-bearing and have been silently regressed before. Edit `Cargo.toml` atomically and re-check both after every edit.

## 3. Change scope (you may touch)

- Workspace `Cargo.toml`: `rust-version` → `"1.90"`; the `zip` dependency line (version and, only if required, feature names).
- `Cargo.lock`: only the changes caused by the `zip` upgrade (use `cargo update -p zip --precise <version>` or editing the requirement then `cargo update -p zip`; see §5.2). Review `git diff Cargo.lock` and report every crate that changed.
- `crates/app/src/task_queue.rs` and `crates/app/src/task_queue/zip_tests.rs`, `crates/app/src/ui` only as far as the `zip` API change forces; keep behaviour identical.
- `README.md`: the minimum Rust version statement (and nowhere else in the README).
- Other documents only where a grep for `1.85` finds a statement about the minimum Rust version (list them; edit those statements).
- `.cargo/audit.toml`: **only** to remove an ignore that is no longer needed (report it), never to add one.
- The triage table goes into the review request (§7), not into the repository.

## 4. Non-change scope (you must not touch)

- Any dependency other than `zip` and what `zip` itself pulls in. **No bare `cargo update`**, no other `-p` upgrades, no feature changes elsewhere.
- `edition`, `resolver`, lints, `.github/workflows/ci.yml` (CI must be fixed by making the facts true, not by editing the job), `rfcs/`.
- ZIP behaviour: entry names (`relative_path`), `Deflated` compression, timestamp logic, the all-or-nothing write from RFC 009, default file name.
- Adding any `ignore` to `.cargo/audit.toml`, or any `--ignore` flag anywhere. If something cannot be fixed, report; the architect and owner decide.

## 5. Required implementation

### 5.1 Minimum Rust version
1. Set `rust-version = "1.90"`; verify `resolver = "3"` and `edition = "2024"` are unchanged.
2. Update the README statement ("Requires Rust 1.90 or newer"). Grep the whole repository (excluding `target/` and `.git-exclude/`) for `1.85` and report each hit and what you did.
3. Prove it: `cargo +1.90 check --workspace --all-targets --all-features --locked` passes and `cargo +1.89 check ...` fails (record both; install toolchains with `rustup toolchain install <v> --profile minimal` if missing).

### 5.2 `zip` upgrade
1. Find the newest released `zip` whose advisories are fixed (`>= 9.0.0`) that builds with the 1.90 floor; the registry on the maintainer's machine contains 9.0.4 and 9.0.5 (`rust-version = "1.88"`). State the version you chose and why.
2. Update the requirement and the lock, with `default-features = false` kept and the **smallest** feature set that still provides Deflate for writing and reading in tests. Feature names may have changed between 2 and 9: read the crate's `Cargo.toml` `[features]` and choose; record the mapping.
3. Adapt the code only as far as the compiler requires. Do not change observable behaviour.
4. Run `cargo tree -i zip` and confirm a single `zip` version remains in the tree. If an older `zip` is still pulled in transitively, **stop and report** (the advisories would remain).
5. Compare the dependency graph before and after (`cargo tree` diff). Report new crates and new licences (the project is Apache-2.0; run `cargo deny check licenses` if a config exists, otherwise list any new crate's licence).

### 5.3 Audit
1. `cargo audit` after the change: the three `zip` vulnerabilities must be gone and **no new vulnerability** must appear. If a new vulnerability appears, stop and report.
2. Warnings: for each of the 14 (and any new) warnings produce a row: advisory id, crate and version, kind (unmaintained / unsound / yanked), which workspace crate depends on it (`cargo tree -i <crate>`), whether it is a **build-time** or **runtime** dependency, whether the code path is reachable in this application (give evidence, or say "unknown"), whether a fixed or replacement version exists within the semver range, and your recommendation (leave / upgrade / replace). Do not act on the table; the architect decides.
3. Check whether the two quick-xml ignores in `.cargo/audit.toml` are still needed (remove the ignore only if `cargo audit` stays clean without it, report either way).

## 6. Required tests and validation

- All existing tests still pass, in particular the ZIP tests from RFC 009 (`task_queue/zip_tests.rs` and the two timestamp tests). The upgrade must not need test changes beyond API renames; if a test must change meaning, stop and report.
- Add **one** new test only if the upgrade exposes behaviour not covered: for example that the produced ZIP is readable by the new `ZipArchive` and that entry names and timestamps survive. Otherwise add none.
- Gates on the final tree (report exactly what was and was not run):

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo audit
cargo +1.90 check --workspace --all-targets --all-features --locked
```

- **CI is part of the evidence.** After the last push, record the GitHub Actions run for that commit and the verdict of all five jobs (`fmt`, `clippy`, `test`, `msrv`, `audit`). Expected: all green. If any job is red, say which and why; do not edit the workflow to make it pass.
- Manual check: run the application once with an isolated config directory (`XDG_CONFIG_HOME` pointing at a scratch directory; never touch the maintainer's real settings), drop a PNG, enable the monochrome option, press "Download all (ZIP)", and verify the saved archive with `unzip -l` and `unzip -t`. Follow `.git-exclude/rules/project-instructions-gui-common.md` for GUI testing.

## 7. Acceptance criteria

- `rust-version = "1.90"`; README and every other minimum-version statement agree; `+1.90` check passes and `+1.89` fails (evidence).
- `zip >= 9.0.0`; `cargo audit` reports no vulnerabilities; single `zip` in the tree; no new vulnerabilities.
- Every gate green locally and all five CI jobs green on the final commit (or an honest report).
- ZIP behaviour unchanged (tests and the manual `unzip -t` evidence).
- Warning triage table delivered.
- No change outside §3; `edition`/`resolver` unchanged.

## 8. Prohibited shortcuts

- No ignores, no `continue-on-error`, no editing of CI to hide a failure, no loosening of lints.
- No upgrading unrelated dependencies "while you are there".
- No test deletion or weakening; no changes of expected values to match new library behaviour without reporting.
- No tags, releases, force-pushes or history rewriting.

## 9. Security and compatibility constraints

- The goal is a clean audit by upgrading; never by silencing.
- Raising the declared minimum Rust version is a compatibility statement; it corrects a false one (the locked tree never built below 1.90). Do not raise it above 1.90.
- Persisted settings and the ZIP layout are unchanged.

## 10. Known risks

- `zip` 9 may have renamed types, feature flags or the `DateTime` constructors; if the change is larger than a mechanical rename, stop and report with options before continuing.
- A transitive dependency may still require `zip` 2; see §5.2 item 4.
- A new compiler or dependency may produce new clippy lints; fix only those caused by the change.

## 11. Required review request

In `.git-exclude/review-request/rfc-024-dependency-baseline/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the commit range; the chosen `zip` version and reasoning; the feature mapping; the `Cargo.lock` change list; the `cargo tree` before/after summary; the MSRV probe results (+1.89 fail, +1.90 pass); audit output before and after; the warning triage table; the grep results for `1.85`; the CI run URL and the verdict of each job; the manual ZIP evidence (`unzip -l`, `unzip -t`); skipped checks with reasons; deviations; and open questions for the architect. Report to the owner only the path of the `README.md`.
