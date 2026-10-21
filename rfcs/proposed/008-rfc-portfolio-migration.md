# RFC 008 — Migrate the RFC portfolio to the RFC 000 layout

**Status.** Proposed
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
- **D2 Version form in Status.** Recommended: `Implemented (1.26.0)`, matching the tag format with no `v` prefix.
- **D3 Handoff storage.** RFC 000 allows `rfcs/handoffs/NNN-slug/`. Current handoffs live untracked under `.git-exclude/`. Recommended: keep working handoffs untracked, and promote only reviewed, current ones to `rfcs/handoffs/` when an RFC reaches `accepted/`.
- **D4** Dispositions in the table above. *Owner reply 2026-10-21: "AUR publishment is prioritized."* The architect reads this as an ordering of the work (RFC 020 first), not as an answer to the disposition table, so **D4 remains open**; it is not urgent. Since 2026-10-21 RFC 010 and RFC 020 are `accepted/`, which already exercises the 5-folder layout.

D1 is closed; D2–D4 remain open.

## Non-goals

No RFC content is redesigned here beyond what the table states; the retargeting edits to 002/003/004 are mechanical corrections of stale references, not new design.

## Acceptance criteria

- Every file under `rfcs/<state>/` has a Status field equal to its folder.
- `rfcs/README.md` lists every RFC, including 000, with working relative links.
- `scripts/check-rfcs.sh` exits 0.
- No RFC deleted; no number reused or changed.

## Risk

Low. Documentation-only; fully reversible with git. Residual risk is a missed inbound link, which the grep in step 4 and the check script cover.
