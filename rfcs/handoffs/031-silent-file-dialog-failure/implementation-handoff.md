# Developer Handoff — RFC 031: one dialog at a time, and a notice when no dialog can appear (job J12b)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (read the **GUI Evidence and Automation Rules** and **Test Optimization Rules** again; Rust 2024, 2018+ module style without `mod.rs`, tests separated from implementation, English only, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 031](../../done/031-silent-file-dialog-failure.md), section **"Decisions after the spike"** (D1–D5 are binding). Spike: `.git-exclude/review-request/rfc-031-dialog-spike/` and its review `.git-exclude/reviewed/rfc-031-dialog-spike-review-2026-11-15.md`. Reuse `session-lib.sh`, `private-bus.sh`, `portal-stub-never-answers.py` from the spike's `evidence/` (copy what you need into your own review-request folder; do not rewrite them). On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused. **A false notice on a working system is worse than the silent failure**: when in doubt, do nothing new.

**Workflow:** `main`; `git pull --ff-only` first; **`git log origin/main..` before every push** (shared checkout); one commit per step; gates chained with `&&` (`cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit && cargo doc --workspace --no-deps --locked`; four existing doc warnings; audit 0 / 11); wait for each commit's CI before the next push; never red; no tags, force-push or history rewriting.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| A second request for any dialog (button, shortcut, either Save kind, the chooser) does nothing while one is open; every answer re-enables | logic and state | `update()` tests, shown failing before the change |
| The verdict (show the notice or open the dialog) is right for every combination of the three facts, for a sandbox, and for a failed or slow probe | logic (pure function on hand-made facts) | table test over all combinations |
| Real: a double Ctrl+S opens **1** window | real UI, one run | as the spike's Q1 script |
| Real: no portal (private bus) and zenity absent → one toast, no window; frontend without backend (real `xdg-desktop-portal`, as spike Q4) → one toast, no window; the owner's real bus → no toast and the chooser opens | real UI, one script, three scenarios | analysis after the run |
| `zenity` present and portal missing → **no** toast (the false-notice guard) | logic (the verdict table) and, if you can put a harmless `zenity` stub on a scratch `PATH`, one real scenario | the spike did not run this: say what you did |

Captures: at most one per scenario, analysed after the run; windows counted with `niri msg -j windows`. Owner's real `settings.json` mtime before and after.

## 1. Step 1 — D1: one dialog at a time (one commit; push; wait for CI)

1. Rename `AppState::chooser_open` to `dialog_open` (doc: "a file dialog (the chooser or a Save dialog) is open and has not answered yet; while it is set a further dialog request does nothing; cleared by the dialog's answer, never by a drop"). Rename in production code, `job_tests.rs`, `shortcuts_tests.rs`; **test names stay**, list every test whose body changed only by the field name.
2. Guard (check, set, start the task) in `pick_file_requested`, `download_one_requested` and the `DownloadAllRequested` arm (make it a handler like the others, `update/download.rs`; body unchanged apart from the guard). Clear the flag in **every answer**: `FilePicked(None)`, `FilePicked(Some)`, both arms of `DownloadOneTargetPicked` and of `DownloadAllTargetPicked` (find the real message names with `rg`; list them in the review request). A drop never clears it.
3. Tests first, failing before: second `DownloadAllRequested` / `DownloadOneRequested` / Ctrl+S returns no task; a Save request while the chooser is open does nothing, and the reverse; every answer (`None` and `Some`, all four Save arms, both chooser arms) re-enables; a drop leaves the flag alone. Existing download tests keep their names and pass.
4. Real run (one): double Ctrl+S gives **1** window; Esc closes it; Ctrl+S again gives 1 window.

## 2. Step 2 — D2/D3: the notice (one commit; push; wait for CI)

