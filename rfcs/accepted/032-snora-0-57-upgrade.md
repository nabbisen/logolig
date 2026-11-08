# RFC 032 — Upgrade snora from 0.53 to 0.57

**Status.** Accepted (owner authorization 2026-11-09, release 1.28.0)
**Priority.** P1 (a build break is waiting on the next `cargo update`; the keyboard work of 1.28.0 builds on the result)
**Tracks.** snora's general notice to application teams, 0.57.0 (received 2026-11-08) and its migration guides 0.53 → 0.54 → 0.55 → 0.56 → 0.57.
**Touches.** `Cargo.toml`, `Cargo.lock`, `crates/app/src/app.rs` (toast sweep, toast subscription, optionally toast attention), docs. **Not** the views, the sidebar, or the keyboard handling (see "Out of scope").

## Summary

snora 0.57.0 is out and the application pins 0.53. The upgrade is small in code and large in what it enables, so it is done **alone, first, and without adopting any new feature**, which keeps the review about one question: "does the application behave exactly as before on the new engine?" What the new engine offers the keyboard work (RFC 027) is decided there, after a spike, not here.

## Evidence (architect, 2026-11-09)

- **Spike in a throwaway worktree (nothing committed):** `snora = "0.57"` with the existing features, `cargo update -p snora`. Exactly **one** compile error: `snora::toast::sweep_expired` no longer exists (`crates/app/src/app.rs`, the `ToastTick` handler). It was deprecated in 0.54 and removed in 0.57. After replacing it, `cargo clippy --workspace --all-targets --all-features -- -D warnings` is clean and `cargo test --workspace --all-features` passes **253 of 253**. The lock gains one package, `snora-interaction` (the former `snora-control`, a dependency of `snora`; the application never names it).
- `cargo audit` on the spike tree reports **14 allowed warnings** (12 today). The difference is not yet explained; it is an acceptance criterion below.
- Our toasts all set their lifetime explicitly (`persistent()` for errors, `with_lifetime(...)` otherwise), so snora's new default lifetime (`Toast::new`: errors persistent, others reading time) changes nothing for us. `ToastLifetime::DEFAULT` is not used.
- The application sets `on_close_modals(Message::CloseModals)` but opens no modal or menu overlay. snora 0.55 makes Escape close such overlays and stop keys reaching the page beneath them; with no overlay open, nothing changes. Our own Esc handling (`Shortcut::EscapeOrBack`) is unaffected.
- The application builds its navigation, buttons and cards from **plain `iced::widget::button`** with its own styles. snora's keyboard reach (0.55) applies to **snora's own controls** only, and the migration guide says plain iced buttons "aren't affected". Moving to 0.57 therefore gives our screens **no** keyboard reach by itself; that is RFC 027's question.
- snora is still on iced 0.14; no iced change. snora's MSRV 1.88 is below ours (1.90).

## Requirements

1. `snora` is pinned to `0.57` (same features: `widgets`, `lucide-icons`, `default-features = false`). The lock gains `snora-interaction` and updates `snora`, `snora-core`, `snora-widgets` and nothing else except what `cargo` must move; the diff of `Cargo.lock` is listed in the review request.
2. `sweep_expired` is replaced by `Holds::sweep`. `Holds` is **state**, not a temporary: `AppState` owns one `snora::toast::Holds` for the application's lifetime. (The spike used a throwaway `Holds::default()` only to prove the build; it would forget holds between ticks and must not be copied.)
3. **Toast attention (decision D1 below):** `on_toast_attention` is wired to `Holds::attend`, so a toast does not expire while the pointer rests on it. The 0.54 `on_toast_hover` family is **not** used (deprecated, removed in 0.59).
4. No other behaviour changes. In particular: no `snora::keyboard::traverse`, no `AppLayout::shortcuts`, no `AppLayout::escape`, no `design::*` control, no `SideBarLabels`, no theme preset, no `iced-default` change.
5. A GUI check (rules: `.git-exclude/rules/project-instructions-gui-common.md`) on the 0.57 build compares with the 1.27.0 build for the three nav pages and the Result screen: sizes, toast placement, the toast close icon (snora 0.55 made it smaller on the design path; we use the default path, to be confirmed), the sidebar footprint. Every difference is listed, even one the dev team judges harmless.
6. The advisory delta (12 → 14 allowed warnings) is explained package by package in the review request: new, changed, or pre-existing. If a new advisory comes from the upgrade, say so; the 0.53 → 0.54 guide §5 gives a `cargo update -p ...` line for the ones snora's own lock cleared, and it is applied only if it removes warnings.
7. Documentation: `architecture.md`'s dependency line and the packaging/security notes that name the snora version are updated; the version table gets no new row until the release that ships this (1.28.0).

## Decisions

- **D1 (architect, recommended; owner may overrule): adopt toast attention.** A toast that disappears while the pointer is on it is a reading failure, the very class RFC 029 fixed by lifetime. With `Holds` required anyway, `on_toast_attention` is three more lines and no new concept. The keyboard half (`FocusGained`/`FocusLost`) is delivered by snora and inert for us until RFC 027 makes a toast's × reachable. If the owner prefers "nothing new in this RFC", drop requirement 3; requirement 2 stays.
- **D2: our own reading-time function stays.** snora 0.54 added `ToastLifetime::reading_time` (2 s + 1 s per 6 characters, at least 6 s). RFC 029's function (`toast_time`) is tuned to this application's texts and is reviewed and tested; replacing it would change behaviour for no gain. Revisit only if snora's function is documented as a stable contract and ours becomes a duplicate worth deleting.
- **D3: the dependency is the owner's own library**, and `snora-interaction` is published by the same project. The security review note for the release records the new package and its origin; no other new transitive package is expected.

## Out of scope (decided elsewhere)

- Making the application keyboard-operable with snora's focus engine (`traverse`, snora's sidebar, `design::button`, shortcuts `Ctrl+1..9`, `Escape` routing): **RFC 027**, phase 0 spike, now starting from this baseline. See its update of 2026-11-09.
- The sidebar naming its items (`SideBarLabels`) and the AAA presets: not used by our own sidebar; relevant only if RFC 027 chooses snora's sidebar.
- Skipping 0.55.0 is moot (we land on 0.57.0).

## Test plan

- The existing 253 tests stay green and **keep their names**. Add: a test that `ToastTick` removes an expired toast through the stored `Holds` and keeps a non-expired one; a test that a hold (attention entered) keeps a toast past its deadline and that leaving restarts the countdown, if `Holds` is testable without a GUI (otherwise say so in the review request and cover it by the GUI check).
- Gates chained: `fmt`, `clippy -D warnings`, `test`, `cargo +1.90 check`, `cargo audit`.
- GUI evidence per requirement 5, with the toast-hold behaviour shown by two captures (pointer on the toast past its deadline; pointer moved away, toast gone).

## Acceptance criteria

The application builds and passes all gates on snora 0.57; behaviour is unchanged except that a toast waits while the pointer is on it (if D1 stands); the advisory delta is explained; the GUI comparison lists every difference; and a short note in the review request says which of snora's new capabilities RFC 027 can build on (facts seen while working, not design).

## Risks

- Appearance drift from snora's sizing changes (0.55): contained by the GUI comparison. Mitigation if something shifts on the default path: report it; do not patch styles in this job.
- An advisory introduced by a new transitive package: contained by requirement 6; the owner decides acceptance.
