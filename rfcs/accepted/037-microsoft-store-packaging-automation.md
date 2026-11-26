# RFC 037 — Microsoft Store packaging: a version that cannot drift, then a build in CI, then (owner's choice) submission

**Status.** Accepted (architect, 2026-11-22, after the owner's remark of the same day: the Store manifest version "must be updated at every release"; "how about CI integration for automation of the update?"; the logo images on the Microsoft Store are being replaced with this release). Phase A: release 1.29.0 (job M1). Phases B and C: planned, gated (below).
**Priority.** P1 for phase A (a stale Store version blocks a Store update and is invisible until submission); P2 for B; C waits for the owner.
**Touches.** `packaging/windows/`, `scripts/`, `.github/workflows/` (phase A: one step in the existing `fmt` job; phase B: a new workflow), `packaging/README.md`, the release checklist in `ROADMAP.md`.

## Evidence (2026-11-22)

- `packaging/windows/AppxManifest.xml` is **hand-maintained**; its `Identity Version` was `1.26.2.0` through the releases 1.27.0 and 1.28.0 (the architect had left it as "the owner's decision"; nothing reminded anyone, and nothing in CI looks at it). `packaging/README.md`: "manual, unrelated to the AUR".
- CI runs on `ubuntu-latest` only; **no Windows build or test exists in CI**. The release notes of 1.27.0 and 1.28.0 say "Windows and macOS were not exercised".
- The four package tile images (`Assets/`) were replaced with the new logo in 1.28.0 (RFC 033). They are inputs of the package; **the Store listing images** (the app tile icon and the box art shown on the product page, screenshots) are separate uploads in Partner Center and are not part of the package.
- Code on `main` after the 1.28.0 tag is identical to the tag (`git diff 1.28.0..HEAD -- crates Cargo.*` is empty), so a package built from the commit that fixes the manifest contains exactly the 1.28.0 program.

## Phase A — the version is derived and guarded (job M1, 1.29.0)

1. **One source of truth:** `[workspace.package] version` in `Cargo.toml`. The Store version is that version **plus a fourth part `.0`** (`1.28.0` → `1.28.0.0`): the Store reserves the fourth part for its own use, so it stays `0`, and each submission must be higher than the last (Cargo's SemVer order gives that).
2. `scripts/appx-version.sh check` (exit 0 only if the manifest's `Identity Version` equals `<workspace version>.0`; the message names both values and the fix) and `scripts/appx-version.sh update` (rewrites that one attribute, nothing else). `grep`/`sed`/`awk` only, `set -u`, `local` variables, runnable from any directory; a test script with throw-away fixtures that shows a passing and a failing case for each rule (wrong version, wrong part count, a fourth part other than 0, missing attribute, an unrelated attribute untouched by `update`).
3. **CI:** the check runs as one step of the existing `fmt` job (before the toolchain install, like `check-rfcs.sh`; job names are a contract, no job is added). Effect: a version bump that forgets the manifest **fails CI on that commit**; the release candidate commit (version bump) must include the manifest update, and that cannot be forgotten.
4. **Release checklist** (`ROADMAP.md` §5): a line "the Store manifest version is current (the guard enforces it); tile images change only when the logo changes".
5. `packaging/README.md`: a short Windows section: the version rule, what is in the package, what is **not** (Store listing images), and that building and submitting are still manual.

## Phase B — a Windows build and an unsigned package in CI (spike first; planned with 1.30.0)

A workflow on `windows-latest` (on `workflow_dispatch` and on a pushed version tag): `cargo build --release --locked`, stage the package layout (the executable, `Assets/`, the manifest), run `makeappx pack` (the Windows SDK is on the runner), upload the `.msix` as a workflow artifact for the owner to download. **Unsigned is correct for the Store** (Partner Center signs on ingestion); no certificate, no secret. It also gives the project its **first Windows build and test evidence** (`cargo test` on Windows). The spike must first answer: does the program build and pass on `windows-latest` unchanged (the portal probe and the `cfg` gates were written but never compiled for Windows), what is the runner's SDK path, what does the Store's validation (`MakeAppx` and the Windows App Certification Kit's static checks) say about the layout. Hash and size of the artifact are printed in the log.

**Phase B result (B1, 2026-11-26; reviewed).** `windows-latest` (Windows Server 2025): the program **builds unchanged** and `cargo +1.90 check` passes; **522 of 523** Windows tests pass (the one failure is a CRLF-dependent test of ours, fixed in job B2 together with a `.gitattributes`); `makeappx pack` with validation prints only "Package creation succeeded" for the layout (executable, four tiles, manifest); the unsigned `.msix` is a workflow artifact (about 9.4 MB). The workflow is separate and non-blocking (`.github/workflows/windows-package.yml`). Not done, by decision: running `appcert.exe` (an unsigned package cannot be installed, so most of its tests cannot run), a build cache, reproducible builds. Not verified: the tag trigger (until the 1.30.0 tag; `workflow_dispatch` is the fallback) and any Windows **runtime** behaviour. **B2 (2026-11-27; reviewed):** the test fixed and a second, silent weakness of a source guard found and fixed (both CRLF-dependent); `.gitattributes` (`* text=auto eol=lf`, the Store manifest `-text`); the Windows test step is strict; **Windows: 525 tests passed, 0 failed**, package packed; `workflow_dispatch` verified. Still unverified: the tag trigger (the 1.30.0 tag) and Windows runtime behaviour.

## Phase C — submission automation (the owner's decision; not planned)

Submitting from CI (the Microsoft Store submission API or the `msstore` tool) needs an Azure AD application's credentials (tenant, client id and secret) as repository secrets: **the owner's account and secrets, the same boundary as the AUR key (RFC 020 D4)**. Recommendation if the owner wants it: a GitHub **environment** with required reviewers, the credentials only there, CI creating a **draft submission** that the owner reviews and submits in Partner Center (certification stays a human decision, and a wrong submission cannot go live unattended). Store listing images (the app tile icon, box art, screenshots) can be derived from the master by Logolig's own exporter and kept in `packaging/windows/store-listing/` for upload; their sizes and rules are taken from Partner Center's current requirements by the owner or the architect before any job (not guessed here).

## Not in scope

Signing certificates; the Store listing text; changing `Publisher` or `Name` (they are the Store-assigned identity); a Windows installer outside the Store; macOS.

## Risks

Phase A: none beyond a stricter CI (the failure message says how to fix it). Phase B: the first Windows build may reveal real defects (that is the point); keep it a separate, non-blocking workflow until it is green. Phase C: secrets and an outward action; owner-gated.
