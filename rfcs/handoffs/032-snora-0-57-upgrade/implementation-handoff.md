# Developer Handoff — RFC 032: Upgrade snora from 0.53 to 0.57 (job J7d)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 032](../../accepted/032-snora-0-57-upgrade.md) (Accepted; owner authorized it into release 1.28.0 on 2026-11-09, with decision D1). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** You are the only job running. Work directly on `main`: `git pull --ff-only` first, small commits (short imperative subject), push after each commit that passes the local gates. Never rewrite history or force-push. Do not tag or release.

**Position in the schedule:** job J7d in `ROADMAP.md`, the first job of release 1.28.0. Nothing else is open.

## 1. Purpose

Move snora from 0.53 to 0.57.0 with the same features, replace the one API the new version removed, and add one small behaviour: a toast does not expire while the pointer rests on it. Everything else must look and behave as in 1.27.0.

## 2. Background (verified facts; the architect spiked this in a throwaway worktree)

- Pinned today: `snora = { version = "0.53", default-features = false, features = ["widgets", "lucide-icons"] }` in the workspace `Cargo.toml`. iced stays 0.14. snora 0.57's MSRV is 1.88 (ours is 1.90).
- With `0.57` the build has **exactly one error**: `snora::toast::sweep_expired` does not exist any more (`crates/app/src/app.rs`, the `Message::ToastTick` arm). It was deprecated in 0.54 and removed in 0.57. After the fix, clippy `-D warnings` is clean and 253 tests pass.
- The lock gains one package, `snora-interaction` (the renamed `snora-control`); the application never names it and must not (do not add it to any `Cargo.toml`).
- `cargo audit` on the spike tree showed 14 allowed warnings (12 on `main`). Unexplained; you explain it (§5.6).
- All our toasts set their lifetime explicitly (`persistent()` for errors, `with_lifetime(...)` otherwise, via `toast_time`), so snora's new default lifetime does not apply. `ToastLifetime::DEFAULT` is not used. **Do not touch `toast_time`.**
- The application sets `on_close_modals(Message::CloseModals)` and opens no modal or menu overlay; snora 0.55 changes only what happens while one is open. Our `Shortcut::EscapeOrBack` handling stays as it is.
- API in 0.57 (read it in `~/.cargo/registry/src/index.crates.io-*/snora-0.57.0/src/toast.rs` and `snora-core-0.57.0/src/{toast,layout}.rs`):
  - `snora::toast::Holds` (`Default`), `Holds::sweep(&mut self, toasts: &mut Vec<Toast<M>>, now: Instant)`, `Holds::attend(&mut self, id: u64, attention: ToastAttention, toasts: &mut [Toast<M>], now: Instant)`.
  - `AppLayout::on_toast_attention(f: fn(u64, ToastAttention) -> Message)`: **a plain function pointer**, so an enum variant that takes `(u64, ToastAttention)` can be passed directly.
  - `ToastAttention` is `#[non_exhaustive]` with `PointerEntered`, `PointerLeft`, `FocusGained`, `FocusLost`. `attend` handles all four and ignores an unknown one; your code passes the value through and must **not** `match` on it.
- Upstream's guides (read before you start): the notice `.git-exclude/upstream/snora/receive/0.57.0/snora-0.57.0-general-notice-to-app-teams-2026-11-08.md` and, online, `https://nabbisen.github.io/snora/guides/migration-0.53-to-0.54.html`, `…-0.54-to-0.55.html`, `…-0.55-to-0.56.html`, `…-0.56-to-0.57.html`. Do not adopt anything from them beyond §5.

## 3. Change scope (you may touch)

- Workspace `Cargo.toml`: the `snora` requirement only (`0.57`; features unchanged).
- `Cargo.lock`: only the changes caused by `cargo update -p snora`.
- `crates/app/src/app.rs` and `crates/app/src/shell.rs`: the toast sweep, the stored `Holds`, the new message and the one `.on_toast_attention(...)` call. If `AppState` is built in a place that also needs the new field (tests, `Default`), change only that.
- Tests for the above; `docs/src/architecture.md` (and any other document that names the snora version or the `sweep_expired` call) where the sentence is now false (list them).
- Nothing else.

