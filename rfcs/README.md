# Logolig RFCs

This folder collects design documents for upcoming logolig features.
Each RFC pairs **external design** (what users see and how they interact
with the feature) with **internal design** (how implementers build it).
Some RFCs only need internal design — when the user-visible behaviour is
already self-evident from the title alone, padding the document with an
external section adds noise.

The active set of RFCs corresponds to the priority themes listed under
`v1.22.0+ (option)` in [`docs/architecture.md`](../docs/architecture.md)'s
ROADMAP table. Each RFC is intended to be a complete handoff package for
an implementer — the surrounding repository (architecture.md, export-spec.md,
ui-a11y.md) provides context but the RFC itself should be self-contained
enough that a new contributor can implement it without spelunking through
prior version history.

## Index

| # | Title | Scope | Status |
| --- | --- | --- | --- |
| [0001](./0001-mobile-ux-refinements.md) | Mobile UX refinements | medium (4 sub-topics) | Draft |
| [0002](./0002-user-specified-flatten-color.md) | User-specified flatten color | medium | Draft |
| [0003](./0003-bmp-input-support.md) | BMP input support | small | Draft |
| [0004](./0004-locale-zh-cn.md) | Simplified Chinese locale | medium | Draft |
| [0005](./0005-microsoft-app-logos.md) | Microsoft app logos | small | Implemented (v1.26.0) |
| [0006](./0006-drop-zone-drag-drop-repair.md) | Drop-zone drag-and-drop repair | small | Implemented (v1.26.1) |

"Status: Draft" means the RFC is reviewed and ready for implementation
but no implementer has started yet. It moves to "In Progress" once a
version branch is opened, and to "Implemented (vX.Y.Z)" once merged.

## Template

The template is **lightweight by default**: an implementer reading the
document should be able to start coding within minutes. Skip sections
that don't apply. For medium-or-larger RFCs, add the optional sections
listed below.

```markdown
# RFC NNNN: <Title>

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

- **English only** — matches the rest of `docs/`.
- **Self-contained** — an implementer should not need to read 5 other
  files to start. Quote or restate the relevant context inline.
- **Decisions over options** — RFCs should land on a recommendation, not
  open a debate. If alternatives were considered and rejected, briefly
  note why; don't reopen the discussion in the RFC body.
- **Cite the ROADMAP row** — link back to `docs/architecture.md` for the
  one-paragraph summary that justified the RFC's existence.
