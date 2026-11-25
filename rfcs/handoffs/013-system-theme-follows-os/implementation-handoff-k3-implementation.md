# Developer Handoff — RFC 013 job K3: "System" follows the operating system

Role and rules as always (`.git-exclude/roles/*`, `.git-exclude/rules/*`; GUI Evidence and Test Optimization rules; Rust 2024, tests in separate files, English only, `unsafe` forbidden; **relative paths with `cd`, or none**; `bash -n` only for syntax checks). Governing text: [RFC 013](../../done/013-system-theme-follows-os.md), **"Result of the spike and the design it fixes" (items 1–6 are binding)**. Spike: `.git-exclude/review-request/rfc-013-system-theme-spike/` (its `settings-stub.py`, `private-bus.sh`, `k3-run.sh` are reusable as they are). **Workflow:** `main`; `git pull --ff-only`; `git log origin/main..` before every push; one commit per step; gates chained (audit 0 / 12); CI waited for each (and **look at the run after your push**); no tags, force-push or history rewriting.

## 0. Evidence plan

| Question | Level | Evidence |
|---|---|---|
| `resolve_theme` for every `Mode {None, Light, Dark} × ThemeMode {System, Light, Dark}` | pure | 9 cases; `System` follows the mode (`None` → Light), `Light`/`Dark` ignore it |
| Message handling: boot task and events, last wins; a leading `None` then `Dark` ends Dark; `Dark` then `None` ends Light; the persisted settings do not change | `update()` tests | hand-built messages; the persisted-leaf test pattern of J11 |
| Settings text in the three states, English and Japanese, both themes, all window sizes, no overflow | headless matrix + i18n | key parity; the existing layout matrix extended with the three states |
| A theme change repaints (System + Dark mode renders dark, + Light renders light), in the same session driver | headless | pixel mean against the dark/light thresholds |
| The preview's "System" surface follows the mode | logic/headless | the four mock-up colour functions take the resolved mode |
| The real application follows a private-bus stub (start dark, live change, no service) | **real UI, one run, my word** | below |

## 1. Work (tests first, shown failing before)

1. `AppState.system_theme: iced::theme::Mode` (default `None`); a message `SystemThemeMode(Mode)`; at boot `iced::system::theme()` as a task and `iced::system::theme_changes()` as a subscription, both mapped to that message; the handler stores it (no `persist_settings`, no leaf changes). `resolve_theme` becomes the one function of the table above. **Do not add a "re-ask after the window opens"** (RFC 013 decision 2).
2. The Settings page text per decision 3: keys for the three states (a template with the resolved theme name reusing `ThemeNameLight`/`ThemeNameDark`, or three keys: choose, say which; new `MessageKey`s are the accepted source-compatibility note); English and Japanese; the Japanese reads naturally next to the neighbouring strings (list the pairs for the owner's review list `.git-exclude/tmp/ja-review-K3.md`, not blocking).
3. The preview: the mock-ups and the picker label receive the resolved mode instead of matching `ThemeMode::System` themselves (decision 4; `ui/preview_panel/mockups.rs` four functions, `pickers.rs` label).
4. Docs: `docs/src/functional-spec.md` (the theme setting: System follows the operating system where it reports a theme; otherwise Light; the text on the Settings page), the README sentence on themes if it states the old behaviour, `ui-a11y.md` only if it speaks of themes. State plainly that Windows and macOS were not exercised.
5. `ROADMAP.md`'s release checklist line for the stub check is the architect's.

## 2. The one real run (my word for exactly this; native Wayland; X11 only if cheap)

`scripts/gui-verify-test.sh` first (ALL REPLAY CASES BEHAVE). The spike's private bus and stub (copied from its evidence folder), pinned `settings.json` with theme System and locale `en`, `scripts/gui-session.sh`, unlocked session, one instance by pid, a second-signal check by the page's luminance (analysed afterwards), the owner's real `settings.json` mtime before and after, the owner's real session and settings never touched (the stub runs on the private bus only). Scenarios, each a second signal: stub says dark → the page is dark and Settings reads "System (currently Dark)"; the stub emits light → the page repaints light and the text changes; back to dark; **no service** → light and "System (not reported; using Light)". One run; a repeat needs my words. Delete the binary copy, the run directory, the bus and the stub afterwards.

## 3. Evidence, acceptance, prohibited

Review request `.git-exclude/review-request/rfc-013-system-theme/README.md` (report only its path): failing-before lists; the test-list diff (named additions, none removed); the real-run log and analysis; the Japanese pairs; the SemVer additions; gates and CI per commit. Acceptance: `System` follows the OS where it reports one, Light otherwise, honestly labelled; a live change repaints; the preview follows; the setting file is unchanged; no `unsafe`; gates and CI green. Prohibited: a direct D-Bus read in the application or a new dependency edge; a "re-ask" hack; writing the owner's desktop settings (the stub is private-bus only); sampling loops; absolute `cd`; tags, force-push or history rewriting.
