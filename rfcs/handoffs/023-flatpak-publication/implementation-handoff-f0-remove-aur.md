# Developer Handoff — RFC 020/023 job F0: remove the AUR packaging (the owner dropped AUR)

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; English only; **relative paths with `cd`, or none; never an absolute `cd`**). Governing texts: [RFC 020](../../done/020-ci-and-aur-packaging.md) "Amendment 2026-11-22" and [RFC 023](../../accepted/023-flatpak-publication.md) "Decisions 2026-11-22". Review of K1: `.git-exclude/reviewed/rfc-014-ingest-final-review-2026-11-22.md`. **Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit; gates of a non-code commit (`cargo fmt --check`, `cargo doc --workspace --no-deps --locked`, `bash scripts/check-rfcs.sh`) plus CI waited for; no tags, force-push or history rewriting.

## Why

The owner will publish on Flathub and not on the AUR. The AUR files were verified and are harmless, but a distribution path nobody uses rots, costs a CI container job and confuses readers ("is there an AUR package?"). History keeps them.

## Work (one commit, evidence by `git diff --stat` and greps)

1. **Remove:** `packaging/aur/` (the `PKGBUILD.in`), `scripts/render-aur.sh`, and `.github/workflows/packaging.yml` (it only checks the AUR package; **F1 adds a new validation workflow**, do not create it here).
2. **Keep:** `packaging/linux/` (the desktop file and the hicolor icons: the Flatpak upstream files), `packaging/icons/` (the logo master and its provenance), `packaging/windows/` (the Store), `scripts/appx-version.sh`, `scripts/gui-*.sh`, `scripts/check-rfcs*.sh`.
3. **`packaging/README.md`:** rewrite to what is true: what is in `packaging/` (Linux upstream files: desktop file, icons; the logo master; the Windows Store manifest and tiles), a Windows section as now, and **one paragraph "Flathub"**: RFC 023, the manifest and the submission are the owner's, upstream files and validation are in this repository (the validation arrives with F1; say so in the future tense only if you cannot avoid it, else leave the sentence to F1). Remove every AUR instruction and the `makepkg`/`namcap`/clean-chroot text.
4. **`packaging/icons/README.md`:** any sentence that mentions the AUR package file list is updated (the Linux icons are used by the Flatpak and by `cargo install` users' desktop integration if any: say only what is true).
5. **Documentation:** `rg -n -i '\baur\b|pkgbuild|makepkg|logolig-bin|render-aur'` over `README.md`, `docs/src`, `packaging`, `scripts`, `.github`: fix every hit that describes a current feature (release notes of 1.27.0 and 1.28.0 are history: leave them; `ROADMAP.md` and `rfcs/` are the architect's: leave them). `docs/src/architecture.md`: the release table rows are history; the packaging description in the workspace section, if it mentions the AUR as current, is corrected.
6. **CI:** after the removal the `Packaging` workflow no longer exists. State in the review request that the five `ci.yml` jobs are the whole CI until F1 adds a validation job; check that no other file refers to the removed workflow (branch-protection names live on GitHub, not here: say that the owner may need to remove `aur-source` from required checks if it was ever required, and that you cannot see it).
7. `bash scripts/check-rfcs.sh` must pass (links into `packaging/aur/` from RFCs or handoffs would break: it will list them; those documents are history: **do not edit RFCs or handoffs**; if the checker fails on a link to a removed file, report it and the architect adjusts the RFC text).

## Evidence, acceptance, prohibited

Review request `.git-exclude/review-request/rfc-020-remove-aur/README.md` (report only its path): `git diff --stat`, the grep before and after, the checker output, CI run id. Acceptance: no AUR file or instruction remains in the current tree; the checker, fmt, doc and CI are green; `packaging/linux/` untouched. Prohibited: touching `packaging/linux/`, `packaging/windows/`, `crates/`, RFCs, handoffs, `ROADMAP.md`; adding a Flatpak manifest or any file under `packaging/flatpak/` (the manifest is the owner's, written without AI assistance; the dev team never writes it); tags, force-push or history rewriting.
