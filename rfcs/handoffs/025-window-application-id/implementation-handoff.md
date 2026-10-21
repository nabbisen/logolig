# Developer Handoff — RFC 025: Window application ID

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`).

Governing RFC: [RFC 025](../../accepted/025-window-application-id.md) (Accepted). The RFC defines what and why; this handoff defines how and how to prove it. On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled.

**Workflow (owner decision 2026-10-21): no topic branches and no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first, small commits (short imperative subject), push after each commit that passes the local gates. Never rewrite history or force-push. Do not tag or release. This is job J2 in `ROADMAP.md`; start only after J1 (RFC 024) is reviewed.

## 1. Purpose

The application reports an **empty** window application id (Wayland `app_id`). Desktop environments therefore cannot match the window to a desktop entry. Set the id to `io.github.nabbisen.Logolig` so installed packages (AUR now, Flatpak later) get their icon and grouping.

## 2. Background (verified facts)

- Verified by execution (2026-10-21, niri, debug build, isolated config): `niri msg --json windows` shows `app_id` as the empty string for the Logolig window.
- `crates/app/src/app.rs::run` builds `iced::application(AppState::boot, update, view)` with `.font(..)`, `.title(..)`, `.theme(..)`, `.subscription(..)`; no window settings.
- Not verified: the exact iced 0.14 API. Expected on Linux: `iced::window::Settings { platform_specific: iced::window::settings::PlatformSpecific { application_id: ..., ..Default::default() }, ..Default::default() }` passed through `.window(..)`. **Read the dependency source** (`iced`, `iced_core`, `iced_winit` 0.14 in `~/.cargo/registry/src/*/`) to confirm the field names and how they map to `WM_CLASS` under X11. The platform-specific struct is different on each OS, so the code must be guarded for Linux.
- The window title is "Logolig" (from `MessageKey::AppTitle`); do not change it.
- Owner decision (RFC 025 D1): the id is `io.github.nabbisen.Logolig`. Only Flatpak *requires* a reverse-DNS name; on every distribution the window id must equal the desktop file's base name, which is why it is set here.

## 3. Change scope (you may touch)

- `crates/app/src/app.rs` (`run`) and, if cleaner, one small new private function that returns the window settings so it can be unit-tested; a new module file is acceptable only if the project rules for module layout are followed. Keep the id in **one** public-to-the-crate constant (for example `pub(crate) const APP_ID: &str`).
- Tests for the constant and the settings function; the test must live in a separate test file according to the project rule (do not add to the inline `mod tests` of `app.rs` unless it is the only practical place; if so, say so in the review request).
- `docs/src/architecture.md` is **not** to be edited here (RFC 018 owns documentation drift). Add one sentence to `docs/src/ui-a11y.md` or none: do not edit docs in this job unless a statement becomes false.

## 4. Non-change scope (you must not touch)

- Window size, title, icon, theme, subscriptions, any message or state handling.
- Packaging files (`packaging/`): the desktop file belongs to RFC 020 phase 2.
- Windows and macOS behaviour: the setting must be compiled only on Linux (`#[cfg(target_os = "linux")]`); other platforms keep their current window settings exactly. You cannot build for Windows or macOS here; say so, and write the code so that it cannot affect them.
- No new dependency; no `unsafe`.

## 5. Required behaviour

1. On Linux the running window reports the application id `io.github.nabbisen.Logolig` (Wayland `app_id`; X11 `WM_CLASS` class derived by winit; record the actual value).
2. Nothing else about the window changes.
3. Non-Linux targets compile unchanged (cfg-guarded).

## 6. Required tests and evidence

- A unit test asserting the constant equals `io.github.nabbisen.Logolig` and, if the settings are produced by a function, that the Linux settings carry it.
- **Manual, mandatory (GUI rules in `.git-exclude/rules/project-instructions-gui-common.md`; isolated config via `XDG_CONFIG_HOME` pointing at a scratch directory, never the maintainer's real settings):** launch the built binary; record `niri msg --json windows` `app_id` before (from the base commit; expect empty) and after (expect the id). Under the X11 backend (`env -u WAYLAND_DISPLAY`), record `xprop WM_CLASS` (or `xdotool getwindowclassname <id>`) and state the instance and class values; if they differ from the id, report that for the AUR/Flatpak desktop-file decision (`StartupWMClass`).
- Gates, reported exactly: `cargo fmt --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-features --locked`.
- CI: after the last push, record the GitHub Actions run for the commit and the verdict of each of the five jobs. Report honestly if any is red and why.

## 7. Acceptance criteria

The id is reported on Wayland and X11 as recorded; the diff touches only the files in §3; gates green; non-Linux code paths untouched (show the `cfg` guard).

## 8. Prohibited shortcuts

- No guessing of the iced API; read the source and quote the lines you relied on.
- No setting of the id on non-Linux targets; no change to the window title to "fix" matching.
- No `StartupWMClass` or desktop file edits here; report findings.

## 9. Security and compatibility

None beyond the above: one constant and one window setting; no input, no I/O.

## 10. Known risks

- The iced API may differ from the expected shape; if setting the id requires more than a few lines or touches runtime code, stop and report with options.
- Under X11 the class may be derived differently from the Wayland id; that is a finding, not a defect to fix here.

## 11. Required review request

In `.git-exclude/review-request/rfc-025-window-app-id/` with a `README.md` entry point containing everything required by section 7 of the mid-capability operating instructions, plus: the commit range; the quoted dependency source lines; before/after `app_id` and `WM_CLASS` evidence; gates and CI run verdicts; skipped checks with reasons (Windows/macOS not built); and open questions for the architect. Report to the owner only the path of the `README.md`.
