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

## 6. Required evidence

All GUI work follows `.git-exclude/rules/project-instructions-gui-common.md`: run with an **isolated config** (`XDG_CONFIG_HOME` pointing at a scratch directory; never touch the maintainer's real settings), resize to 1280 × 720 and focus the window before typing, capture with `niri msg action screenshot-window --id <id> --path <relative path>` (not ImageMagick `import`).

1. **Before/after screenshots**, same input and window size, for: Home empty; Home Result (use `packaging/windows/Assets/Square150x150Logo.png` as the input file, default plan); Customize (Advanced expanded); Settings; each in **Light and Dark** theme; plus the mobile layout at about 700 px width for Home Result. Put them side by side in the review request and describe each visible difference in words (colours, sizes, spacing, anything that moved).
2. **Toast behaviour** (the 0.41 fix and the toast changes): trigger toasts of each kind you can reach (warning: type an invalid custom size such as `5` in Customize and press Add; success: save a file; the non-square or JPEG advisory by dropping a suitable file; error: drop an unsupported file), and record for each: appearance, auto-dismiss time, the dismiss `×`, and that a **click on a toast body does not activate a widget beneath it** (arrange the page so that a control lies under the toast, click the toast body, confirm nothing underneath reacts; also run the same check on the base build to show the difference, or say that it cannot be arranged).
3. **README screenshot.** Replace `docs/assets/screenshot-01.png` with a fresh capture of the same scenario as the current one (Result screen, dark theme, input = `Square150x150Logo.png`, Microsoft app logos enabled in Customize, the Preview section collapsed), window about 1026 × 943 if possible. Say which platform and compositor you used; the old one was a Windows capture, a Linux one is acceptable. Check that the download icons render correctly and that no personal data is visible (window title, paths). Update the README sentence only if it describes something that changed.
4. **Regression check by use:** language switch (English, Japanese, system), theme switch, settings persistence across a restart (isolated config), navigation Home/Customize/Settings, the bottom navigation at mobile width, drag-hover feedback (`FileHovered`) if you can produce it, the ZIP download with monochrome on. **This check also closes an open item of job J1 (the `zip` 9 upgrade):** enable the monochrome option in Customize through the UI, convert a PNG, press "Download all (ZIP)", save through the dialog, and record `unzip -l` and `unzip -t` of the saved archive (expected: every artifact once, including `mono/favicon-32.png`, `mono/favicon-192.png`, `mono/favicon-512.png`, `mono/favicon.ico`, and no errors).
5. **Dependency evidence:** `cargo tree -p logolig-app --depth 1` before and after; `cargo audit` before and after (expected: same advisories, snora-related warning set possibly changed; report the delta; **no new vulnerability allowed**).

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
