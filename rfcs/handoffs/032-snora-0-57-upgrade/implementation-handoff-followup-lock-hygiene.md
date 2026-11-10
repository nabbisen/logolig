# Developer Handoff — follow-up of RFC 032: three single-crate lock updates and one documentation sentence (job J7e)

Same roles, rules and workflow as [`implementation-handoff.md`](./implementation-handoff.md) (directly on `main`, `git pull --ff-only` first, gates chained with `&&`, no red push, no tags, relative `cd`). Review of J7d: `.git-exclude/reviewed/rfc-032-snora-0-57-upgrade-review-2026-11-10.md`. Precedent: J1b and J2a (hygiene inside the owner-authorized "clean audit" objective of the release).

## Why

Today's tree has 14 allowed audit warnings; two are new since the 1.27.0 candidate and none comes from snora: `pin-utils` 0.1.0 unmaintained (RUSTSEC-2026-0441, via `wasmtimer`, a wasm-only dependency present in the lock) and `async-executor` 1.14.0 yanked (via `zbus` ← `ashpd` ← `rfd`, and `zbus` ← `mundy` ← `iced_winit`). `clipboard-win` 5.4.1 is yanked too. Your experiment in the J7d request showed that `cargo update -p async-executor -p clipboard-win -p wasmtimer` gives 14 → 11 warnings with 0 vulnerabilities. A shipped release should not carry yanked crates when the fix is a patch update.

## Scope

1. `cargo update -p async-executor -p clipboard-win -p wasmtimer` (**no `-p tokio`**, it is current; no bare `cargo update`). Expected lock diff: `async-executor` 1.14.0 → 1.14.1, `clipboard-win` 5.4.1 → 5.4.3, `wasmtimer` 0.4.3 → 0.4.5, and the removal of `pin-utils` if nothing else needs it. **If the command moves any other package, stop and report** (do not commit). Check each updated crate's licence is unchanged.
2. `docs/src/functional-spec.md` §10 (toasts): add the sentence that a toast does not expire while the pointer rests on it and that its countdown restarts in full when the pointer leaves; errors stay until dismissed (state only what RFC 032 shipped; do not mention keyboard focus, which is not reachable yet). One or two sentences, same style as the section.
3. Nothing else: no `Cargo.toml` edit, no code change.

## Evidence required

- `cargo audit` before and after, block by block: expected 14 → 11 warnings, 0 vulnerabilities; say which three went and explain any other difference.
- Gates chained on the final tree: `cargo fmt --check && cargo clippy --workspace --all-targets --all-features --locked -- -D warnings && cargo test --workspace --all-features --locked && cargo +1.90 check --workspace --all-targets --all-features --locked && cargo audit` (261 tests expected).
- `cargo build --release --locked` works.
- **One GUI smoke check** (rules: `.git-exclude/rules/project-instructions-gui-common.md`; isolated `XDG_CONFIG_HOME` and `XDG_STATE_HOME`; never start the binary without them): on native Wayland, release build from the final tree, Ctrl+O opens **exactly one** chooser window (it uses the portal through `zbus`, which `async-executor` serves), a PNG is chosen, the Result screen appears. Same criteria as the J7d regression run (Download colour on the Result screen). One run; stop at the first anomaly.
- CI on the pushed commit: the five jobs by name.
- Review request: `.git-exclude/review-request/rfc-032-lock-hygiene/README.md`; report only its path.

## Prohibited

No `-p` other than the three named, no `[patch]`, no version pin, no `cargo update` without `-p`, no `rfcs/` or `ROADMAP.md` edits, no new feature.
