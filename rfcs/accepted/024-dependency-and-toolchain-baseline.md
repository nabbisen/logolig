# RFC 024 — Dependency and toolchain baseline (MSRV, `cargo audit`)

**Status.** Accepted (owner, 2026-10-21: "Yes" to D1 and D2 as recommended)
**Priority.** P1 (CI is red on two jobs; the declared MSRV is false)
**Handoffs.** [rfcs/handoffs/024-dependency-and-toolchain-baseline/](../handoffs/024-dependency-and-toolchain-baseline/README.md)
**Tracks.** Findings of the RFC 020 phase-1 CI work.
**Touches.** `Cargo.toml` (`rust-version`, possibly `zip`), `Cargo.lock`, `README.md`, `.cargo/audit.toml`, `.github/workflows/ci.yml`, ZIP code in `crates/app`.

## Summary

Two honest red CI jobs need decisions: the declared minimum Rust version (1.85) has never been true, and `cargo audit` reports three vulnerabilities in `zip` 2.4.2.

## Evidence (verified by execution in CI and locally, 2026-10-21)

- `cargo check --workspace --locked` on rustc 1.85.1 fails before compiling: `app-json-settings 2.0.3` needs 1.90; `iced 0.14.0`, `image 0.25.10`, `wgpu 27.0.1` need 1.88; several others 1.87. `cargo +1.90 check` passes, `+1.89` fails. **Effective MSRV of the locked tree: 1.90.** README says "Requires Rust 1.85 or newer".
- `cargo audit` (cargo-audit 0.22.2, fresh advisory DB, 583 crates): **3 vulnerabilities**, all `zip` 2.4.2: RUSTSEC-2026-0367 (SUID/SGID bits preserved on extraction), -0368 (`extract` creates symlinks pointing outside the destination), -0369 (permissions applied through symlinks without confinement); fixed in `zip` >= 9.0.0. Plus 14 allowed warnings (unmaintained/unsound notices), not triaged.
- Reachability (developer's reading, confirmed by the architect's code review): all three concern **extraction**. Production code only **writes** archives (`ZipWriter`); no call to `ZipArchive::extract` exists; tests only read entries. Not reachable today.
- `zip` 9.0.5 declares `rust-version = "1.88"` (registry metadata), within the effective 1.90.
- Two earlier audit ignores exist for quick-xml (build-time, via `wayland-scanner`) with a stated removal condition.

## Decisions (architect recommendation; the owner decides compatibility and risk acceptance)

- **D1 MSRV: declare the truth.** Set `rust-version = "1.90"` and correct the README. Reason: a false compatibility statement misleads users and packagers; the real floor is dictated by a direct dependency, and every distribution channel under consideration (Arch, Flathub runtimes) ships a newer compiler. Alternative (hold dependencies back to reach 1.85) means pinning old versions of `iced`, `image`, `wgpu`, which is neither clean nor secure. The `msrv` CI job then verifies the new floor.
- **D2 `zip`: upgrade to >= 9 rather than ignore.** The vulnerabilities are unreachable, so this is not an emergency, but a clean audit is the standard the owner asked for, and the ZIP code was just reworked (RFC 009). A small handoff: upgrade, adapt API changes, keep the ZIP tests. If the upgrade proves larger than expected, the interim fallback is a documented ignore in `.cargo/audit.toml` with the unreachability rationale and a removal condition, **approved by the owner as an explicit risk acceptance**.
- **D3 Audit warnings:** triage the 14 warnings once (reachable or build-time, upgrade possible, upstream status) and record the result in this RFC or a short note; no blanket ignores. **Pre-triage (snora's 0.53 guide, 2026-10-21):** three of the 14, RUSTSEC-2026-0370 (`zvariant`), -0371 (`zbus_names`), -0372 (`zvariant_derive`), are *unmaintained* notices, not vulnerabilities; they enter the graph on Linux through iced's default `linux-theme-detection` feature (D-Bus colour-scheme detection) and cannot be cleared while iced uses `zbus` 5. Recommendation: leave them as warnings, and keep that iced feature because RFC 013 depends on it. The implementer still verifies this with `cargo tree -i`.
- **D4 CI:** keep `ubuntu-latest` unless a runner change breaks a run; revisit then (pinning `ubuntu-24.04` is a one-line change). The `actions/cache` restore/save split stays.
- **D5 Required checks:** after D1 and D2 land and the jobs are green, the owner marks all five jobs (`fmt`, `clippy`, `test`, `msrv`, `audit`) as required in branch protection. Not before; a required red job blocks every merge.

## Acceptance criteria

`msrv` and `audit` green in CI on `main`; README states the real minimum Rust version; no ignore added without a written rationale and an owner decision; `cargo test --workspace --all-features` unchanged apart from any ZIP test adaptations.

## Risk

Low. Raising the declared MSRV may affect anyone building with Rust 1.85–1.89, who could not build the locked tree anyway.

## Review outcome and further decisions (architect, 2026-10-21)

Job J1 was implemented and reviewed (`.git-exclude/reviewed/rfc-024-dependency-baseline-review-2026-10-21.md`): `rust-version = "1.90"`, README corrected, `zip` 9.0.5, all five CI jobs green, `cargo audit` with 0 vulnerabilities. Decisions on the questions raised by the review:

- **`zip` feature set: accepted.** `deflate-flate2-zlib-rs` is the old `deflate` minus the optional zopfli compressor (verified in the `zip` 9.0.5 manifest: `deflate` = `deflate-zopfli` + `deflate-flate2-zlib-rs`). The explicit pure-Rust `zlib-rs` backend is preferable to relying on another crate to enable a flate2 backend (robust, no C dependency).
- **Audit triage, pre-decided rows:** leave RUSTSEC-2026-0370/-0371/-0372 (zvariant family, Linux D-Bus theme detection; RFC 013 needs it), ttf-parser, paste, and lru (pinned by iced's `cryoglyph`; clears only with an iced upgrade). Leave rustybuzz until RFC 015 phase 2b, which makes it reachable.
- **Follow-up job J1b (small): three single-crate lock updates** that clear three more warnings because patched releases exist: `anyhow` (>= 1.0.103), `event-listener` (>= 5.4.2), `memmap2` (>= 0.9.11). Handoff: `rfcs/handoffs/024-dependency-and-toolchain-baseline/implementation-handoff-followup-lock-updates.md`.
- **`vtracer` hygiene (rows 1–4: adler, ansi_term, atty x2):** four warnings come from `vtracer` 0.6.5's old `image` 0.23 / `clap` 2 chain. Not urgent (not on an executed path as far as known), but unclean. Scheduled as an evidence-only spike in release 1.29.0 (ROADMAP J16): can the chain be avoided through features, a newer vtracer, or a replacement, and what would a replacement cost in output quality?
- **D5 revised (required checks).** All five jobs are green, but **marking them "required" does not help the owner's workflow**: required status checks gate pull-request merges, while the agreed workflow is direct pushes to `main` (no branches). A required check cannot stop a direct push that is red after it lands. The protection that works is on the implementer's side: run all gates locally with a command chain that stops on the first failure (`&&`), and push only when green, which every handoff requires and the J1 report shows was violated once (the red `ec9e1ed`, repaired within minutes). The architect watches CI after every push. The owner may still enable "required" later if the project moves to pull requests.