## 4. Non-change scope (you must not touch)

- Every other dependency (**no bare `cargo update`**; the only `-p` allowed is `snora`, plus the optional line in §5.6 if and only if it is applied under that clause), `iced` features, `edition`, `resolver`, `rust-version`, lints, CI files, `.cargo/audit.toml`.
- No `snora::keyboard::*` (`traverse`, `Shortcuts`, `Escape`, `dismiss_on_escape`), no `AppLayout::shortcuts`/`escape`, no `design::*` control or feature, no `SideBarLabels`, no AAA preset, no `iced-default` change, no `on_toast_hover`/`ToastHover`/`Holds::hover` (deprecated), no use of `ToastLifetime::reading_time`.
- No change to any view, style, layout, sidebar, button, i18n text, or toast creation function.
- `rfcs/` and `ROADMAP.md`.

## 5. Required implementation

1. **Baseline first.** On the base commit run the gates (§7) and record `cargo tree -p logolig-app --depth 1` and `cargo audit` output into the evidence folder. Build the **base** release binary and keep it (`cp target/release/logolig` to your scratch directory) for the GUI comparison in §6.
2. Bump `snora` to `0.57` and run `cargo update -p snora`. Review `git diff Cargo.lock`: list every package added, removed or changed, with versions; confirm iced is still 0.14.x and no other direct dependency moved. If another direct dependency moved, stop and report.
3. Replace `snora::toast::sweep_expired(&mut state.toasts, Instant::now())` by `state.holds.sweep(&mut state.toasts, Instant::now())`. `holds: snora::toast::Holds` is a **field of `AppState`**, created with `Holds::default()` where the other fields are built. It lives as long as the state; a temporary `Holds::default()` per tick is wrong (it would forget every hold).
4. Add `Message::ToastAttention(u64, snora::ToastAttention)` (check the actual re-export path in 0.57; use the same path style as the other `snora::` imports), handled by `state.holds.attend(id, attention, &mut state.toasts, Instant::now())` returning `Task::none()`. In `shell::view` add `.on_toast_attention(Message::ToastAttention)` to the `AppLayout` chain.
5. A toast removed by `DismissToast` or the Esc handler leaves a stale id in `Holds`; `Holds::sweep` forgets ids whose toast is gone, so no extra code is needed. Do not add any. Verify by reading `sweep` and say so in the report.
6. **Advisories.** Compare `cargo audit` before and after package by package: which warning is new, which changed, which is pre-existing; for each new one name the dependency path (`cargo tree -i <crate>`). Zero vulnerabilities is required. If the 0.53 → 0.54 guide's `cargo update -p tokio -p wasmtimer -p async-executor -p clipboard-win` line removes warnings, **report the effect on the audit and ask in the review request**; apply it only if the architect says so (it is outside this job otherwise).
7. Documentation: update the sentences that name snora 0.53 (grep `docs/`, `README.md`, `packaging/`, `SECURITY*` if present). Do not add a version-table row (that happens at the release).

## 6. Required evidence (proportional to risk; follow `.git-exclude/rules/project-instructions-gui-common.md` and the automation rules in the RFC 028 handoff §6.6, which are in that file)

Questions: (Q-a) did anything visible change that we did not intend; (Q-b) does a toast wait under the pointer; (Q-c) did behaviour stay the same.

**6.1 Visual comparison (Q-a).** Reuse the capture script and inputs of the RFC 028 evidence if they still exist (`.git-exclude/review-request/rfc-028-snora-upgrade/`); otherwise write a smaller one: Home (empty), Result, Customize, Settings, each in Light and Dark at the launch window size, plus one view at about 700 px width (mobile bottom navigation). Capture with the **base** binary (saved in §5.1) and with the new binary, same pinned config, same input (`packaging/windows/Assets/Square150x150Logo.png`). Compare each pair by machine (identical bytes or a pixel-difference count with the bounding box of the difference) and report a table: view, theme, `identical` or `differs (N pixels, where)`. Look at a picture only where a pair differs and describe the difference in words. Snora's 0.55 sizing changes (chrome icons 14 → 16 px, toast close icon) may show on toasts; list them, do **not** patch styles. If everything is identical, say so.

