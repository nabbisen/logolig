# Logolig roadmap and release plan

**Status.** **Authorized by the owner on 2026-10-21** (plan, the scope and order of release 1.27.0, amended the same day with job J2b = RFC 028, the recommendation of a minor version, and strictly serial jobs). The architect makes the schedule and manages the release cycle; the owner authorizes it. Authorizing the plan is not a release decision: the owner still authorizes each tag and publication separately, and major version transitions remain the owner's alone (none is proposed). Changes to scope or order are announced by the architect before they are made.

## 1. Rules of the cycle

- **One job at a time, directly on `main`** (owner decision 2026-10-21: no topic branches, no concurrent jobs). Each job is a Developer Handoff under `rfcs/handoffs/NNN-slug/`, followed by a review by the architect before the next job starts.
- Every job ends with: local gates green (`fmt`, `clippy -D warnings`, `test`), CI green on the pushed commit, a review request with evidence, and the architect's verdict (Approved, Conditionally Approved, Corrections Required).
- Version numbers follow SemVer. A release bundles reviewed jobs; the architect proposes the version, the owner authorizes it, tags have no `v` prefix (`X.Y.Z`). Publishing to the AUR or Flathub is done or approved by the owner.
- **Effort sizes** are relative (S small, M medium, L large) and are not calendar estimates. Dates are not promised until the first jobs have shown the real pace; the architect will then add dates and report deviations early.
- RFC lifecycle: `proposed/` → `accepted/` → `done/` (moved when the work ships, with the release in its Status).

## 2. Release 1.27.0: first public release for Arch Linux (recommended minor)

**Objective.** A first release that is honest, safe and installable: correct output, no known data-losing defect, true compatibility statement, clean audit, green CI, a keyboard path for the main task, and an AUR source package.

**Why a minor release, not 1.26.3.** It changes visible behaviour (contain-fit of non-square logos, advisories, keyboard shortcuts) and corrects the declared minimum Rust version.

