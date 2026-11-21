# Developer Handoff — RFC 037 phase A: the Store manifest version is derived and guarded (job M1)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; relative paths with `cd`; English only). Governing RFC: [RFC 037](../../accepted/037-microsoft-store-packaging-automation.md), phase A. **Shell and one CI step only: no Rust, no `Cargo` change.** Model: `scripts/check-rfcs.sh`, `scripts/check-rfcs-test.sh` and the J16 review. **Do this before K1** (it is small and the manifest drifted twice already). The architect has already set `packaging/windows/AppxManifest.xml` to `Version="1.28.0.0"` (code identical to the 1.28.0 tag).

**Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; two commits (script and test; then the CI step and docs); CI waited for each (the Packaging workflow also runs when `packaging/**` changes: it must stay green); no tags, force-push or history rewriting.

## 1. `scripts/appx-version.sh`

`check` and `update` modes (`appx-version.sh check [ROOT]`, `update [ROOT]`; optional ROOT only so the test can point at throw-away trees); `set -u`, every helper variable `local`, `grep`/`sed`/`awk` only, runnable from any directory, under about 60 lines of code. Rules:
- The workspace version is the `version = "…"` line of `[workspace.package]` in the root `Cargo.toml` (three numeric parts; anything else is an error naming the file).
- The manifest value is the `Version="…"` attribute of the **`<Identity …>` element** (not any other attribute named `Version`; the manifest has `MinVersion`/`MaxVersionTested` attributes which must not match).
- Expected value: `<workspace version>.0`. `check` prints `appx-version: ok (1.28.0.0)` or `packaging/windows/AppxManifest.xml: Identity Version is X, expected Y (run: bash scripts/appx-version.sh update)` and exits non-zero. `update` rewrites **only that attribute's value**, preserving everything else byte for byte (show `git diff --stat` has one line changed), and is idempotent.

## 2. `scripts/appx-version-test.sh`

Throw-away trees under `mktemp -d` (created and removed by the test; nothing written in the repository). Cases, each with the expected message or result: version in sync → pass; stale manifest → fail with the message naming both values; a fourth part other than `0` → fail; a three-part manifest version → fail; missing `Identity Version` → fail with a clear message; a malformed workspace version → fail; `update` fixes a stale manifest and then `check` passes; `update` leaves `MinVersion`/`MaxVersionTested`, the publisher, the tile paths and every other byte untouched (compare with `cmp` against the expected file); `update` twice changes nothing the second time. The last case runs `check` on the real repository (it passes now).

## 3. CI and docs (second commit)

One new step in the existing `fmt` job of `.github/workflows/ci.yml`, **right after the "RFC index and links" step and before the toolchain install**: name "Store manifest version", `run: bash scripts/appx-version.sh check`. **Do not add or rename a job.** Show the CI log with the step passing; show it failing once **on a throw-away copy** (not pushed red). `packaging/README.md`: a Windows section stating the version rule (`<workspace version>.0`, the fourth part is the Store's), what the package contains (executable, `Assets/`, manifest), that the **Store listing images are separate uploads and not part of the package**, and that building and submitting are still manual (RFC 037 phases B and C). The `ROADMAP.md` release checklist line is the architect's, not yours.

## 4. Evidence, acceptance, prohibited

Review request: `.git-exclude/review-request/rfc-037-appx-version/README.md` (report only its path): the cases and their outputs, the one-line `git diff --stat` of `update` on a stale copy, CI run ids, the packaging workflow result. Acceptance: script and test pass; the real tree passes; the step is in CI and green; no job renamed; nothing but the manifest's one attribute is ever rewritten. Prohibited: any change to `Publisher`, `Name`, the tiles or other manifest content; Rust or `Cargo` changes; network access; tags, force-push or history rewriting.
