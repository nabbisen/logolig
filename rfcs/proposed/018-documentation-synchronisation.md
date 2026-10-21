# RFC 018 — Documentation synchronisation with v1.26.2

**Status.** Proposed
**Priority.** P2 (docs are the test-design source of truth per project rules)
**Tracks.** Drift between README/`docs/src/` and the implementation.
**Touches.** `README.md`, `docs/src/architecture.md`, `functional-spec.md`, `export-spec.md`, `ui-a11y.md`, `rfcs/README.md`.

## Summary

Project rules say test cases validate the **design specification**, not the code. That only works when the specification is right. The earlier review fixed the worst drift; the items below remain at v1.26.2.

## Evidence (verified by reading docs against code)

| # | Document says | Code does | Decision needed |
|---|---|---|---|
| 1 | README, `ui-a11y.md`, `functional-spec.md`, dev instructions: active state uses a `▣` prefix | `✓` (`marker::READY`) is used in `preview_panel` and `settings_page`; `▣` appears only in a comment | Decided: RFC 021 (D1) |
| 2 | `functional-spec.md` §3 and handoff: vtracer default **Sharp** | `ExportPlan::default().vtracer_preset == Default`; the UI slider maps Simple/Detailed to Sharp/Default/PhotoRich | Decided: keep `Default`, fix docs (D2); Sharp checked by RFC 022 |
| 3 | `functional-spec.md`: three resize algorithms | `ResizeAlgorithm` has five (adds Mitchell–Netravali, Catmull–Rom) | Docs only |
| 4 | handoff: ICO frames 16/24/32/48 | `default_ico_sizes() = [16, 32, 48]`; spec §8 says ICO "fixed" but `ico_sizes` is a persisted plan field with add/remove methods and no Customize UI | Docs only; clarify "not editable in UI" |
| 5 | `architecture.md` workspace tree lists `picker_overlay.rs`, `bottom_nav.rs`, `logolig-*` crate directories | removed in v1.22.0; actual directories are `crates/{app,core,i18n}` | Docs only |
| 6 | `architecture.md` roadmap table: rows v1.26.1, v1.26.2 precede v1.26.0; the final `v1.26.0+ (option)` row contains duplicated, truncated text; "Added in Step N" history | table order and content | Docs only |
| 7 | `functional-spec.md` §16 version history ends at v1.22.0 (no 1.23–1.26.2 rows) | releases up to 1.26.2 exist (see the `architecture.md` roadmap) | Docs only |
| 8 | `ui-a11y.md`: "Every operation reachable by mouse must also be reachable by keyboard", "iced's default focus ring is preserved", labels centralised in `accessibility::label` | `label` is dead code; buttons are not keyboard-focusable (see RFC 019) | Depends on RFC 019 |
| 9 | `functional-spec.md` §2 magic bytes and extension "must agree" | not enforced | Depends on RFC 014 |
| 10 | `functional-spec.md` §3: ZIP/mono layout implied; `export-spec.md` mentions a `.logolig-<pid>-<nanos>.tmp` staging flow as the primary path | primary path is in-memory download; disk export is a legacy API used by tests | Docs only |
| 11 | README "Settings persistence": `docs.rs` badges for `logolig-app` etc. | crates not yet published (README says "after publication") | Owner: publication plan |
| 12 | Source comments cite spec sections (`§6.2`, `§12`) of a document that is not in the repository | n/a | Replace with doc/RFC references over time |
| 13 | `docs/src/architecture.md` links to `../rfcs/...`, which resolves to `docs/rfcs/` (does not exist) | the RFC folder is `../../rfcs/` from `docs/src/`; the links were already wrong before the 2026-10-21 renumbering | Docs only |

## Design

1. One change per document, each with a "verified against commit <hash>" line in the PR description (not in the document).
2. Ask the owner for D1/D2 first; do not fix docs in the direction of the code where the doc may be the intended design.
3. Docs that depend on pending RFCs (rows 8 and 9 above: keyboard parity, extension/magic policy) are updated by those RFCs, not here.
4. Add a short **Maintainers** chapter pointer to `rfcs/README.md` for the lifecycle policy (RFC 008).
5. Add a CI link check for relative Markdown links (RFC 020).

## Decisions (2026-10-21, applying the owner's philosophy: clean, safe, robust; users must not be confused)

- **D1 — marker: neither glyph is adopted as documented.** Reviewing it showed the real problem is not `✓` versus `▣`: the code reuses `✓` ("Ready") to mean "selected", and both are font-dependent text glyphs. Decision: treat it as a UI design item, now **RFC 021** (distinct cues per meaning, bundled icons instead of text glyphs). Until RFC 021 lands, row 1 is left open and the docs are not edited in either direction, so that documentation is not changed twice.
- **D2 — vtracer default: keep the implemented `Default`; correct the documents.** Reason: the default must be the choice with the most predictable result for the widest range of inputs, and `Default` is vtracer's own balanced setting; changing it would alter the output of existing users for an unverified benefit. In reviewing it, a doubt appeared about whether "Sharp" does what its name says (a higher `corner_threshold` should smooth, not sharpen). That is tracked as **RFC 022**; the docs must not call Sharp "the default" or "the sharpest" until it is resolved. Row 2 is closed by editing the documents to say "default = balanced (`Default`); Sharp is an opt-in preset".

Rows 1, 8 and 9 stay open until RFC 021, RFC 019 and RFC 014 respectively; the other rows can proceed now.

## Acceptance criteria

A reviewer can follow `docs/src/` and the README on a clean checkout and observe exactly what is described; every row above is closed or explicitly linked to the RFC that will close it.

## Risk

Low (documentation).
