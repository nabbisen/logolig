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
| [008](./done/008-rfc-portfolio-migration.md) | RFC portfolio migration to the RFC 000 layout | 1.28.0 |
| [009](./done/009-result-artifact-identity-and-zip-integrity.md) | Result artifact identity and ZIP integrity | 1.27.0 |
| [010](./done/010-non-square-raster-sources.md) | Non-square raster sources in square outputs | 1.27.0 |
| [011](./done/011-conversion-job-identity.md) | Conversion job identity and concurrent-input policy | 1.28.0 |
| [012](./done/012-web-manifest-input-validation.md) | Web manifest input validation | 1.29.0 |
| [013](./done/013-system-theme-follows-os.md) | "System" theme follows the operating system | 1.29.0 |
| [014](./done/014-ingest-validation-and-input-limits.md) | Ingest validation policy and input limits | 1.29.0 |
| [016](./done/016-i18n-completeness-and-dead-keys.md) | i18n completeness and dead-key cleanup | 1.29.0 |
| [017](./done/017-source-structure-conformance.md) | Source structure conformance (tests, file size, dead code) | 1.28.0 |
| [018](./done/018-documentation-synchronisation.md) | Documentation synchronisation (J15 done; row 9 waits for RFC 014) | 1.28.0 |
| [019](./done/019-keyboard-accessibility.md) | Keyboard accessibility (phases 1 and 2 done; the focus model is RFC 027) | 1.28.0 |
| [020](./done/020-ci-and-aur-packaging.md) | CI gates (AUR packaging dropped, phases 2–4 removed/withdrawn) | 1.27.0 (phase 1) |
| [021](./done/021-selection-state-indicators.md) | Selection-state indicators | 1.28.0 |
| [022](./done/022-vtracer-sharp-preset-verification.md) | Verify the vtracer "Sharp" preset | 1.29.0 |
| [024](./done/024-dependency-and-toolchain-baseline.md) | Dependency and toolchain baseline (MSRV, `cargo audit`) | 1.27.0 |
| [025](./done/025-window-application-id.md) | Window application ID and desktop integration | 1.27.0 |
| [026](./done/026-result-screen-layout-small-windows.md) | Result screen layout at small window heights | 1.28.0 |
| [027](./done/027-keyboard-focus-model.md) | Keyboard focus model (implemented for 1.28.0: snora controls, Tab, ring) | 1.28.0 |
| [028](./done/028-snora-upgrade-0-53.md) | Upgrade snora 0.25.2 → 0.53.0 | 1.27.0 |
| [029](./done/029-toast-lifetime-by-content.md) | Toast lifetime proportional to its content | 1.27.0 |
| [030](./done/030-bound-vectorization-cost-and-keep-ui-responsive.md) | Bound the cost of vectorization and keep the UI responsive during conversion | 1.27.0 |
| [031](./done/031-silent-file-dialog-failure.md) | A file dialog that cannot open must not fail silently | 1.28.0 |
| [032](./done/032-snora-0-57-upgrade.md) | Upgrade snora from 0.53 to 0.57 | 1.28.0 |
| [033](./done/033-new-application-logo.md) | Adopt the new application logo (docs, Linux package, Microsoft Store) | 1.28.0 |
| [034](./done/034-readable-interface-for-older-users.md) | A readable, comfortable interface for older users and low vision (1.29.0) | 1.29.0 |
| [035](./done/035-headless-ui-test-level.md) | A headless UI test level (layout, size, language and theme matrix without a window) | 1.28.0 |
| [036](./done/036-snora-0-62-upgrade.md) | Upgrade snora 0.57 → 0.65 (steps 0.62 and 0.65; the keyboard `control` and form controls) | 1.28.0 |

### Accepted (`accepted/`)

| # | Title | Priority |
| --- | --- | --- |
| [015](./accepted/015-svg-text-and-external-resources.md) | SVG input: text rendering and external resources (phased) | P1 / P2 |
| [037](./accepted/037-microsoft-store-packaging-automation.md) | Microsoft Store packaging: a guarded version, a Windows build in CI, optional submission | P1 |
| [023](./accepted/023-flatpak-publication.md) | Flatpak publication (Flathub) | P1 |
### Proposed (`proposed/`)

| # | Title | Priority |
| --- | --- | --- |
| [004](./proposed/004-locale-zh-cn.md) | Simplified Chinese locale (deferred) | P3 |
| [007](./proposed/007-wayland-drag-hover-feedback.md) | Wayland drag-hover feedback follow-up | P3 |

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
