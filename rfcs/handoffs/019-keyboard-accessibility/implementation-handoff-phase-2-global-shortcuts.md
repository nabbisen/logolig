# Developer Handoff — RFC 019, Phase 2: global keyboard shortcuts and the in-app list (job J4)

Role: you are the mid-capability implementation and testing agent. Follow `.git-exclude/roles/ai-multi-agent-software-development-organization-and-workflow.md`, `.git-exclude/roles/mid-capability-model-operating-instructions.md` and `.git-exclude/rules/` (Rust 2024, 2018+ module style without `mod.rs`, English only, tests separated from implementation, `unsafe` forbidden; relative paths with `cd`; GUI evidence rules in `.git-exclude/rules/project-instructions-gui-common.md`, "GUI Evidence and Automation Rules").

Governing RFC: [RFC 019](../../accepted/019-keyboard-accessibility.md), section "Phase 1 results" and "Phase 2 design (binding)" (added with this handoff). On any conflict, stop and send a clarification request.

Owner philosophy: clean, safe and secure, robust design; users must not be confused or misled. The owner kept the "accessible" tagline and asked for better keyboard UI/UX: this job is the first real step; the Tab/focus model is RFC 027 (not here).

**Workflow (owner decision): no topic branches, no concurrent jobs.** Work directly on `main`: `git pull --ff-only` first, small commits, **chain every gate with `&&` and push only when all pass**, no force-push, no tags. Job J4 in `ROADMAP.md`; prerequisite: J3b reviewed (it is).

## 1. Purpose

A keyboard-only user can do nothing today (RFC 019 phase 1, executed). Give the **main task** a complete keyboard path with a small, predictable set of global shortcuts, and tell users about them inside the application. The Customize page (checkboxes, slider, text fields) stays mouse-only until RFC 027; the documentation must say so.

## 2. Background (verified facts)

- Verified (iced 0.14.0 source): `iced::keyboard::Event::KeyPressed { key, modified_key, physical_key, location, modifiers, text, repeat }`; `Key::to_latin(physical_key)` returns the Latin letter or digit for a physical key (layout independent); `Modifiers::command()` is Ctrl on Linux/Windows and Cmd on macOS.
- Verified (iced 0.14): `iced::event::listen_with(f: fn(Event, event::Status, window::Id) -> Option<Message>)`. The application already uses it in `app.rs::subscription` for file hover/drop and window resize. `Status::Captured` means a widget used the event.
- Verified (iced_widget 0.14.2 `text_input.rs`): a **focused** text input captures typed text and its own editing combinations (Ctrl+C, Ctrl+X, Ctrl+V and similar); it does not capture Ctrl+O, Ctrl+S, Ctrl+digit or F1. Escape handling in the input is to be observed (see tests).
- The application has no key handler (phase 1 grep). `snora::keyboard::dismiss_on_escape` exists but is built for modals/menus (the application has none); do not use it. snora's F6 zone navigation is **not** part of this job (RFC 027 phase 0 evaluates it).
- Existing messages to reuse (no new behaviour): `PickFileRequested`, `DownloadAllRequested`, `EditCancelled`, `NavPageSelected(NavPage)`, `DismissToast(id)`.

## 3. The shortcuts (binding; RFC 019 "Phase 2 design")

Let **Cmd** mean `modifiers.command()` (Ctrl on Linux/Windows, Cmd on macOS). A shortcut fires only for a **key press that is not a key repeat** (`repeat == false`) and whose event status is **Ignored** (not captured by a widget).

| Keys | Action | Active only when |
|---|---|---|
| Cmd+O | `PickFileRequested` | page Home and screen Empty or Result (the drop zone, or Back then the drop zone, is available to the mouse) |
| Cmd+S | `DownloadAllRequested` | page Home, screen Result, result assets present (the "Download all (ZIP)" button is visible) |
| Esc | dismiss the **newest** toast (`DismissToast(id)`); if no toast: `EditCancelled` | the toast case: always; the back case: page Home and screen Result |
| Cmd+1 / Cmd+2 / Cmd+3 | `NavPageSelected(Home / Customize / Settings)` | always |
| F1 | `NavPageSelected(Settings)` (the shortcut list lives there) | always |

