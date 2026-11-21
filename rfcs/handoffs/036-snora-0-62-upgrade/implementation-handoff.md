# Developer Handoff — RFC 036: upgrade snora 0.57 → 0.62 (job J13a)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*` — read the **Test Optimization Rules** again; relative paths with `cd`; English only). Governing RFC: [RFC 036](../../done/036-snora-0-62-upgrade.md). Review of the spike: `.git-exclude/reviewed/rfc-027-focus-spike-review-2026-11-15.md`.

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before the push; gates chained (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit 0 / 11); CI waited for; no tags, force-push or history rewriting.

## Evidence plan

| Question | Level | Evidence |
|---|---|---|
| Does it build and pass with no code change? | gates | the architect's trial says yes; confirm and show |
| Which packages change? | recorded state | `git diff --stat Cargo.lock`; list every changed package with versions; expected: only `snora`, `snora-core`, `snora-interaction`, `snora-widgets` |
| Is the look unchanged? | headless, machine comparison | snapshots of 5 screens × 2 themes at 1280 × 720 on 0.57 and 0.62 with the **same** test code and the hermetic fonts; byte compare (`cmp`); describe and judge any difference by looking only at the pictures that differ |
| Is the release dependency tree otherwise unchanged? | recorded state | `cargo tree -e normal -p logolig-app` before/after, differences listed |

No real-window run in this job (the release run covers it).

## Work (one commit; if a code change turns out to be needed, a second commit for it, explained)

1. Change `snora` to `"0.62"` in the workspace `Cargo.toml`; `cargo update -p snora` only (never a general update); show the lock diff.
2. Snapshot comparison: write the snapshot tests as a **temporary, uncommitted** file (patch kept in the review-request folder); do not commit golden images. Take them first on `5cccf0c`'s tree (0.57), then on the upgraded tree.
3. Gates; push; CI.

## Required evidence (review request: `.git-exclude/review-request/rfc-036-snora-0-62/README.md`; report only its path)

The table above filled in; the lock diff; the `cmp` results and any picture that differs with your description; gates and CI run id; anything in snora 0.58–0.62 you noticed that touches code we use (read the crates' docs/sources; the registry holds all versions).

## Acceptance / prohibited

Acceptance: build and tests green with at most trivial code changes (each explained); only `snora*` packages change in the lock; snapshots identical or every difference judged harmless by you and the architect. Prohibited: any use of `snora::control`, `focus_zone` or other new API in this job; general `cargo update`; committed golden images; new `#[allow]`.
