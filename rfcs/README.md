# Logolig RFCs

RFCs record **what was decided and why**. Their lifecycle follows
[RFC 000](./done/000-rfc-lifecycle-policy.md) (the 5-folder variant):

| Folder | State |
| --- | --- |
| `proposed/` | open for review; implementers do not start |
| `accepted/` | review complete; implementation may start; handoffs in [`handoffs/`](./handoffs/) |
| `done/` | shipped in a release (the Status field names it, for example `Implemented (1.26.0)`) |
| `archive/` | withdrawn or superseded; kept as a record, never deleted |

The **folder is the source of truth** for the state; the Status field inside each file must agree.
Numbers are three digits (`NNN`), assigned once, never reused and never changed. Execution documents
for implementers live in `handoffs/NNN-slug/` and inherit their state from the RFC.
The order of work is in [`ROADMAP.md`](../ROADMAP.md). Priorities are the architect's
recommendation; the owner authorizes the plan.

## Index

### Implemented (`done/`)

| # | Title | Shipped in |
| --- | --- | --- |
| [000](./done/000-rfc-lifecycle-policy.md) | RFC lifecycle policy | policy in effect |
| [005](./done/005-microsoft-app-logos.md) | Microsoft app logos | 1.26.0 |
| [006](./done/006-drop-zone-drag-drop-repair.md) | Drop-zone drag-and-drop repair | 1.26.1 |

### Accepted (`accepted/`)

RFC 009 and RFC 010 are implemented on `main` and move to `done/` with the release that ships them.

| # | Title | Priority |
| --- | --- | --- |
| [020](./accepted/020-ci-and-aur-packaging.md) | GitHub Actions CI and AUR packaging (phased) | P1 |
| [024](./accepted/024-dependency-and-toolchain-baseline.md) | Dependency and toolchain baseline (MSRV, `cargo audit`) | P1 |
| [025](./accepted/025-window-application-id.md) | Window application ID and desktop integration | P1 |
| [028](./accepted/028-snora-upgrade-0-53.md) | Upgrade snora 0.25.2 → 0.53.0 | P1 |
| [030](./accepted/030-bound-vectorization-cost-and-keep-ui-responsive.md) | Bound the cost of vectorization and keep the UI responsive during conversion | P1 |
| [029](./accepted/029-toast-lifetime-by-content.md) | Toast lifetime proportional to its content | P1 |
| [015](./accepted/015-svg-text-and-external-resources.md) | SVG input: text rendering and external resources (phased) | P1 / P2 |
| [019](./accepted/019-keyboard-accessibility.md) | Keyboard accessibility (phase 1 done; phase 2 shortcuts next) | P1 |
| [027](./accepted/027-keyboard-focus-model.md) | Keyboard focus model (phased: spike first) | P1 |
| [009](./accepted/009-result-artifact-identity-and-zip-integrity.md) | Result artifact identity and ZIP integrity | P1 (implemented, unreleased) |
| [010](./accepted/010-non-square-raster-sources.md) | Non-square raster sources in square outputs | P1 (implemented, unreleased) |
| [008](./accepted/008-rfc-portfolio-migration.md) | RFC portfolio migration to the RFC 000 layout | P2 (script outstanding) |

### Proposed (`proposed/`)

| # | Title | Priority |
| --- | --- | --- |
| [011](./proposed/011-conversion-job-identity.md) | Conversion job identity and concurrent-input policy | P2 |
| [012](./proposed/012-web-manifest-input-validation.md) | Web manifest input validation | P2 |
| [013](./proposed/013-system-theme-follows-os.md) | "System" theme follows the operating system | P2 |
| [014](./proposed/014-ingest-validation-and-input-limits.md) | Ingest validation policy and input limits | P2 |
| [017](./proposed/017-source-structure-conformance.md) | Source structure conformance (tests, file size, dead code) | P2 |
| [018](./proposed/018-documentation-synchronisation.md) | Documentation synchronisation | P2 |
| [021](./proposed/021-selection-state-indicators.md) | Selection-state indicators | P2 |
| [023](./proposed/023-flatpak-publication.md) | Flatpak publication (Flathub) | P2 |
| [026](./proposed/026-result-screen-layout-small-windows.md) | Result screen layout at small window heights | P2 |
| [031](./proposed/031-silent-file-dialog-failure.md) | A file dialog that cannot open must not fail silently | P2 |
| [004](./proposed/004-locale-zh-cn.md) | Simplified Chinese locale (deferred) | P3 |
| [007](./proposed/007-wayland-drag-hover-feedback.md) | Wayland drag-hover feedback follow-up | P3 |
| [016](./proposed/016-i18n-completeness-and-dead-keys.md) | i18n completeness and dead-key cleanup | P3 |
| [022](./proposed/022-vtracer-sharp-preset-verification.md) | Verify the vtracer "Sharp" preset | P3 |

### Archive (`archive/`)

| # | Title | Reason |
| --- | --- | --- |
| [001](./archive/001-mobile-ux-refinements.md) | Mobile UX refinements | Withdrawn: the picker popups were removed in v1.22.0 |
| [002](./archive/002-user-specified-flatten-color.md) | User-specified flatten colour | Withdrawn: a new setting for a rare need ("less is more") |
| [003](./archive/003-bmp-input-support.md) | BMP input support | Withdrawn: rare format, extra parser surface |

## Template

The template is **lightweight by default**: an implementer reading the
document should be able to start coding within minutes. Skip sections
that don't apply. For medium-or-larger RFCs, add the optional sections
listed below.

```markdown
# RFC NNN: <Title>

**Status.** Proposed | Accepted | Implemented (X.Y.Z) | Withdrawn — reason | Superseded by RFC NNN
**Priority.** P1 | P2 | P3
**Tracks.** What this addresses.
**Touches.** Where the work lands.

## Summary

Two or three sentences. What is the change in plain language.

## Background  *(optional — omit when the title is self-explanatory)*

Why does this exist now? What changed in the world or in the codebase
that makes this worth doing?

## External design

What does the user see? Screen mocks, copy, behavioural rules. If the
feature is purely internal (no user-visible surface), say so and skip.

## Internal design

How does an implementer build this? Module placement, type signatures,
data flow, module boundaries.

---

The four sections above are the lightweight default. The four below are
recommended once the RFC reaches medium scope or higher.

## Requirements  *(medium+)*

Numbered, testable statements. Each requirement should be implementable
and verifiable in isolation.

## Design  *(medium+ — replaces "Internal design" when present)*

A more thorough version of "Internal design" that walks through state
transitions, edge cases, and rejected alternatives.

## Test plan  *(medium+)*

Which test files get touched, what each new test verifies. Distinguish
unit tests from integration tests from manual checks.

## Security considerations  *(medium+ — write "N/A" if truly nothing)*

Any way this feature can be misused, leak data, or expand the attack
surface. For desktop favicon work this is mostly N/A but consider:
filesystem writes, untrusted input parsing, network calls (logolig has
none), persisted state side-channels.
```

## Principles

- **English only** — matches the rest of `docs/src/`.
- **Self-contained** — an implementer should not need to read 5 other
  files to start. Quote or restate the relevant context inline.
- **Decisions over options** — RFCs should land on a recommendation, not
  open a debate. If alternatives were considered and rejected, briefly
  note why; don't reopen the discussion in the RFC body.
- **Cite the ROADMAP row** — link back to `docs/src/architecture.md` for the
  one-paragraph summary that justified the RFC's existence.
