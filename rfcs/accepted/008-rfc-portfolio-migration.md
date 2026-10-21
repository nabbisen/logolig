# RFC 008 — Migrate the RFC portfolio to the RFC 000 layout

**Status.** Accepted (migration executed 2026-10-21; `scripts/check-rfcs.sh` outstanding)
**Priority.** P2 (governance; the owner prioritised AUR publication, RFC 020, first on 2026-10-21)
**Tracks.** Governance. Implements the adoption step of [RFC 000](../done/000-rfc-lifecycle-policy.md).
**Touches.** `rfcs/` layout, `rfcs/README.md`, Status fields of RFC 001–007, inbound links in `docs/src/architecture.md`.

## Summary

RFC 000 declares the 5-folder layout adopted, but `rfcs/` is still flat: RFC 001–007 sit at the top level, only `done/` exists, and `rfcs/README.md` neither lists RFC 000 nor uses RFC 000's states. This RFC performs the migration as one dedicated change, as RFC 000 recommends, and records the owner decisions it needs.

## Evidence

Verified (file listing and reading, 2026-10-21):

- `rfcs/` contains `001`–`007` flat plus `done/000-rfc-lifecycle-policy.md`; no `proposed/`, `accepted/`, `archive/`.
- `rfcs/README.md` uses `Draft | In Progress | Implemented | Withdrawn` and defines **Draft** as "reviewed and ready for implementation". RFC 000 defines **Draft** as "author still writing". The two meanings conflict.
- RFC 005 says `Accepted`; the README index says `Implemented (v1.26.0)`. RFC 006 says `Implemented (v1.26.1)`.
- RFC 000 specifies three-digit numbers (`NNN`). Until 2026-10-21 the files 001–007 were named with four digits (`0001`–`0007`). The owner ruled that `NNN` is correct; the files, titles and inbound links were renamed accordingly (history preserved with `git mv`), and RFC 008–020 were numbered `NNN` from the start.
- RFC 001 (picker popup), RFC 002 (Settings drawer "Misc" section) and RFC 004 (picker overlay, `Locale` stored as enum) describe UI or types removed or changed in v1.22.0. RFC 003 cites `domain/source.rs` (actual: `source_asset.rs`) and an `image` feature list that now includes `ico`.
- Release tags have no `v` prefix (`1.26.2`), while Status examples and existing RFCs write `v1.26.1`.

## Proposal

Use the **5-folder variant** (the design/implementation roles are separate in this project, which is the condition RFC 000 gives for `accepted/`).

1. Create `rfcs/proposed/`, `rfcs/accepted/`, `rfcs/archive/`. Keep `rfcs/done/`.
2. Move files with `git mv` (history preserved) and update the Status field **in the same commit**:

| RFC | Current | Proposed disposition | Needs owner decision |
|---|---|---|---|
| 001 Mobile UX | flat, "Draft" | `archive/`, `Withdrawn — picker popups were removed in v1.22.0; 1.d hamburger remains dormant`. Open a new RFC only if mobile work is wanted again. | Yes |
| 002 Flatten colour | flat, "Draft" | `proposed/`, after retargeting §External design to the Customize page | Yes (still wanted?) |
| 003 BMP input | flat, "Draft" | `proposed/`, after refreshing paths and feature list | Yes (still wanted?) |
| 004 zh-CN locale | flat, "Draft" | `proposed/`, after retargeting to the Settings page and `locale: Option<String>` | Yes (still wanted?) |
| 005 Microsoft logos | flat, "Accepted" | `done/`, `Implemented (1.26.0)` | No |
| 006 Drop-zone repair | flat, "Implemented" | `done/`, `Implemented (1.26.1)` | No |
| 007 Wayland hover | flat, "Draft" | `proposed/` | No |

3. Rewrite `rfcs/README.md` to the RFC 000 index shape (grouped by state, with Priority), list RFC 000, and replace the old Draft/In Progress vocabulary.
4. Fix inbound links (`grep -rl "rfcs/000" .`; the `docs/src/architecture.md` handoff table links to 001–004).
5. Add `scripts/check-rfcs.sh` implementing the "Optional CI invariants" from RFC 000 (see RFC 020 for wiring into CI).

