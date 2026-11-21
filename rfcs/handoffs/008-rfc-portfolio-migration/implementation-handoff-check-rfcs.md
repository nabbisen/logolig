# Developer Handoff — RFC 008 remainder: `scripts/check-rfcs.sh` and its wiring (job J16)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; relative paths with `cd`; English only). Governing texts: [RFC 008](../../done/008-rfc-portfolio-migration.md) (step 5), [RFC 000](../../done/000-rfc-lifecycle-policy.md) "Optional CI invariants", [RFC 020](../../accepted/020-ci-and-aur-packaging.md) (CI job names are a contract). Review of J7f: `.git-exclude/reviewed/rfc-033-new-application-logo-review-2026-11-21.md`. **Shell only (plus one workflow step): no Rust, no `Cargo` change.**

Owner philosophy: clean, safe and robust; a check that cannot fail is not a check; "none of them need sophisticated parsing: `grep`, `find` and basic shell suffice" (RFC 000), so keep it small and readable.

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; two commits (script, then wiring); CI waited for each; no tags, force-push or history rewriting.

## The script: `scripts/check-rfcs.sh`

`set -u` (and `set -e` where it helps), every helper variable `local`, runnable from any directory (it finds the repository root from its own path), exit 0 only when every check passes, every failure printed as `file: what is wrong`, a final count. The invariants (RFC 000, adapted to the 5-folder layout):

1. **Status matches the folder:** the first word of the `**Status.**` field of each RFC under `rfcs/{proposed,accepted,done,archive}/` is `Proposed`, `Accepted`, `Implemented`, and (`Withdrawn` or `Superseded`) respectively. (Draft folders do not exist here.)
2. **Filename:** `NNN-slug.md` (three digits, lowercase slug with hyphens).
3. **No RFC number appears in two folders.**
4. **Index both ways:** every RFC file is linked from `rfcs/README.md`, and every RFC link in `rfcs/README.md` points to an existing file **in the folder the link names** (the link's folder is the state; it must equal the file's actual folder).
5. **Handoffs:** every `rfcs/handoffs/NNN-slug/` directory has a matching RFC number in some folder (`release-1.27.0` and similar non-numbered folders are an explicit allow-list inside the script, commented), and no handoff directory contains `proposed/`, `accepted/`, `done/` or `archive/` subfolders.
6. **Relative links resolve:** in every Markdown file under `rfcs/`, `docs/src/` and in `README.md`, `ROADMAP.md`: a link or image target that is not a URL, not an anchor-only `#…` and not `mailto:` must exist; **a leading `/` means the repository root** (RFC 033 decided this for the README), otherwise relative to the file; a trailing `#anchor` is ignored (anchors are not checked: say so in the header comment); targets inside fenced code blocks are ignored. Reuse the logic of J15's `link-check.py` idea, but in the script's own language (shell with `grep -o`/`sed`; if that proves unreadable, a ten-line `python3 -I` helper called by the script is acceptable: say why).

## Tests of the checker (it must be able to fail)

A test script `scripts/check-rfcs-test.sh` that builds throw-away copies of a **tiny fixture tree** under a `mktemp -d` directory (created and removed by the test; no files in the repository) and shows, for each of the seven invariants, one case that passes and one that fails with the expected message: a wrong Status word; a bad filename; a duplicated number; an RFC missing from the index and an index link to a missing file and a link naming the wrong folder; an orphan handoff directory and a handoff with a `done/` subfolder; a broken link, a broken `/`-rooted link, a link inside a code fence that must be ignored. Then run the real script on the repository: it must pass now (if it finds real problems, **list them and fix only documentation/index problems**; report anything that needs a decision).

## Wiring (second commit)

Add **one step** to the existing `fmt` job of `.github/workflows/ci.yml`, named "RFC index and links", running `bash scripts/check-rfcs.sh`. Do **not** add or rename a job (job names are a contract: branch protection refers to them). The step must run before `cargo fmt` so the failure is quick. Show the CI run; show the step failing once **on a throw-away copy** or by reasoning from the test (do not push a red commit).

## Evidence, acceptance, prohibited

Review request: `.git-exclude/review-request/rfc-008-check-rfcs/README.md` (report only its path): the invariants table with the test case for each; the output of the real run; any real problem found and how it was resolved; CI run ids. Acceptance: the script and its test pass; the real tree passes; the step is in CI and green; the script is under about 120 lines (excluding comments), readable. Prohibited: any change outside `scripts/`, `.github/workflows/ci.yml` and (only to fix a reported real problem) `rfcs/` and `docs/` text; a renamed or added CI job; network access; tags, force-push or history rewriting.