**6.2 Toast attention (Q-b).** One error-free scenario with a success toast (e.g. convert the input; the success toast has a known lifetime computed by `toast_time`): (1) capture right after it appears; (2) move the pointer onto the toast (by a method the GUI rules allow on the tested backend) and wait **past its lifetime** (compute it from the text, then add 3 s): capture → the toast **is still there**; (3) move the pointer away: capture shortly after → still there (the countdown restarts from the full lifetime, per upstream), and a last capture after the lifetime → gone. Also one capture of an **error** (persistent) toast to confirm that it is unchanged and that `×` still dismisses it. If the pointer cannot be placed reliably on the tested backend (Wayland pointer injection is limited), say exactly what was tried and fall back to a unit test of the hold (below) and a documented gap; do not invent evidence. Do not sample frames densely.

**6.3 Regression by use (Q-c).** By state, not pictures: theme switch and language switch persist in the isolated `settings.json`; the keyboard shortcuts of RFC 019 phase 2 (Ctrl+O opens exactly one chooser; Ctrl+1/2/3; Esc dismisses the newest toast, then returns to the start screen) behave as before on the new build. These are exactly the paths snora 0.55's overlay/keyboard changes could touch; run them once each.

**6.4 Unit tests.** Keep the existing 253 tests and their names. Add: (a) `ToastTick` removes an expired toast through the stored `Holds` and keeps a non-expired one; (b) a toast with an attention `PointerEntered` is **not** removed at a time after its deadline, and is removed after `PointerLeft` plus its full lifetime, using the `Instant` arguments of `attend`/`sweep` (the handlers take `Instant::now()`; if that makes (b) untestable through `update`, test the `Holds` use in a small pure helper you introduce **only if** it is a few lines, or test it at the `Holds` level and state in the report that `update` is covered by the GUI check); (c) a `DismissToast` of a held toast does not break the next sweep. Tests live in the existing test modules/files per the rules, not inline.

## 7. Required validation (report exactly what was and was not run)

```
cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit
```

After the last push record the GitHub Actions run for the commit and the verdict of each of the five jobs (`fmt`, `clippy`, `test`, `msrv`, `audit`). The packaging workflow need not run (nothing under `packaging/` changes), but if the lock change alters the Linux build, note whether `cargo build --release --locked` still works.

## 8. Acceptance criteria

- `snora = 0.57.x`, features unchanged; iced 0.14.x; no other direct dependency changed; lock diff reported.
- `Holds` is owned by `AppState`; `on_toast_attention` is wired; no deprecated 0.54–0.57 API is used (`cargo clippy -D warnings` shows no deprecation).
- All gates green locally and in CI; test count 253 + the new tests, with the real number reported.
- The visual comparison and the toast-attention evidence are reported honestly (including any difference and any gap).
- The advisory delta is explained.
- No change outside §3.

## 9. Prohibited shortcuts

- No `#[allow]` to hide a deprecation or lint; no temporary `Holds` per call; no `match` on `ToastAttention`; no style or layout "fixes" to hide visual differences; no test weakening; no version pin of any other crate to avoid a change.
- No tags, releases, force-pushes, history rewriting.

## 10. Security and compatibility constraints

- No new network access, I/O or input path. Report any new transitive crate that links system libraries, and the licence of each added package (the project is Apache-2.0).
- Persisted settings unchanged: the upgrade must not alter the config file.
- MSRV stays 1.90.

## 11. Known risks

- Appearance drift from snora's 0.55 sizing changes: report, do not patch.
- A `Holds` that is created per tick instead of stored (see §5.3); a hold that never ends because the pointer-left event is lost when the toast is covered or the window loses focus: if you observe a toast that never expires, report it with the steps (that would be an upstream issue, not something to work around silently).
- If building against 0.57 fails in a way the guides do not explain, stop and report with the exact error.

## 12. Required review request

In `.git-exclude/review-request/rfc-032-snora-0-57-upgrade/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the commit range; the guides you read; the lock diff summary and licences; the visual table and the written differences; the toast-attention evidence; the regression results; gates, audit before/after and CI verdicts; skipped checks with reasons; deviations; and open questions for the architect. Report to the owner only the path of the `README.md`.