1. **Dependency:** `ashpd = { version = "0.11", default-features = false }` in `crates/app/Cargo.toml`, under the same `[target.'cfg(any(target_os = "linux", target_os = "freebsd", target_os = "dragonfly", target_os = "netbsd", target_os = "openbsd"))'.dependencies]` as `rfd` uses. Show `git diff --stat Cargo.lock` (target: one line, no package), `cargo tree -e normal` package set unchanged, MSRV 1.90 check, `cargo audit` 0 / 11.
2. **New file `crates/app/src/portal.rs`** (cfg-gated; an always-present stub for other platforms that returns "unknown", so the callers have no cfg noise): `Facts { portal: bool, backends: usize, zenity: bool, sandboxed: bool }`, an async `probe() -> Option<Facts>` (`None` = the probe failed or took more than 2 s: use `tokio::time::timeout` or the equivalent already in the tree, no new package), and a **pure** `verdict(&Facts) -> Verdict { DialogAvailable, NoDialog }`: `NoDialog` only if `!zenity && (!portal || (!sandboxed && backends == 0))`. Names counted: owned **or** activatable. Sandbox detection: `/.flatpak-info` exists, or `SNAP` set (read-only checks, no dependency). `zenity` on `PATH`: a plain lookup of the executable bit on each `PATH` entry.
3. **Call site:** in the three dialog tasks of `task_queue.rs`, before `rfd`: probe; if `Some(facts)` and `verdict == NoDialog`, deliver a new message `DialogUnavailable` (which clears `dialog_open` and pushes **one** error toast through the existing toast machinery) instead of opening the dialog; otherwise open it as today. A failed probe never changes behaviour.
4. **Texts** (en and ja, in the i18n crate with the existing key-parity test): chooser variant: "No file dialog is available on this system. Drag a file onto the window instead, or install a desktop portal with a file dialog (for example xdg-desktop-portal-gtk)." and a Save variant: "No save dialog is available on this system. Install a desktop portal with a file dialog (for example xdg-desktop-portal-gtk)." Do not claim more than the probe knows. Have the Japanese checked for naturalness against the neighbouring strings.
5. **Tests:** the verdict table over **all 2⁴ combinations** (including both sandbox values), a test that a failed probe leaves the dialog request unchanged, a test of the message handler (flag cleared, exactly one toast), i18n parity.
6. **Real run (one script, three scenarios, no repeats):** (a) private bus without portal and no zenity → Ctrl+O: toast, 0 windows, `dialog_open` cleared (a second Ctrl+O gives a second toast, not a stack: check what the toast machinery does and report); (b) real `xdg-desktop-portal` without backend (spike Q4 setup): toast, 0 windows; (c) the owner's real bus (read only): chooser opens, no toast. If you can supply a harmless `zenity` stub on a scratch `PATH` for (a), add scenario (d): no toast.

## 3. Step 3 — D4/D5: documentation (one commit)

User docs (`docs/src`, the page that describes choosing and saving files) and the packaging guide (`packaging/`): the known limit from D4 in one sentence each. The AUR template `packaging/aur/logolig/PKGBUILD.in` `optdepends` line for `xdg-desktop-portal-impl` rewritten per D5 (also check `packaging/aur` for a `logolig-bin` template and apply the same wording; `scripts/render-aur.sh` still renders it, shown). RFC 018 syncs the rest later.

## 4. Required evidence (review request: `.git-exclude/review-request/rfc-031-dialogs/README.md`; report only its path)

§0 table filled in; the failing-before lists; the verdict table; the `Cargo.lock` diff, tree comparison, MSRV and audit results; the real-run logs and analyses; the toast machinery behaviour on repeats; the test-list diff (named additions; adapted tests listed); gates and CI per commit by id; real-config mtime.

## 5. Acceptance criteria

Any second dialog request is ignored while one is open and every answer clears the flag; a double Ctrl+S gives one window; the notice appears only with positive evidence (table-tested), never on a probe failure, never in a sandbox for the backend check, never when `zenity` exists; the dialog is not opened when the notice is shown; no new package; nothing outside `crates/app/src/{app*,task_queue*,portal*}`, the i18n crate, `Cargo.toml` of the app, `docs/`, `packaging/` changed; gates and CI green at every pushed commit.

## 6. Prohibited shortcuts

No timeout that clears `dialog_open`; no timing heuristic for cancel versus failure; no direct `ashpd` portal calls to open dialogs (design B); no new `#[allow]`; no behaviour change when the verdict is `DialogAvailable`; no probe on the UI thread; no start or stop of services on the owner's real bus; no tags, force-push or history rewriting.