## Decisions requested from the owner

- **D1 Numbering width — decided by the owner (2026-10-21): `NNN`.** Existing RFCs were renamed from `NNNN` to `NNN`; numbers themselves are unchanged. Remaining work under this item: confirm no external references to the old four-digit names exist (issue tracker, commit messages cannot be rewritten and are left as is).
- **D2 Version form in Status — decided (architect, under the owner's delegation, 2026-10-21): `Implemented (1.26.0)`, no `v`.** It matches the release tag format (`X.Y.Z`, no `v` prefix), so a Status value can be pasted into `git tag` / `git show` without translation.
- **D3 Handoff storage — decided by the owner (2026-10-21): `rfcs/handoffs/NNN-slug/`, tracked from the start.** Implementation handoffs are durable instructions and live next to their RFC as RFC 000 describes (`implementation-handoff*.md`, optional `README.md`). `.git-exclude/tasks/` is temporary and volatile: it may hold the prompt that starts an agent, never the handoff itself. Review requests and evidence written by the implementer remain working material outside the repository. The architect had first placed the handoffs in `.git-exclude/tasks/dev-team/` on the strength of an open recommendation; they were moved on the owner's correction. Binding design details must live in the RFC, not only in a handoff; the RFCs were updated accordingly.
- **D4 Dispositions — decided (architect, under the owner's delegation, applying the owner's philosophy, 2026-10-21):**

| RFC | Disposition | Reason |
|---|---|---|
| 001 Mobile UX | `archive/`, Withdrawn | The picker popups it refines were removed in v1.22.0; the dormant "fourth nav item" topic can return as a new RFC |
| 002 Flatten colour | `archive/`, Withdrawn | "Less is more": a new setting and validated input for a rare need; white already covers compatibility |
| 003 BMP input | `archive/`, Withdrawn | Rare logo format; every format adds parser surface and documentation; unsupported files already get a clear error |
| 004 zh-CN | `proposed/`, deferred (P3) | Wanted for reach, but a translation that is not natively reviewed can mislead; schedule after RFC 016 (fewer keys) and with a native reviewer; refresh the text to the current UI first |
| 005 Microsoft logos | `done/`, `Implemented (1.26.0)` | Shipped |
| 006 Drop-zone repair | `done/`, `Implemented (1.26.1)` | Shipped |
| 007 Wayland hover | `proposed/` (P3) | Unchanged |

Withdrawn RFCs stay in `archive/` as records (RFC 000: never delete) and can be reopened by a **new** RFC.

All of D1–D4 are closed.

## Non-goals

No RFC content is redesigned here beyond what the table states; the retargeting edits to 002/003/004 are mechanical corrections of stale references, not new design.

## Acceptance criteria

- Every file under `rfcs/<state>/` has a Status field equal to its folder.
- `rfcs/README.md` lists every RFC, including 000, with working relative links.
- `scripts/check-rfcs.sh` exits 0.
- No RFC deleted; no number reused or changed.

## Risk

Low. Documentation-only; fully reversible with git. Residual risk is a missed inbound link, which the grep in step 4 and the check script cover.

## Execution record (2026-10-21)

- Folders `accepted/`, `archive/` created next to `proposed/` and `done/`; RFC 001–007 moved with `git mv` (history preserved) and their Status fields rewritten in the RFC 000 form; `rfcs/README.md` rebuilt as a state-grouped index; the inbound links in `docs/src/architecture.md` were updated (they also pointed at the wrong directory before; RFC 018 row 13 is closed by this).
- **Outstanding:** `scripts/check-rfcs.sh` implementing the "Optional CI invariants" of RFC 000, wired into CI by RFC 020 (scheduled in `ROADMAP.md` with RFC 018). Until it exists, the check is a manual sweep (`grep` for inbound links before every move).