| Job | RFC / handoff | Size | Blocked by | Notes |
|---|---|---|---|---|
| J0 | Integrate the reviewed work: RFC 009, RFC 010, RFC 019 phase 1, CI workflow (PR #2) | S | none | **Done 2026-10-21:** applied linearly to `main` (`git cherry-pick -x`, no merge commits, conflict in `crates/core/Cargo.toml` resolved by keeping both `[[test]]` entries); gates green on the result (fmt, clippy, 178 tests); PR #2 closed with a pointer. The remote topic branches are no longer needed |
| J1 | RFC 024: MSRV 1.90, `zip` ≥ 9, audit triage | S–M | J0 | **Done and reviewed 2026-10-21** (all five CI jobs green, 0 vulnerabilities); required-checks idea revised, see RFC 024 |
| J1b | RFC 024 follow-up: three single-crate lock updates (`anyhow`, `event-listener`, `memmap2`) | S | J1 | **Done and reviewed 2026-10-22** (audit warnings 14 → 11, 0 vulnerabilities, gates green); added 2026-10-21 by the architect as a small hygiene job inside the authorized objective "clean audit" |
| J2 | RFC 025: window application ID | S | J1b | **Done and reviewed 2026-10-24** (181 tests, five CI jobs green; the window reports `io.github.nabbisen.Logolig` on Wayland and X11; `StartupWMClass` not needed) |
| J2a | RFC 024 follow-up: `tokio` 1.52.3 → 1.54.1 (RUSTSEC-2026-0386, new in the advisory database 2026-10-24) | S | J2 | **Done and reviewed 2026-10-24** (audit warnings 13 → 12, 181 tests, five CI jobs green); added by the architect as hygiene inside the authorized "clean audit" objective |
| J2b | RFC 028: snora 0.25.2 → 0.53.0 (same features, no `design`), retake the README screenshot | S–M | J2a | **Done and reviewed 2026-10-24** (12 of 12 visual pairs identical; toast click-through fix demonstrated; monochrome ZIP evidence closed; warning toast contrast 3.19 → 6.05; release notes must mention the toast look). Added 2026-10-21 (owner authorized). Overlay click-through fix, AA toasts, F6 zones for the keyboard job |
| J3 | RFC 015 phase 2a (**Done and reviewed 2026-10-24**: 202 tests, five CI jobs green; both usvg parse sites now data-URL-only; one combined advisory): data-URL-only SVG resolver, `svg::analyze`, one advisory | M | J2b | Closes the local-file read; ends silent misleading SVG output |
| J3b | **Proposed, not authorized:** RFC 029 toast lifetime proportional to content (advisories need 9–17 s to read, live 4 s) | S | J3 | Owner decision requested |
| J4 | RFC 019 phase 2: global shortcuts, shortcut list, comment fixes | M | J3 | First real keyboard path (choose, download all, back, pages); may use snora's F6 zones (J2b) |
| J5 | RFC 020 phase 2: AUR source package (`logolig`), packaging CI | M | J4, **owner: master icon** | Files named by the application ID; local dry run only |
| J6 | Release preparation: version 1.27.0, release notes, final gates, candidate verification | S | J5 | The architect submits a release recommendation |
| J7 | **Owner:** authorize 1.27.0, tag, first manual AUR publication (RFC 020 phase 3), then the README install section | S | J6 | Owner actions |

**Readiness criteria (all must hold; the architect reports the evidence):** all five CI jobs green on the release commit; `cargo audit` clean; RFC 009, 010, 015 (2a), 019 (phase 1 and 2), 024, 025, 028 implemented and reviewed; documentation matches behaviour; the dry-run package builds in a clean environment and the window shows the application icon; no unresolved critical finding; known limitations written in the release notes (keyboard coverage, SVG text not drawn, ja text review status).

**Not in 1.27.0.** Full focus model, SVG text rendering, `logolig-bin`, AUR automation, Flatpak, robustness items.

## 3. Release 1.28.0: keyboard and UI correctness (proposed)

| Job | RFC | Size | Notes |
|---|---|---|---|
| J8 | RFC 017: source structure (inline tests out, `app.rs` split, dead code) | L | Enabler: do it before the large UI work so reviews stay mechanical |
| J9 | RFC 021: selection-state indicators | M | Settles icons and cues before the focus indicator is designed |
| J10 | RFC 027: keyboard focus model (phase 0 spike, then Home, Result, Customize, Settings) | L | The owner's priority: makes "accessible" true |
| J11 | RFC 026: result screen layout | S | Back button must always be reachable |
| J12 | RFC 018: documentation synchronisation | S | Closed last, from the evidence of the finished work |
| J12b | RFC 008 remainder: `scripts/check-rfcs.sh` (RFC 000 invariants), wired into CI | S | The migration itself was executed on 2026-10-21 |
| T1 | RFC 020 phases 3–4: `logolig-bin`, `release.yml` with attestation, AUR automation | M | Needs the owner's `aur` environment and CI key (RFC 020 D4) |

## 4. Release 1.29.0: distribution reach and robustness (proposed)

| Job | RFC | Size | Notes |
|---|---|---|---|
| J13 | RFC 023: Flatpak (upstream files, MetaInfo, validation CI; the owner writes the manifest and submits to Flathub) | M–L | Owner prefers the ordinary Flathub route; AI-policy decision pending |
| J14 | RFC 015 phase 2b: text rendering with generic-family repair and substitution advisory | M | Includes the Flatpak font set |
| J15 | RFC 011, 012, 014: stale results, manifest validation, input limits | M | Robustness and safety |
| J16 | RFC 013, 016, 022: system theme, i18n hygiene, vtracer preset check; `vtracer` dependency-hygiene spike (RFC 024) | M | Polish and correctness |

The architect may split or reorder these as evidence arrives and will say so before acting. RFC 001–003 are withdrawn and RFC 004 is deferred (RFC 008 D4, decided 2026-10-21); RFC 027's phase 0 spike (J10) is the first step of the focus model.

## 4a. Distribution track

| Step | Owner | Architect/dev |
|---|---|---|
| Master icon (SVG or ≥ 1024 px PNG) in `packaging/icons/` | provides (request issued 2026-10-21) | J5 uses it |
| First AUR publication of `logolig` | publishes by hand | prepares and verifies the package |
| `aur` environment and CI-only key | sets up (conditions in RFC 020 D4) | T1 after confirmation |
| Flathub | reads the policy, writes the manifest and the submission | upstream files and validation only |

## 5. Release checklist applied to every release

1. Gates: `fmt`, `clippy -D warnings`, `test`, MSRV check, `cargo audit` all green in CI on the release commit.
2. Security review note by the architect (new data flows, external integrations, dependency changes); threat model updated if a data flow is new.
3. Documentation (README, `docs/src`) matches behaviour; RFC statuses and the index are current; implemented RFCs are moved to `done/` with the release.
4. Evidence kept: review requests, gate logs, CI run URLs, known limitations.
5. Release notes and a draft GitHub Release; **the owner publishes**.
6. Post-release: record deviations from plan and lessons; update this file.

## 6. Authorizations

1. **Granted 2026-10-21:** this plan, in particular the scope and order of 1.27.0 and the recommendation of a minor version.
2. **Granted 2026-10-21:** the ordering of jobs J1–J7 (strictly serial).
3. Resolved: the reviewed work was integrated into `main` as J0.
4. **Still to be authorized separately, when the time comes:** the release tag and publication of 1.27.0 (J7), the first AUR publication, and every later release.

## 7. Risks to the schedule

| Risk | Effect | Handling |
|---|---|---|
| Master icon arrives late | J5 cannot finish | Everything else in J5 proceeds; the release waits for the icon only |
| `zip` 9 needs more than renames | J1 grows | Report options; interim documented ignore only by the owner's explicit risk acceptance |
| Focus model larger than estimated | 1.28.0 slips | Phase 0 spike first; deliver per screen with reviews |
| Flathub policy outcome | 1.29.0 scope | Owner reads the policy before work starts |
| Serial execution | No parallel slack | Keep jobs small; review quickly |