Rules: **a shortcut works exactly when the control it stands for is available to the mouse** (this also avoids the stale-result race of RFC 011: nothing starts a second ingest while converting). Letters and digits are matched layout-independently with `key.to_latin(physical_key)` (so Japanese and other non-Latin layouts work) and require **exactly** `command` and no Alt, no Shift (Cmd+Shift+O is not Cmd+O); Esc and F1 require **no** modifier. Typing a plain letter or digit never triggers anything. No other keys are bound; do not bind Ctrl+W, Alt+F4, Tab, arrows or F6.

## 4. Change scope (you may touch)

- New `crates/app/src/shortcuts.rs` (declare `mod shortcuts;` in `main.rs`) with: `pub(crate) enum Shortcut { ChooseFile, DownloadAll, EscapeOrBack, Page(NavPage), Help }`; `pub(crate) fn shortcut_for(key: &Key, physical: key::Physical, modifiers: Modifiers, repeat: bool) -> Option<Shortcut>` (pure; no state); `pub(crate) fn command_key_label() -> &'static str` (`"Ctrl"`, or `"Cmd"` under `cfg(target_os = "macos")`). Tests in `crates/app/src/shortcuts/tests.rs` (separate file, `#[cfg(test)] mod tests;`).
- `crates/app/src/app.rs`: the subscription gets a keyboard arm: `Event::Keyboard(KeyPressed{..})` with `status == Ignored` → `Message::ShortcutPressed(Shortcut)` (a plain `fn` pointer as `listen_with` requires; no captured state); a new `Message::ShortcutPressed(Shortcut)` and its handler `handle_shortcut(state, shortcut) -> Task<Message>` that applies the "active only when" column and **dispatches the existing message through `update`** (no duplicated logic). Tests in the existing inline `mod tests` (known violation, RFC 017: add there, do not restructure).
- The shortcut list on the **Settings page** (`crates/app/src/ui/settings_page.rs`): a section "Keyboard shortcuts" below Language and Theme: one row per shortcut (keys in a monospace-free plain text column, action text in the user's language), plus the honest note below. No new page, no new navigation item, no popup.
- i18n: new keys in `crates/core/src/message_key.rs`, `crates/i18n/src/dictionary.rs`, `locales/en.toml`, `ja.toml`, and `crates/i18n/tests/i18n.rs` key lists. Texts (architect-defined; do not reword; the ja texts have not had a native review, say so):
  - `ShortcutsSectionTitle`: en `Keyboard shortcuts` / ja `キーボードショートカット`
  - `ShortcutChooseFile`: en `Choose a file` / ja `ファイルを選ぶ`
  - `ShortcutDownloadAll`: en `Download all (ZIP)` / ja `すべてダウンロード（ZIP）`
  - `ShortcutEscape`: en `Close the message, or go back to the start` / ja `メッセージを閉じる、または最初の画面に戻る`
  - `ShortcutPages`: en `Go to Home, Customize or Settings` / ja `ホーム・カスタマイズ・設定を開く`
  - `ShortcutHelp`: en `Show this list` / ja `この一覧を表示`
  - `ShortcutsLimitNote`: en `Moving between buttons and fields with the Tab key is not available yet.` / ja `Tab キーでボタンや入力欄の間を移動する機能は、まだありません。`
  The key labels (`Ctrl+O`, `Ctrl+S`, `Esc`, `Ctrl+1 / 2 / 3`, `F1`; on macOS `Cmd` instead of `Ctrl`) are built from `command_key_label()` and are not translated.
- Source comments that wrongly claim keyboard accessibility (phase 1 found them): `crates/app/src/ui/drop_zone.rs` line 6 ("Keyboard-accessible alternative path…") and `crates/app/src/ui/preview_panel.rs` line 134 ("buttons as keyboard-accessible §12"): rewrite them to state what is true (the button is mouse-operated; the file chooser is also reachable with Cmd+O).
- Docs: `docs/src/ui-a11y.md` section "Keyboard support (current)": rewrite **from your evidence** (never before): what is supported (the shortcuts), what is not (Tab traversal, focus indicator, Customize page controls, etc.), the tested platforms and backends. `docs/src/functional-spec.md` §14 and the transition table row "ESC key" (it is implemented now), and §10 if needed.

## 5. Non-change scope (you must not touch)

Any widget behaviour, focus state, focus indicators, Tab handling, F6/snora zone helpers, `dismiss_on_escape`, toast logic other than dismissal, `Cargo.toml`/`Cargo.lock`, dependencies, other screens' layout, `rfcs/` and `ROADMAP.md`. No new setting, no remapping, no menu bar.

## 6. Required tests (design-derived; write them first)

`shortcuts/tests.rs` (pure): a table over every row of section 3 including: Cmd+O/S/1/2/3, Esc, F1 → the right `Shortcut`; plain `o`, `s`, `1`; Cmd+Shift+O; Cmd+Alt+O; Esc with a modifier; `repeat == true` for each; unbound keys (Tab, arrows, F6, Cmd+W) → `None`; **layout independence**: a logical key that is not Latin (for example the Japanese `ち`) with the physical key of `O` and Cmd → `ChooseFile`; `command_key_label` is `Ctrl` here.
`app.rs` inline tests (state): Cmd+O does nothing on Customize/Settings or while Converting, works on Empty and Result; Cmd+S works only on Result with assets; Esc dismisses the **newest** toast first (two toasts: one dismissed per press, no navigation), then goes back from Result, and does nothing on Empty/Customize/Settings/Converting; Cmd+1/2/3 and F1 set `nav_page`; Cmd+S on Empty does nothing; the handler returns the same `Task` count as the corresponding mouse message where observable (`Task::units()`).
i18n: the seven keys exist and are non-empty in `en` and `ja`; `ja` differs from `en`.
A guard that the two stale comments are gone is **not** required.

## 7. Required evidence (proportional; evidence by state, pictures only where listed)

1. Gates, chained: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit` (expected: 0 vulnerabilities, 12 warnings unchanged).
2. **GUI evidence on both backends.** Isolated config, binary named `logolig`, one instance, window 1280 × 720 focused first. **X11 backend** (`env -u WAYLAND_DISPLAY`, keys by `xdotool key --clearmodifiers`) **and native Wayland** (keys by `wtype`). Native Wayland now has a **positive control** because a shortcut has a visible effect (the page changes), which was missing in phase 1: record it. For each backend, assert by machine (window list or screenshot compare):
   - Cmd+2 → Customize page, Cmd+3 → Settings, Cmd+1 → Home (screenshot differs; one capture each is enough, compare by bytes);
   - F1 → Settings page with the shortcut list;
   - Cmd+O on Home Empty → a file dialog window appears (a new window, not a capture); choose a PNG with `wtype` as in earlier jobs;
   - after conversion, Cmd+S → the save dialog appears;
   - Esc with a toast visible (for example drop a non-square image) → the toast disappears and the screen stays; Esc again on Result → back to the start screen;
   - typing the plain letter `o` in the custom-size field of Customize (focus it by mouse on X11) enters `o` into the field and opens nothing; Cmd+O in that field: **record what happens** (no new window expected to be wrong; report the observed behaviour either way);
   - keys pressed while Converting do nothing harmful (Cmd+O and Cmd+S: no new window).
3. **Pictures (at most two):** the Settings page with the shortcut list in English and the same in Japanese. Everything else is evidence by state. Say which backend.
4. CI: the run of the final commit and each of the five job conclusions (if a job shows `in_progress` while the run is completed, read the raw job log).

## 8. Acceptance criteria

Every row of section 3 behaves as specified on X11 and native Wayland (evidence by state); the pure and state tests exist and pass, written first; the list is visible on the Settings page in both languages; the two stale comments and the docs are corrected from evidence; gates and CI green; no change outside section 4.

## 9. Prohibited shortcuts

No focus machinery, no new dependency, no use of `dismiss_on_escape` or F6, no binding beyond section 3, no shortcut that acts when its control is not available, no duplicated action logic (always dispatch the existing message), no documentation of an unverified platform, no test weakening, no red pushes, no tags or history rewriting.

## 10. Security and compatibility constraints

Key handling reads events only; no I/O, no new input path. A shortcut never bypasses a state check that the mouse path has. Persisted settings are unchanged.

## 11. Known risks

- `listen_with` takes a plain function pointer: the key mapping cannot capture state, hence the state check lives in `handle_shortcut`.
- Escape inside a focused text input is handled by the widget; observe and record, do not work around.
- A window manager or compositor may reserve a combination (for example Ctrl+Alt arrows); the chosen combinations are common application shortcuts, but report any that the compositor swallowed.
- The tab bar of a screen reader or IME may consume some keys; out of scope.

## 12. Required review request

`.git-exclude/review-request/rfc-019-phase-2-shortcuts/README.md` with everything required by section 7 of the mid-capability operating instructions, plus: the mapping function and handler code; a table of every shortcut × backend with the evidence (state), the recorded behaviour of Cmd+O inside a text field; the documentation changes with before/after text; gates and CI verdicts; skipped checks; deviations; open questions. Report to the owner only the path of the `README.md`.
