# Logolig RFCs

This folder collects design documents for upcoming logolig features.
Each RFC pairs **external design** (what users see and how they interact
with the feature) with **internal design** (how implementers build it).
Some RFCs only need internal design — when the user-visible behaviour is
already self-evident from the title alone, padding the document with an
external section adds noise.

The active set of RFCs corresponds to the priority themes listed under
`v1.22.0+ (option)` in [`docs/src/architecture.md`](../docs/src/architecture.md)'s
ROADMAP table. Each RFC is intended to be a complete handoff package for
an implementer — the surrounding repository (architecture.md, export-spec.md,
ui-a11y.md) provides context but the RFC itself should be self-contained
enough that a new contributor can implement it without spelunking through
prior version history.

## Index

| # | Title | Scope | Status |
| --- | --- | --- | --- |
| [001](./001-mobile-ux-refinements.md) | Mobile UX refinements | medium (4 sub-topics) | Draft |
| [002](./002-user-specified-flatten-color.md) | User-specified flatten color | medium | Draft |
| [003](./003-bmp-input-support.md) | BMP input support | small | Draft |
| [004](./004-locale-zh-cn.md) | Simplified Chinese locale | medium | Draft |
| [005](./005-microsoft-app-logos.md) | Microsoft app logos | small | Implemented (v1.26.0) |
| [006](./006-drop-zone-drag-drop-repair.md) | Drop-zone drag-and-drop repair | small | Implemented (v1.26.1) |
| [007](./007-wayland-drag-hover-feedback.md) | Wayland drag-hover feedback follow-up | small | Draft |

### Proposed under the RFC 000 lifecycle policy

The RFC portfolio is being migrated to the layout defined by
[RFC 000](./done/000-rfc-lifecycle-policy.md) (see RFC 008). Until that
migration lands, RFC 001–007 stay in this folder with the old status
vocabulary, and new RFCs follow RFC 000 and live in `proposed/`. In RFC 000
terms, **Proposed** means open for review; the implementer should not start.
Priorities are the architect's recommendation and require owner approval.

**Accepted** (review complete; implementation handoffs in [`handoffs/`](./handoffs/)):

| # | Title | Priority |
| --- | --- | --- |
| [020](./accepted/020-ci-and-aur-packaging.md) | GitHub Actions CI and AUR packaging (phased) | P1 |
| [010](./accepted/010-non-square-raster-sources.md) | Non-square raster sources in square outputs | P1 |
| [009](./accepted/009-result-artifact-identity-and-zip-integrity.md) | Result artifact identity and ZIP integrity | P1 |
| [015](./accepted/015-svg-text-and-external-resources.md) | SVG input: text rendering and external resources (phased: spike first) | P1 / P2 |
| [019](./accepted/019-keyboard-accessibility.md) | Keyboard accessibility (phased: verify and document first) | P1 / P2 |

**Proposed** (open for review):

| # | Title | Priority |
| --- | --- | --- |
| [008](./proposed/008-rfc-portfolio-migration.md) | Migrate the RFC portfolio to the RFC 000 layout | P2 |
| [011](./proposed/011-conversion-job-identity.md) | Conversion job identity and concurrent-input policy | P2 |
| [012](./proposed/012-web-manifest-input-validation.md) | Web manifest input validation | P2 |
| [013](./proposed/013-system-theme-follows-os.md) | "System" theme follows the operating system | P2 |
| [014](./proposed/014-ingest-validation-and-input-limits.md) | Ingest validation policy and input limits | P2 |
| [017](./proposed/017-source-structure-conformance.md) | Source structure conformance (tests, file size, dead code) | P2 |
| [018](./proposed/018-documentation-synchronisation.md) | Documentation synchronisation with v1.26.2 | P2 |
| [021](./proposed/021-selection-state-indicators.md) | Selection-state indicators | P2 |
| [016](./proposed/016-i18n-completeness-and-dead-keys.md) | i18n completeness and dead-key cleanup | P3 |
| [022](./proposed/022-vtracer-sharp-preset-verification.md) | Verify the vtracer "Sharp" preset | P3 |

"Status: Draft" (in the table above, for RFC 001–007) means the RFC is reviewed and ready for implementation
but no implementer has started yet. It moves to "In Progress" once a
version branch is opened, and to "Implemented (vX.Y.Z)" once merged.

## Template

The template is **lightweight by default**: an implementer reading the
document should be able to start coding within minutes. Skip sections
that don't apply. For medium-or-larger RFCs, add the optional sections
listed below.

```markdown
# RFC NNN: <Title>

- **Status**: Draft | In Progress | Implemented (vX.Y.Z) | Withdrawn
- **Target version**: vX.Y.Z (or "v1.22.0+ option")
- **Author**: <name>
- **Created**: YYYY-MM-DD

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
