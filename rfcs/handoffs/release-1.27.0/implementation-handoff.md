# Developer Handoff — Release 1.27.0 preparation (ROADMAP job J6)

Role and rules: as in the earlier handoffs (`.git-exclude/roles/`, `.git-exclude/rules/`; GUI rules in `.git-exclude/rules/project-instructions-gui-common.md`, revised 2026-10-25, authoritative). **Workflow: directly on `main`, one job at a time, `git pull --ff-only` first, gates chained with `&&`, no red push.** This job has no RFC: its authority is `ROADMAP.md` §2 (authorized by the owner on 2026-10-21; the version 1.27.0 is the architect's recommendation, **the owner authorizes the tag and the publication separately**). **You do not tag, release, publish, push to the AUR, or edit `rfcs/` or `ROADMAP.md`.**

## 1. Purpose

Make the tree a verified release candidate for 1.27.0 and write honest release notes, so the owner can decide on the tag with evidence.

## 2. Scope (you may change)

1. **Version.** `[workspace.package] version` in the root `Cargo.toml`: `1.26.2` → `1.27.0`. Update `Cargo.lock` so that **only** the three workspace packages (`logolig`, `logolig-i18n`, `logolig-app`) change their version (for example `cargo update --workspace`, or a `cargo check` without `--locked`); show the lock diff and confirm it has no other line. Do **not** touch `packaging/windows/AppxManifest.xml` (the Store version is a separate owner/Store decision; report its current value `1.26.2.0` and that it was left).
2. **Documentation that states the current version.** `docs/src/functional-spec.md` line 4 ("v1.26.2" as the described version) and the `docs/src/architecture.md` version table: add a v1.27.0 row in the table's existing style (summary from the notes below, no invention); `docs/src/ui-a11y.md` line 33 says "verified in phase 1 on release 1.26.2 and have not changed": re-check the statement is still true and word it correctly (the shortcuts are new; say so in one sentence) rather than only bumping the number. Check `docs/src/export-spec.md` and `functional-spec.md` against the reviewed jobs below and report (do not silently fix beyond a sentence) anything that disagrees with the behaviour.
3. **Release notes.** New file `docs/release-notes/1.27.0.md` (English). Structure: Highlights (3–6 bullets), Changes by area (Output, Conversion, Keyboard, Look, Packaging, Under the hood), Known limitations, Upgrade notes. Content must come from the reviewed jobs only (files in `.git-exclude/reviewed/` and the RFCs named below); **do not invent features**. It must include:
   - Non-square logos are now shown in full (contain-fit) in square outputs, with an advisory when the sides differ by more than 2 % (RFC 010); the favicon of a non-square logo looks different from 1.26.x. ZIP/result integrity: the result you download is the one you see (RFC 009).
   - SVG sources: images referenced from an SVG are no longer read from the computer; text inside an SVG is not drawn yet; one advisory says so (RFC 015 phase 2a).
   - Tracing of `favicon.svg` is limited to 512 px on its longest side; a large flat logo's SVG declares 512 px (RFC 030); conversions of large or photographic images are fast and no longer freeze the window's shortcuts.
   - Keyboard: Cmd/Ctrl+O, Cmd/Ctrl+S, Esc, Cmd/Ctrl+1–3, F1 and the shortcut list in Settings (RFC 019). Limit: full Tab/focus navigation is **not** in this release (RFC 027, planned); a focused text field takes Ctrl+1 as typing.
   - Look: toasts are redrawn (snora 0.53), an overlay no longer blocks clicks behind it; toast lifetime now follows its length (advisories stay 12–20 s) (RFC 028, 029).
   - Window application ID `io.github.nabbisen.Logolig` (RFC 025).
   - Packaging: AUR source package `logolig` prepared (desktop entry, icons); it needs a system font and, for the file chooser, an XDG desktop portal with a backend (`xdg-desktop-portal-gtk` or your desktop's); without one the file chooser does nothing and says nothing (known, RFC 031 proposed).
   - Under the hood: minimum supported Rust 1.90, dependency updates (zip 9, tokio 1.54, snora 0.53), `cargo audit` 0 vulnerabilities with 12 triaged warnings, CI with five jobs plus a packaging job.
   - Known limitations: the Japanese texts added in this release have **not had a native review**; SVG text is not drawn; Windows and macOS were not exercised in this cycle.
   If any statement above is not supported by a review file when you check it, **leave it out and report**.
4. Nothing else. In particular no code change.

## 3. Verification (evidence to deliver)

1. Final gates, chained: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit` on the release-candidate commit. Expected: 253 tests, audit 0 vulnerabilities and 12 warnings.
2. CI on the pushed commit: the five `ci.yml` jobs and the `aur-source` job (the version bump does not touch `packaging/`, so the packaging workflow may not run: say so, and run it with `workflow_dispatch` only if the push did not trigger it and you were able to; otherwise report).
3. **Candidate package build:** render and build the package from a local archive of the candidate commit exactly as `packaging/README.md` describes (`scripts/render-aur.sh logolig 1.27.0 <outdir> --tarball <archive>` — the tarball path, because the tag does not exist; then `makepkg -f`, `namcap` on the PKGBUILD and the package, the exact file list). Do not install it.
4. **Release-build smoke test** (native Wayland, `wtype`, rules above; one run, proportionate): launch the release binary; open a PNG through Ctrl+O; the Result screen appears; Cmd+2 and Cmd+3 change the page; Esc returns; the shortcut list is visible in Settings. Record the window size it opened with; one capture per question at most.
5. A grep that no documentation or README text claims something the notes list as a limitation (for example "accessible" must remain the README's wording; the owner decided so; do not change it).

## 4. Prohibited

Tags, GitHub Releases, pushes to the AUR, edits to `rfcs/`, `ROADMAP.md`, `packaging/windows/`, `crates/` (code), claims in the notes without a review file behind them, "improving" the README install section (J7, after publication).

## 5. Delivery

Small commits directly on `main`; review request at `.git-exclude/review-request/release-1.27.0-candidate/README.md` with: the commit range, the lock diff, gate logs, CI run and per-job conclusions, the package build and `namcap` output, the smoke-test record, the list of documentation sentences you changed and why, anything you left out of the notes and why, deviations, skipped checks, open questions. Report to the owner only the path of that README.
