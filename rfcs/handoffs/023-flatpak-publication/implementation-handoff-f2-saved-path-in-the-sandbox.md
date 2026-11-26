# Developer Handoff — Flathub sandbox findings, job F2: the saved-file message must not show a portal path

Role and rules as always (commits on `main`, gates chained with `&&` and their exit status tested before commit and push, `git log origin/main..` before every push, wait for CI on each commit, relative `cd`). No version change, no tag, no release.

## The defect (found by the owner in a real Flatpak run, 2026-11-26)
After saving ("Download" one file, or "Download all (ZIP)"), the success message shows the path **as the file chooser returned it**. Inside a Flatpak the chooser is the portal, and the path it returns is a document-portal path: `/run/user/<uid>/doc/<document id>/<file name>`. The file is saved where the user chose, but that path means nothing to the user (it is not in their home directory, and it does not exist outside the sandbox). A message that names a place the user cannot find misleads, and "no confusing or misleading UI" is the owner's rule.

The code (`crates/app/src/app/update/download.rs`, `download_one_completed` and `download_all_completed`): `let body = format!("{}", path.display());`.

## Decision (mine; the owner can overrule)
**Show only the file name when the path is a document-portal path; otherwise show the full path exactly as today.** Why the file name and not the real folder: the real host folder is not known to the sandboxed process without asking the Documents portal again (a feature of `ashpd`, a new surface and a portal version dependency). The user has just chosen the folder in the chooser, so the name alone is true and enough. **Out of scope and parked:** asking the portal for the host path. No new package, no new feature, no `unsafe`, no `#[allow]`.

## Work
1. **Find every place the application shows a file path to the user** (grep `display()`, toasts, error messages that carry a path; the technical details inside `AppError` strings in `task_queue.rs` are for the log and are not "shown" unless a view renders them: check which ones a view renders, and report the list). Fix only what the user can see. Report anything else you find; do not widen the job.
2. **One small pure function** (in the app crate, with the other path helpers or next to `download.rs`; your choice, reported): `path_for_message(path: &Path, runtime_dir: Option<&Path>) -> String`, which returns the file name when `path` lies under `<runtime_dir>/doc/` and the full `path.display()` otherwise. `runtime_dir` is `XDG_RUNTIME_DIR` read by the caller (the function itself reads no environment, so it is testable). If `XDG_RUNTIME_DIR` is unset or empty, fall back to the literal prefix `/run/user/<digits>/doc/`. A path without a file name (cannot happen from the chooser) falls back to the full path. Linux logic only; on Windows and macOS the function returns the full path unchanged and a test says so (`cfg` as the code base already does it, no `#[allow]`).
3. **Use it in both completed-handlers** (and in any other place item 1 finds).
4. **Docs:** one sentence in `docs/src/functional-spec.md` where the success message is described (or the nearest place that says what it shows): the file name inside Flatpak, the full path elsewhere.

## Tests (red first, on the unchanged code)
- Headless: `download_one_completed` and `download_all_completed` with a portal path (`/run/user/1000/doc/ab12cd/favicons.zip`): the toast body is `favicons.zip`; red on the head (it shows the full path).
- Unit tests of `path_for_message`: portal path with the runtime dir given; with another runtime dir (`/tmp/rt/doc/x/f.png` under `Some("/tmp/rt")`); `XDG_RUNTIME_DIR` unset (the literal prefix); `/run/user/1000/docx/…` (not the portal: full path kept); `/home/u/doc/x/f.png` (full path kept); a file name with spaces and non-ASCII characters; the `by-app` form (`/run/user/1000/doc/by-app/io.github.nabbisen.Logolig/ab12/f.png`: name only).
- One mutation (the prefix check inverted or removed) shown to fail a test. The existing tests stay green; list any you had to change and why.

## Prohibited
A real-window run (the owner verifies in the Flatpak; I will not ask for a run), a Flatpak manifest or any Flatpak file, new packages or features, changes to the portal/file-chooser flow, version changes, tags, releases, publishing, edits to `rfcs/`, `ROADMAP.md` edits, `#[allow]`, absolute `cd`.

## Delivery
`.git-exclude/review-request/flathub-f2-saved-path/README.md` (report only its path): commits and CI, gates, the list from item 1, red → green, the mutation, test-list diff, anything not verified.
