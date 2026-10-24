# Developer Handoff — RFC 028: Upgrade snora from 0.25.2 to 0.53.0

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 028](../../accepted/028-snora-upgrade-0-53.md) (Accepted; owner authorized it into release 1.27.0 on 2026-10-21). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** You are the only job running. Work directly on `main`: `git pull --ff-only` first, small commits (short imperative subject), push after each commit that passes the local gates. Never rewrite history or force-push. Do not tag or release.

**Position in the schedule:** job J2b in `ROADMAP.md`, after J1 (RFC 024) and J2 (RFC 025), which must be reviewed first. Check that `rust-version` in `Cargo.toml` is `1.90` and that the CI workflow exists; if not, stop and report.

## 1. Purpose

Move the GUI toolkit layer from snora 0.25.2 to 0.53.0 with the same features, so the first public release carries the upstream fixes (overlay/toast click-through, AA-contrast toasts) and so the keyboard work can use snora's F6 zone navigation. Keep behaviour identical apart from what the upgrade itself changes, and document every visible difference.

## 2. Background (verified facts)

- Upstream's migration guides are in `.git-exclude/upstream/snora/receive/0.53.0/snora-0.25.2-to-0.53.0-migration-guides-for-logolig-2026-10-21/` (`README.md` and `guides/`). **Read the README and the guides marked "may break the build" or "behaviour change"** (0.32→0.33, 0.39→0.40, 0.40→0.41, 0.41→0.42, 0.44→0.45) before you start, and `migration-0.52-to-0.53.md`. Pages of the online book are at <https://nabbisen.github.io/snora/> if a guide links to one that is missing.
- Pinned today: `snora = { version = "0.25", default-features = false, features = ["widgets", "lucide-icons"] }` (workspace `Cargo.toml`); lock has snora 0.25.2.
- The application uses only: `snora::{AppLayout, LayoutDirection, render, Toast, ToastIntent, ToastLifetime, Icon, lucide}`, `snora::widget::icon_element_sized`, `snora::toast::{subscription, sweep_expired}`, `snora::lucide::LUCIDE_FONT_BYTES` (grep `crates/` to re-confirm). It does **not** use `snora::design`, `iced::advanced`, canvas, `Emphasis`/`Size`.
- snora 0.53.0 requires Rust 1.88; iced stays 0.14. The workspace enables iced's `tokio` and `image` features itself.
- Upstream states default-path **appearance** changed in 0.42 (toast colours), 0.50 (sidebar buttons), 0.51 (tab bar), 0.52 (label sizes follow text size), 0.53 (modal dim, breadcrumb underline, toast text on some themes). Composition (layer order, dismissal, mirroring) is unchanged without `design`.
- Behaviour change 0.41: overlays now contain pointer input (click on a dialog's own content no longer dismisses it; a click on a toast body no longer reaches the widget below). There is no flag to restore the old behaviour.
- The only committed screenshot is `docs/assets/screenshot-01.png` (1026 × 943, dark theme, captured on Windows with the v1.26.2 build, Microsoft logos enabled; its download buttons show wrong glyphs because the icon font was not registered then). No automated visual baselines exist.
- Three audit warnings (RUSTSEC-2026-0370, -0371, -0372: `zvariant`, `zbus_names`, `zvariant_derive`, unmaintained) are known and pre-triaged (RFC 024); they come from iced's `linux-theme-detection` feature and must not be "fixed" here.

## 3. Change scope (you may touch)

- Workspace `Cargo.toml`: the `snora` requirement only (version `0.53`; features unchanged).
- `Cargo.lock`: only the changes caused by `cargo update -p snora`.
- `crates/app/src/**`: only what the compiler forces (expected: nothing).
- `docs/assets/screenshot-01.png` and the README image reference if the file name or size changes; any document sentence that describes the old look **and is now false** (list them).
- Nothing else.

## 4. Non-change scope (you must not touch)

- Every other dependency (**no bare `cargo update`**, no other `-p`), `iced` features, `edition`, `resolver`, `rust-version`, lints, CI files, `.cargo/audit.toml`.
- The `design` feature of snora (**do not enable it**), `snora::keyboard::cycle_zones`, any new keyboard handling, any layout, style, i18n, or behaviour change. Out of scope on purpose: RFC 019 phase 2 and RFC 027 do those.
- `rfcs/` and `ROADMAP.md`.

## 5. Required implementation

1. **Baseline first (before touching anything).** On the base commit, run the gates (§7) and capture **before** screenshots (§6) with the *base* build.
2. Bump `snora` to `0.53` (same features) and run `cargo update -p snora`. Review `git diff Cargo.lock`: report every crate added, removed or changed, with versions; confirm **iced is still 0.14.x** and that no other direct dependency moved. If another direct dependency moved, stop and report.
3. `cargo build --workspace --all-targets --locked`; fix only what the compiler names. If a fix needs more than a few lines or a behaviour decision, stop and report with options.
4. Licences: list the licence of every new crate in the lock diff (`cargo metadata` or the crate manifests). The project is Apache-2.0; report anything that is not clearly compatible.
5. After the change, run the gates, capture **after** screenshots, run the manual checks (§6), retake the README screenshot (§6.3).

## 6. Required evidence (revised 2026-10-24 after the implementer's interim request: evidence proportional to risk, compared by machine, inspected by eye only where it differs)

**Principle.** Evidence has to answer a question; its quantity is not the measure. The questions for this job are: (Q-a) did anything visible change that we did not intend, (Q-b) is the upstream toast click-through fix really in effect, (Q-c) does the application still behave as before, (Q-d) is the README picture current. Prefer machine comparison (byte or pixel compare) and recorded state (file contents, command output) over pictures; look at a picture only when a comparison says it differs or when the picture is itself a deliverable.

All GUI work follows `.git-exclude/rules/project-instructions-gui-common.md` plus the automation rules in 6.6.

**6.1 Visual regression (Q-a).**
1. **Reuse the existing 12 "before" captures** (6 views × Light/Dark, taken on base commit `13ba2b0` and checked by the architect: all distinct, sizes 1536 × 864 and 840 × 864). Do not retake them.
2. After the upgrade, run **the same capture script once** with the new build to produce the matching 12 "after" files, same pinned config, same input (`packaging/windows/Assets/Square150x150Logo.png`).
3. Compare each before/after pair automatically (identical bytes, or a pixel-difference count). Report a 12-row table: view, theme, `identical` or `differs (N pixels, where)`.
4. Only for pairs that differ: look at them and describe the difference in words. Only then read the appearance guides that can explain it (`guides/migration-0.49-to-0.50.md`, `-0.50-to-0.51.md`, `-0.51-to-0.52.md`, `-0.52-to-0.53.md`); do not read them in advance. If a difference is unexplained by the guides, say so.
5. If the "after" set is **all identical**, say so and stop: that is the best outcome and needs no further pictures.

**6.2 Toasts (Q-b).** On the base build you already recorded that a click on an error toast's body opened the file chooser underneath ("YES") and that the `×` removes the toast; those base files (`toast-before-00..05`) are accepted as the baseline. Required on the new build, nothing more:
1. Error toast (fake `.png`): click its body: the chooser must **not** open (expected "NO"); then click `×`: it disappears. Record the same three captures as the baseline (`after-01-error-toast`, `after-02-after-body-click`, `after-03-after-x`).
2. Warning toast (invalid custom size `5` in Customize): one capture while visible (pair of `toast-before-05`). Auto-dismiss time: **read it from the source** (`snora-core` 0.25.2 `toast.rs:164`: `ToastLifetime::DEFAULT` = 4 s; the application sets 7 s explicitly for the long success toast, `crates/app/src/app.rs`), compare it with the 0.53 source, and confirm with **one** timed check: two captures, one right after the toast appears and one at about 5 s showing it gone. **Do not sample frames densely.**
3. Success and advisory toasts use the same widget and are not required.

**6.3 README screenshot (Q-d).** Replace `docs/assets/screenshot-01.png` with a fresh capture of the same scenario as the current one (Result screen, dark theme, input = `Square150x150Logo.png`, Microsoft app logos enabled in Customize, Preview collapsed), about 1026 × 943 if possible: **at most two attempts**. Say which platform and compositor you used (the old one was a Windows capture). Check that the download icons render and that no personal data (title, paths) is visible.

**6.4 Regression by use (Q-c), evidence by state, not by pictures.**
- Theme switch through the UI (Settings): the `theme` value in the isolated `settings.json` changes, and **one** capture shows the repaint.
- Language switch: English, Japanese, system: the `locale` value in `settings.json` after each; **one** capture of a Japanese screen (shows that text and icons render with the new snora).
- Persistence: restart the app with the same isolated config; read `settings.json` and note that the choices survived (no picture).
- Navigation Home/Customize/Settings and the bottom navigation at mobile width: already covered by the 12-pair matrix (the 700 px views); no extra pictures.
- Drag-hover feedback: **not required** (snora does not touch it; RFC 007 already tracks its platform limits).
- **ZIP with monochrome (closes the open J1 item):** enable monochrome in Customize through the UI, convert a PNG, press "Download all (ZIP)", save through the dialog; record `unzip -l` and `unzip -t` text output (expected: every artifact once, including `mono/favicon-32.png`, `mono/favicon-192.png`, `mono/favicon-512.png`, `mono/favicon.ico`; no errors). No picture needed.

**6.5 Dependency evidence:** `cargo tree -p logolig-app --depth 1` before and after (the "before" files exist); `cargo audit` before and after (before: 0 vulnerabilities, 12 warnings; report the delta; **no new vulnerability allowed**); lock diff and licences of added crates as in section 5.

**6.6 GUI automation rules (apply to every capture script).**
- Name the binary `logolig` (the config directory is named after the executable's file name); run **exactly one** instance; before starting, require zero running instances; kill by exact process name; pick the window by process id.
- A dialog is a window that appeared after your click; after each step assert that the screen changed as expected; **abort the run at the first anomaly** and report it instead of repeating the whole set.
- No frame-sampling loops. A measurement that needs more than a few captures must be replaced by reading the source or by a single timed check.
- If the same capture set has to be repeated a second time, stop and ask the architect before a third.
- The architect proposes to the owner that these rules also be added to `project-instructions-gui-common.md`.

## 7. Required validation (report exactly what was and was not run)

```
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
cargo audit
cargo +1.90 check --workspace --all-targets --all-features --locked
```

After the last push record the GitHub Actions run for the commit and the verdict of each of the five jobs. All five should be green once J1 is done; report honestly if any is red and why (do not edit CI).

## 8. Acceptance criteria

- `snora = 0.53.x` with unchanged features; iced 0.14.x; no other direct dependency changed; lock diff and licences reported.
- All gates green locally and in CI.
- The behaviour listed in §6.4 unchanged; toast behaviour as documented in §6.2 (with the click-through fix demonstrated or its non-demonstrability explained).
- Every visible difference described; the README screenshot replaced and verified.
- No change outside §3.

## 9. Prohibited shortcuts

- No enabling of `design`, no use of new snora APIs, no style or layout "fixes" to hide visual differences.
- No suppression of warnings or lints, no `#[allow]` to get green.
- No test weakening; no ignoring a failing gate.
- No version pin of any other crate to avoid a change.
- No tags, releases, force-pushes, history rewriting.

## 10. Security and compatibility constraints

- No new network access, I/O or input path is introduced; report any new transitive crate that links system libraries.
- Persisted settings are unchanged; the upgrade must not alter the config file.
- The minimum Rust version stays `1.90` (RFC 024); snora 0.53 needs 1.88.

## 11. Known risks

- The default look changes (toast colours, nav/side bar details, modal dim); the owner reviews the screenshots.
- A changed overlay behaviour may expose assumptions in toast handling; report, do not work around.
- The lock may pull a larger graph; report size and licences.
- If building against 0.53 fails in a way the guides do not explain, stop and report with the exact error; upstream offers help if sent `Cargo.toml` and the list of `snora::` and `iced::` imports.

## 12. Required review request

In `.git-exclude/review-request/rfc-028-snora-upgrade/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the commit range; the guides you read; the lock diff summary and licences; before/after screenshots with written differences; the toast evidence; the regression checklist results; gates, audit before/after and CI run verdicts; skipped checks with reasons; deviations; and open questions for the architect. Report to the owner only the path of the `README.md`.
