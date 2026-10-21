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
| 1 | README, `ui-a11y.md`, `functional-spec.md`, dev instructions: active state uses a `▣` prefix | `✓` (`marker::READY`) is used in `preview_panel` and `settings_page`; `▣` appears only in a comment | Which glyph is the design? (D1) |
| 2 | `functional-spec.md` §3 and handoff: vtracer default **Sharp** | `ExportPlan::default().vtracer_preset == Default`; the UI slider maps Simple/Detailed to Sharp/Default/PhotoRich | Fix docs or change default? (D2; changing default alters output) |
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

## Decisions requested

- **D1** Active-state marker: `✓` (code) or `▣` (docs)? Note `✓` is also the "Ready" marker, so one glyph carries two meanings; `▣` is more distinct but less obvious to new users. Recommendation: keep `✓`, update docs, accept the dual meaning because the surrounding text disambiguates. Needs a UI/UX call.
- **D2** vtracer default: documented Sharp or implemented Default? Recommendation: keep implemented `Default` (no output change), correct the docs. The 1.4.2 notes say Sharp was only ever an opt-in preset.

## Acceptance criteria

A reviewer can follow `docs/src/` and the README on a clean checkout and observe exactly what is described; every row above is closed or explicitly linked to the RFC that will close it.

## Risk

Low (documentation).
